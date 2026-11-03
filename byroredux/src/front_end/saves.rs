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
    let dir=crate::save_io::discover_save_dir();
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

pub(super) fn prepare_load(world: &mut World, number: u32) -> Result<()> {
    let dir=crate::save_io::discover_save_dir();
    let fingerprint=world.resource::<SaveRegistry>().schema_fingerprint();
    let snap=snapshot(&dir,number,fingerprint)?;
    byroredux_save::validate_snapshot_types(&world.resource::<SaveRegistry>(),&snap)?;
    if let Some(cell)=crate::save_io::snapshot_cell_context(&snap) {
        crate::cell_loader::validate_cell_loadable(&cell.masters,&cell.esm_path,&cell.cell_editor_id)?;
    }
    let (esm,location)=session(&snap).context("save has no world/cell session")?;
    if !PathBuf::from(&esm).file_name().is_some_and(|name| name.eq_ignore_ascii_case("Oblivion.esm")) {
        bail!("save belongs to another game");
    }
    let mut args=Vec::new();
    let effective=crate::cli_args::effective_args();
    let mut iter=effective.iter();
    while let Some(arg)=iter.next() {
        if ["--cell","--grid","--wrld","--radius","--load"].contains(&arg.as_str()) { iter.next(); }
        else if arg!="--oblivion-main-menu" { args.push(arg.clone()); }
    }
    args.extend(location);
    let output=crate::save_io::queue_startup_load(world,&number.to_string());
    if crate::save_io::command_output_is_failure(&output) { bail!("{}",output.lines.join(" | ")); }
    world.insert_resource(SceneStartArgs(args));
    Ok(())
}
