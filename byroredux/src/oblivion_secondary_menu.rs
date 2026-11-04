//! Live Oblivion Stats, Magic and Map MenuXml pages.
//! These draw the owner's original menu artwork and populate its own authored
//! tiles and list templates, without any egui substitute surfaces.
use std::collections::HashMap;

use anyhow::Result;
use byroredux_core::ecs::{Resource, World};
use byroredux_core::ecs::components::{ActorValues, FactionRanks};
use byroredux_core::character::{Background, CharacterLevel};
use byroredux_menuxml::ScreenTraits;
use byroredux_plugin::esm::records::{EsmIndex, SpellType};
use byroredux_debug_ui::GameMenuArt;

use crate::oblivion_game_menu::{add_original_section_navigation, load, raster};

#[derive(Debug, Clone)]
struct SpellRecordView {
    name: String,
    kind: SpellType,
    cost: u32,
}

/// Immutable original-game display names from the active load order.
#[derive(Debug, Default)]
pub(crate) struct OriginalMenuIndex {
    spells: HashMap<u32, SpellRecordView>,
    races: HashMap<u32, String>,
    classes: HashMap<u32, String>,
    factions: HashMap<u32, String>,
}
impl Resource for OriginalMenuIndex {}

/// The equipped magic-list selection, validated against the actor's learned
/// SpellList before it can be changed. This is transient runtime UI state.
#[derive(Debug, Default)]
pub(crate) struct OriginalMenuSpellSelection(pub u32);
impl Resource for OriginalMenuSpellSelection {}

pub(crate) fn select_known_spell(world: &mut World, form_id: u32) -> bool {
    let Some(player) = crate::systems::PlayerEntity::get(world) else {
        return false;
    };
    if !world.get::<byroredux_scripting::SpellList>(player)
        .is_some_and(|list| list.0.contains(&form_id))
    {
        return false;
    }
    world.insert_resource(OriginalMenuSpellSelection(form_id));
    true
}

pub(crate) fn install_index(world: &mut World, index: &EsmIndex) {
    let originals = OriginalMenuIndex {
        spells: index.spells.iter().map(|(&id, spell)| {
            (id, SpellRecordView {
                name: if spell.full_name.trim().is_empty() {
                    spell.editor_id.clone()
                } else {
                    spell.full_name.clone()
                },
                kind: spell.spell_type,
                cost: spell.cost,
            })
        }).collect(),
        races: index.races.iter().map(|(&id, r)| (id, r.full_name.clone())).collect(),
        classes: index.classes.iter().map(|(&id, c)| (id, c.full_name.clone())).collect(),
        factions: index.factions.iter().map(|(&id, f)| (id, f.full_name.clone())).collect(),
    };
    world.insert_resource(originals);
}

const ATTRIBUTES: [&str; 8] = [
    "Strength", "Intelligence", "Willpower", "Agility",
    "Speed", "Endurance", "Personality", "Luck",
];

const SKILLS: [&str; 21] = [
    "Armorer", "Athletics", "Blade", "Block", "Blunt", "HandToHand",
    "HeavyArmor", "Alchemy", "Alteration", "Conjuration", "Destruction",
    "Illusion", "Mysticism", "Restoration", "Acrobatics", "LightArmor",
    "Marksman", "Mercantile", "Security", "Sneak", "Speechcraft",
];

fn player_value(world: &World, actor: u32, label: &str) -> Option<(f32, f32)> {
    let id = world.try_resource::<byroredux_scripting::ActorValueNames>()?.resolve(label)?;
    let values = world.get::<ActorValues>(actor)?;
    let entry = values.get(id)?;
    Some((entry.current(), entry.base))
}

/// Stats: five original character, attributes, skills, factions, and
/// miscellaneous screens. Unavailable live metrics remain blank; the
/// original source-file sample values must never be shown as player stats.
pub(crate) fn stats_pages(
    world: &World, assets: &crate::hud::HudAssets, screen: ScreenTraits,
) -> Result<Vec<GameMenuArt>> {
    let player = crate::systems::PlayerEntity::get(world);
    let player_name = player.and_then(|entity| {
        let symbol = world.get::<byroredux_core::ecs::Name>(entity)?.0;
        let pool = world.try_resource::<byroredux_core::string::StringPool>()?;
        pool.resolve(symbol).map(str::to_owned)
    }).unwrap_or_else(|| "Player".to_owned());
    let (race, class) = player.and_then(|p| world.get::<Background>(p)
        .map(|b| (b.race_form_id, b.class_form_id)))
        .and_then(|(race, class)| {
            let index = world.try_resource::<OriginalMenuIndex>()?;
            Some((
                index.races.get(&race).cloned().unwrap_or_default(),
                index.classes.get(&class).cloned().unwrap_or_default(),
            ))
        }).unwrap_or_default();
    let level = player.and_then(|p| world.get::<CharacterLevel>(p).map(|v| v.level))
        .unwrap_or(1);
    let vitals = crate::inventory::vitals_snapshot(world).unwrap_or_default();
    let value_for = |needle: &str| vitals.iter().find(|v|
        v.label.eq_ignore_ascii_case(needle)).map(|v| (v.current, v.max))
        .unwrap_or((0.0, 0.0));

    let skills: Vec<_> = SKILLS.iter().filter_map(|&name| {
        let (value, base) = player.and_then(|p| player_value(world, p, name))?;
        Some((name.to_owned(), value, base))
    }).collect();
    let factions: Vec<_> = player.and_then(|p| world.get::<FactionRanks>(p)
        .map(|v| v.0.clone())).unwrap_or_default();
    let index = world.try_resource::<OriginalMenuIndex>();

    (0..5).map(|tab| {
        let mut menu = load(assets, "menus\\main\\stats_menu.xml", "StatsMenu", screen)?;
        menu.set_override("StatsMenu", "user0", (tab + 1) as f32);

        // Stats page one: the source's demo character is replaced with the
        // actual named player and its recorded current / maximum pools.
        menu.set_override_str("stat_p1", "user1", &player_name);
        menu.set_override_str("stat_p1", "user2", &race);
        menu.set_override_str("stat_p1", "user3", "");
        menu.set_override_str("stat_p1", "user4", &class);
        menu.set_override("stat_p1", "user5", level as f32);
        for (i, label) in ["Health", "Magicka", "Fatigue"].iter().enumerate() {
            let (current, max) = value_for(label);
            menu.set_override("stat_p1", &format!("user{}", 6 + i), current);
            menu.set_override("stat_p1", &format!("user{}", 9 + i), max);
        }
        for i in 12..=14 {
            menu.set_override("stat_p1", &format!("user{i}"), 0.0);
        }
        // Page two's 8 attributes each have current and base layers.
        for (i, attr) in ATTRIBUTES.iter().enumerate() {
            let (current, base) = player.and_then(|p| player_value(world, p, attr))
                .unwrap_or((0.0, 0.0));
            menu.set_override("stat_p2", &format!("user{}", i + 1), current);
            menu.set_override("stat_p2", &format!("user{}", i + 9), base);
        }
        if tab == 2 {
            menu.set_override("stat_p3", "user1", 10.0);
            menu.set_override("stat_p3", "user2", 0.0);
            menu.set_override("stat_p3", "user4", 2.0);
            menu.set_override("stat_p3_scroll_bar", "user7", 0.0);
            for (i, (name, value, base)) in skills.iter().take(7).enumerate() {
                let tile_name = format!("byro_skill_{i}");
                let tile = menu.instantiate_template(
                    "stat_skill_template", "stat_p3_window_pane", &tile_name,
                )?;
                menu.set_runtime_target_id(tile, 300 + i as i32);
                menu.set_override(&tile_name, "listindex", i as f32);
                menu.set_override_str(&tile_name, "user4", name);
                menu.set_override(&tile_name, "user1", *value);
                menu.set_override(&tile_name, "user2", *base);
                menu.set_override(&tile_name, "user3", 0.0);
                menu.set_override_str(&tile_name, "user5", "");
            }
        }
        if tab == 3 {
            for (i, (faction_id, rank)) in factions.iter().take(3).enumerate() {
                let name = format!("byro_faction_{i}");
                let tile = menu.instantiate_template(
                    "stat_faction_template", "stat_p4_window_pane", &name,
                )?;
                menu.set_runtime_target_id(tile, 400 + i as i32);
                menu.set_override(&name, "listindex", i as f32);
                menu.set_override_str(&name, "user1", index.as_ref()
                    .and_then(|catalog| catalog.factions.get(faction_id))
                    .map(String::as_str).unwrap_or("Faction"));
                menu.set_override_str(&name, "user2", &format!("Rank {rank}"));
                menu.set_override_str(&name, "user3", "");
            }
        }
        if tab == 4 {
            let misc = [("Level", level.to_string()),
                ("Known Spells", player.and_then(|p| world.get::<byroredux_scripting::SpellList>(p))
                    .map_or(0, |v| v.0.len()).to_string()),
                ("Factions", factions.len().to_string())];
            for (i, (label, value)) in misc.iter().enumerate() {
                let name = format!("byro_misc_{i}");
                let tile = menu.instantiate_template(
                    "stat_misc_template", "stat_p5_window_pane", &name,
                )?;
                menu.set_runtime_target_id(tile, 450 + i as i32);
                menu.set_override(&name, "listindex", i as f32);
                menu.set_override_str(&name, "user1", label);
                menu.set_override_str(&name, "user2", value);
            }
        }
        add_original_section_navigation(&mut menu, "StatsMenu", 2)?;
        Ok(raster(assets, &mut menu))
    }).collect()
}

/// Magic: categories use original icons, list rows, scroll clipping and
/// metadata from the player's actual learned spells.
pub(crate) fn magic_pages(
    world: &World, assets: &crate::hud::HudAssets, screen: ScreenTraits,
) -> Result<Vec<GameMenuArt>> {
    let known_ids = crate::systems::PlayerEntity::get(world)
        .and_then(|p| world.get::<byroredux_scripting::SpellList>(p).map(|v| v.0.clone()))
        .unwrap_or_default();
    let catalog = world.try_resource::<OriginalMenuIndex>();
    let selection = world.try_resource::<OriginalMenuSpellSelection>()
        .map(|current| current.0);
    let mut known: Vec<(u32, SpellRecordView)> = known_ids.iter().filter_map(|id| {
        catalog.as_ref()?.spells.get(id).map(|v| (*id, v.clone()))
    }).collect();
    known.sort_by(|a, b| a.1.name.to_ascii_lowercase().cmp(&b.1.name.to_ascii_lowercase()));
    (0..5).map(|tab| {
        let mut menu = load(assets, "menus\\main\\magic_menu.xml", "MagicMenu", screen)?;
        menu.set_override("MagicMenu", "user0", (tab + 1) as f32);
        for field in ["user1", "user2", "user3", "user4"] {
            menu.set_override("MagicMenu", field, -1.0);
        }
        let rows: Vec<_> = known.iter().filter(|(_, v)| match tab {
            0 => true,
            1 => matches!(v.kind, SpellType::Spell),
            2 => matches!(v.kind, SpellType::Power | SpellType::LesserPower),
            3 => matches!(v.kind, SpellType::Ability | SpellType::Disease),
            _ => false, // Scrolls live in inventory and are not learnt spells.
        }).take(6).collect();
        menu.set_override("MagicMenu", "user5", rows.len() as f32);
        menu.set_override("magic_list_contents", "user0", rows.len() as f32);
        menu.set_override("magic_scroll_bar", "user7", 0.0);
        menu.set_override("magic_scroll_bar", "user8", 6.0);
        // No invented learned-spell icons: those come from MGEF school
        // metadata when the cast-system supplies a compatible effect icon.
        menu.set_override_str("magic_item", "user6", "");
        for (pos, (form_id, spell)) in rows.iter().enumerate() {
            let name = format!("byro_spell_{form_id}");
            let tile = menu.instantiate_template("magic_item_template", "magic_list_contents", &name)?;
            menu.set_runtime_target_id(tile, 700 + pos as i32);
            menu.set_override(&name, "listindex", pos as f32);
            menu.set_override(&name, "user8", 2.0);
            menu.set_override_str(&name, "user1", &spell.name);
            menu.set_override_str(&name, "user2", "");
            menu.set_override_str(&name, "user3", "");
            menu.set_override_str(&name, "user4", "");
            menu.set_override_str(&name, "user5", &spell.cost.to_string());
            menu.set_override_str(&name, "user6", "");
            menu.set_override(&name, "user10", if selection == Some(*form_id) { 2.0 } else { 0.0 });
        }
        add_original_section_navigation(&mut menu, "MagicMenu", 1)?;
        Ok(raster(assets, &mut menu))
    }).collect()
}

/// Map: original five-page menu, original Cyrodiil map DDS, current
/// location, and quest-journal rows from live QuestObjectiveState.
pub(crate) fn map_pages(
    world: &World, assets: &crate::hud::HudAssets, screen: ScreenTraits,
) -> Result<Vec<GameMenuArt>> {
    let location = if let Some(cell) = world.try_resource::<crate::cell_loader::CurrentCellContext>() {
        cell.cell_editor_id.clone()
    } else if let Some(ext) = world.try_resource::<crate::cell_loader::CurrentExteriorContext>() {
        format!("{} ({}, {})", ext.worldspace_key, ext.grid.0, ext.grid.1)
    } else {
        String::new()
    };
    let objectives = crate::objectives::snapshot(world).unwrap_or_default();
    (0..5).map(|tab| {
        let mut menu = load(assets, "menus\\main\\map_menu.xml", "MapMenu", screen)?;
        menu.set_override("MapMenu", "user0", (tab + 1) as f32);
        menu.set_override_str("MapMenu", "user1", &location);
        menu.set_override_str("MapMenu", "user2", "");
        // Gamebryo normally populates this texture at runtime. Cyrodiil's
        // *actual installed original* raster is shipped with the game.
        menu.set_override_str("map_world_map", "filename",
            "Menus\\Map\\World\\cyrodiil_resized.dds");
        menu.set_override("map_world_map", "width", 570.0);
        menu.set_override("map_world_map", "height", 570.0);
        menu.set_override("map_world_map", "user12", 0.0);
        menu.set_override("map_world_map", "user13", 0.0);
        if tab >= 2 {
            menu.set_override("map_log_window_pane", "height", 650.0);
            menu.set_override("map_log_window_pane", "user0", objectives.len() as f32);
            for (pos, objective) in objectives.iter().take(4).enumerate() {
                let name = format!("byro_quest_{pos}");
                let tile = menu.instantiate_template("map_item_template",
                    "map_log_window_pane", &name)?;
                menu.set_runtime_target_id(tile, 800 + pos as i32);
                menu.set_override(&name, "listindex", pos as f32);
                menu.set_override_str(&name, "user1", &objective.quest);
                menu.set_override_str(&name, "user2", &objective.text);
                menu.set_override_str(&name, "user3", &objective.text);
                menu.set_override_str(&name, "user4", "");
                menu.set_override(&name, "user6", 0.0);
            }
        }
        add_original_section_navigation(&mut menu, "MapMenu", 3)?;
        Ok(raster(assets, &mut menu))
    }).collect()
}

/// Rebuild the original secondary Tab sections when the player opens Tab.
pub(crate) fn refresh(
    world: &World, ui: &mut byroredux_debug_ui::DebugUiState,
    args: &[String], size: (u32, u32),
) -> Result<()> {
    let (esm, data) = match crate::cli_args::esm_discovery_root(args) {
        Ok(pair) => pair,
        Err(_) => return Ok(()),
    };
    if !std::path::Path::new(&esm).file_name()
        .is_some_and(|f| f.eq_ignore_ascii_case("Oblivion.esm"))
    {
        return Ok(());
    }
    let assets = crate::hud::HudAssets::oblivion(&data)?;
    let screen = crate::oblivion_game_menu::authored_screen(size)?;
    let magic = magic_pages(world, &assets, screen)?;
    let stats = stats_pages(world, &assets, screen)?;
    let map = map_pages(world, &assets, screen)?;
    ui.install_original_sections(magic, stats, map)
        .map_err(anyhow::Error::msg)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oblivion_skill_list_has_no_duplicate_categories() {
        let unique: std::collections::HashSet<_> = SKILLS.into_iter().collect();
        assert_eq!(unique.len(), 21);
    }

    #[test]
    #[ignore = "uses original installed Oblivion XML/texture archives"]
    fn installed_oblivion_stats_magic_map_render_and_navigate() {
        let data = std::env::var_os("BYROREDUX_OBLIVION_DATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(
                r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data"
            ));
        let assets = crate::hud::HudAssets::oblivion(&data).unwrap();
        let screen = crate::oblivion_game_menu::authored_screen((1280, 720)).unwrap();
        let world = World::new();
        for (name, pages) in [
            ("stats", stats_pages(&world, &assets, screen).unwrap()),
            ("magic", magic_pages(&world, &assets, screen).unwrap()),
            ("map", map_pages(&world, &assets, screen).unwrap()),
        ] {
            assert_eq!(pages.len(), 5, "{name} page count");
            for (tab, page) in pages.iter().enumerate() {
                assert!(page.rgba.chunks_exact(4).any(|p| p[3] > 0),
                    "{name} tab {tab} has no original art");
                for section in 0..4 {
                    assert!(page.targets.iter().any(|t|
                        t.name == format!("byro_section_{section}")
                    ), "{name} tab {tab} missing cross-section target {section}");
                }
                let prefix = match name {
                    "stats" => "stat_tabs_p",
                    "magic" => "magic_tabs_p",
                    _ => "map_tabs_p",
                };
                // Vanilla source XML intentionally clears `target` on the
                // currently selected category tab. Only the other four
                // tabs stay clickable.
                let tabs = page.targets.iter().filter(|t|
                    t.name.strip_prefix(prefix)
                        .is_some_and(|v| v.chars().all(|c| c.is_ascii_digit()))
                ).count();
                assert!(tabs >= 4, "{name} tab {tab} missing other source tab targets");
            }
        }
    }
}
