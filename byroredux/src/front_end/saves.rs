//! Validation and session selection reuse the existing engine save format.
use anyhow::{bail, Context, Result};
use byroredux_core::ecs::World;
use byroredux_save::{Snapshot, SaveRegistry};
use std::{io::Read, path::{Path,PathBuf}};

pub(super) struct SaveSlot { pub number: u32, pub label: String }
fn snapshot(dir: &Path, number: u32, fingerprint: u64) -> Result<Snapshot> {
    let file = std::fs::File::open(byroredux_save::disk::slot_path(dir,number))?;
    if file.metadata()?.len()>64*1024*1024 { bail!("save exceeds front-end 64 MiB bound"); }
    let mut bytes=Vec::new(); file.take(64*1024*1024+1).read_to_end(&mut bytes)?;
    if bytes.len()>64*1024*1024 { bail!("save grew past bound"); }
    Ok(byroredux_save::decode(&bytes,fingerprint)?)
}
fn session(snap: &Snapshot) -> Option<(String,Vec<String>)> {
    if let Some(cell)=crate::save_io::snapshot_cell_context(snap) {
        Some((cell.esm_path,vec!["--cell".into(),cell.cell_editor_id]))
    } else {
        let exterior=crate::save_io::snapshot_exterior_context(snap)?;
        Some((exterior.esm_path,vec!["--wrld".into(),exterior.worldspace_key,
            "--grid".into(),format!("{},{}",exterior.grid.0,exterior.grid.1),
            "--radius".into(),exterior.radius_load.to_string()]))
    }
}
pub(super) fn slots(world: &World) -> Vec<SaveSlot> {
    let dir=world.resource::<crate::save_io::SaveState>().dir.clone();
    let fingerprint=world.resource::<SaveRegistry>().schema_fingerprint();
    byroredux_save::disk::slots_by_recency(&dir).into_iter().take(256).filter_map(|number| {
        let snap=snapshot(&dir,number,fingerprint).ok()?;
        byroredux_save::validate_snapshot_types(&world.resource::<SaveRegistry>(),&snap).ok()?;
        let (esm,args)=session(&snap)?;
        if !Path::new(&esm).file_name()?.to_str()?.eq_ignore_ascii_case("Oblivion.esm") { return None; }
        Some(SaveSlot { number, label:format!("Engine slot {number} — {}",args.join(" ")) })
    }).collect()
}

/// NOT_SAVED_BY_DESIGN: one-shot startup recipe consumed before the first scene.
pub(crate) struct SceneStartArgs(pub Vec<String>);
impl byroredux_core::ecs::Resource for SceneStartArgs {}

/// Preserve content/rendering options, replacing the one-shot front-end location.
fn scene_args(effective: Vec<String>, location: Vec<String>) -> Vec<String> {
    let mut args=Vec::new();
    let mut iter=effective.into_iter();
    while let Some(arg)=iter.next() {
        if ["--cell","--grid","--wrld","--radius","--load"].contains(&arg.as_str()) { iter.next(); }
        else if arg!="--oblivion-main-menu" { args.push(arg); }
    }
    args.extend(location);
    if !args.iter().any(|a|a=="--hud") { args.push("--hud".into()); }
    args
}

/// Resolve the installed tutorial marker, never substitute a guessed start pose.
pub(super) fn new_game_start(args: &[String]) -> Result<(Vec<String>,crate::save_io::PlayerPose)> {
    use byroredux_plugin::esm::cell::CellRef;
    use byroredux_core::math::Vec3;
    let (esm,_)=crate::cli_args::esm_discovery_root(args).map_err(anyhow::Error::msg)?;
    if !Path::new(&esm).file_name().is_some_and(|n| n.eq_ignore_ascii_case("Oblivion.esm")) {
        bail!("New requires the selected Oblivion installation");
    }
    let masters=crate::cli_args::parse_all_string_args(args,"--master");
    let paths: Vec<&str>=masters.iter().map(String::as_str).chain(std::iter::once(esm.as_str())).collect();
    let (index,_)=crate::cell_loader::load_order::parse_record_indexes_shared(&paths,false)?;
    // Installed Oblivion.esm: REFR 00032AB5 CGPlayerStartMarker, base 00000034.
    let (cell,marker)=index.cells.locate_refr(0x00032AB5).context("CGPlayerStartMarker is missing from the selected load order")?;
    let CellRef::Interior {editor_id}=cell else {bail!("tutorial start marker is not in an interior");};
    let position=crate::cell_loader::position_zup_to_yup(marker.position);
    let rotation=crate::cell_loader::rotation_zup_to_yup_quat(marker.rotation);
    let (yaw,pitch)=crate::scene::yaw_pitch_from_forward(rotation * -Vec3::Z);
    if !byroredux_physics::PhysicsWorld::translation_is_sane(position) || !yaw.is_finite() || !pitch.is_finite() {
        bail!("tutorial marker has an invalid pose");
    }
    let mut recipe=scene_args(args.to_vec(),vec!["--cell".into(),editor_id.into()]);
    recipe.retain(|a|a!="--fly");
    if !recipe.iter().any(|a|a=="--player") { recipe.push("--player".into()); }
    Ok((recipe,crate::save_io::PlayerPose {position:position.to_array(),yaw,pitch,character_mode:true}))
}

pub(super) fn prepare_load(world: &mut World, number: u32) -> Result<()> {
    let dir=world.resource::<crate::save_io::SaveState>().dir.clone();
    let fingerprint=world.resource::<SaveRegistry>().schema_fingerprint();
    let snap=snapshot(&dir,number,fingerprint)?;
    byroredux_save::validate_snapshot_types(&world.resource::<SaveRegistry>(),&snap)?;
    let (esm,location)=session(&snap).context("save has no world/cell session")?;
    if !PathBuf::from(&esm).file_name().is_some_and(|name| name.eq_ignore_ascii_case("Oblivion.esm")) {
        bail!("save belongs to another game");
    }
    let effective=crate::cli_args::effective_args();
    let (selected,_)=crate::cli_args::esm_discovery_root(&effective).map_err(anyhow::Error::msg)?;
    if Path::new(&esm).canonicalize()? != Path::new(&selected).canonicalize()? {
        bail!("save uses a different installation; launch with the saved plugin set");
    }
    let masters=crate::save_io::snapshot_cell_context(&snap).map(|c|c.masters)
        .or_else(||crate::save_io::snapshot_exterior_context(&snap).map(|c|c.masters)).unwrap_or_default();
    let canonical=|paths:Vec<String>|->Result<Vec<PathBuf>> {paths.into_iter().map(|p|Ok(PathBuf::from(p).canonicalize()?)).collect()};
    if canonical(masters)? != canonical(crate::cli_args::parse_all_string_args(&effective,"--master"))? {
        bail!("save master/load order differs from the selected session");
    }
    if let Some(cell)=crate::save_io::snapshot_cell_context(&snap) {
        crate::cell_loader::validate_cell_loadable(&cell.masters,&cell.esm_path,&cell.cell_editor_id)?;
    } else if let Some(exterior)=crate::save_io::snapshot_exterior_context(&snap) {
        crate::cell_loader::build_exterior_world_context(&exterior.masters,&exterior.esm_path,
            exterior.grid.0,exterior.grid.1,exterior.radius_load,Some(&exterior.worldspace_key))?;
    }
    let args=scene_args(effective,location);
    let output=crate::save_io::queue_startup_load(world,&number.to_string());
    if crate::save_io::command_output_is_failure(&output) { bail!("{}",output.lines.join(" | ")); }
    world.insert_resource(SceneStartArgs(args));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn front_end_save_browser_filters_corrupt_and_cross_game_slots() {
        let dir=tempfile::tempdir().unwrap();
        let mut world=World::new();
        world.insert_resource(crate::save_io::build_save_registry());
        world.insert_resource(crate::save_io::SaveState::new(dir.path().into(),4));
        for (slot,esm) in [(0,"Oblivion.esm"),(1,"FalloutNV.esm")] {
            world.insert_resource(crate::cell_loader::CurrentCellContext {
                cell_editor_id:"SyntheticInterior".into(),esm_path:esm.into(),masters:vec![],
            });
            let registry=world.resource::<SaveRegistry>();
            let snap=byroredux_save::save_world(&world,&registry).unwrap();
            let bytes=byroredux_save::encode(&snap,registry.schema_fingerprint()).unwrap();
            std::fs::write(byroredux_save::disk::slot_path(dir.path(),slot),bytes).unwrap();
        }
        std::fs::write(byroredux_save::disk::slot_path(dir.path(),2),b"corrupt engineering fixture").unwrap();
        let slots=slots(&world);
        assert_eq!(slots.len(),1);
        assert_eq!(slots[0].number,0);
        assert!(prepare_load(&mut world,2).is_err());
        assert!(world.try_resource::<SceneStartArgs>().is_none());
    }
    #[test]
    fn front_end_scene_recipe_replaces_location_preserves_assets_and_enables_hud() {
        let args=["engine","--esm","Oblivion.esm","--oblivion-main-menu","--cell","OldCell",
            "--grid","1,2","--wrld","OldWorld","--radius","3","--load","7","--bsa","Meshes.bsa","--hud"]
            .map(String::from).to_vec();
        let recipe=scene_args(args,vec!["--cell".into(),"Prison".into()]);
        assert_eq!(recipe,["engine","--esm","Oblivion.esm","--bsa","Meshes.bsa","--hud","--cell","Prison"]);
        let fnv=scene_args(vec!["--esm".into(),"FalloutNV.esm".into()],vec!["--cell".into(),"Test".into()]);
        assert_eq!(fnv,["--esm","FalloutNV.esm","--cell","Test","--hud"]);
    }
    #[test]
    fn front_end_new_game_rejects_other_game_before_reading_records() {
        assert!(new_game_start(&["--esm".into(),"FalloutNV.esm".into()]).is_err());
    }
    #[test]
    #[ignore="requires the user's installed original Oblivion data"]
    fn front_end_original_new_game_marker_smoke() {
        let data=std::env::var_os("BYROREDUX_OBLIVION_DATA").map(PathBuf::from)
            .unwrap_or_else(||PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data"));
        let args=vec!["--esm".into(),data.join("Oblivion.esm").to_string_lossy().into_owned(),"--oblivion-main-menu".into()];
        let (recipe,pose)=new_game_start(&args).unwrap();
        assert_eq!(crate::cli_args::parse_string_arg(&recipe,"--cell").as_deref(),Some("ImperialDungeon01"));
        assert!((pose.position[0]-567.8674).abs()<0.01);
        assert!((pose.position[1]+101.5979).abs()<0.01);
        assert!((pose.position[2]-53.9168).abs()<0.01);
        assert!(recipe.iter().any(|a|a=="--player"));
        println!("Original CGPlayerStartMarker: {:?}, yaw {}",pose.position,pose.yaw);
    }
}
