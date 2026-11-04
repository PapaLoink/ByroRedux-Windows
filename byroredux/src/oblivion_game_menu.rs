//! Original Oblivion in-game menus read directly from the user's installed
//! MenuXml + original DDS/font archives. Engine state only fills authored traits
//! and runtime item templates; it does not draw a substitute menu.
use anyhow::{Context, Result};
use byroredux_debug_ui::{GameMenuArt, GameMenuHitTarget, InventorySnapshot};
use byroredux_menuxml::{MenuRenderer, ScreenTraits};

pub(crate) fn add_original_section_navigation(menu: &mut MenuRenderer, root: &str, selected: usize) -> Result<()> {
    // All four glyphs are taken from Oblivion's original all_tabs_icons.dds,
    // including their authored 64x64 atlas crop regions (inventory, magic,
    // stats and map). Egui contributes no drawn menu chrome.
    let screen = menu.screen();
    let x = (screen.width - 79.0).max(0.0);
    let y = (screen.height / 2.0 - 148.0).max(0.0);
    let mut tiles = format!(
        "<locus>2</locus><x>{x}</x><y>{y}</y><depth>99</depth>"
    );
    for section in 0..4 {
        let crop = section * 64;
        let alpha = if section == selected { 255 } else { 168 };
        tiles.push_str(&format!(
            "<rect name=\"byro_section_{section}\">                <id>{}</id><target>2</target><locus>2</locus><depth>2</depth>                <width>64</width><height>64</height><x>0</x><y>{}</y>                <image name=\"byro_section_art_{section}\">                    <filename>Menus\\Shared\\all_tabs_icons.dds</filename>                    <width>64</width><height>64</height><cropx>0</cropx>                    <cropy>{crop}</cropy><alpha>{alpha}</alpha>                </image></rect>",
            600 + section, section * 68
        ));
    }
    menu.graft_inline_fragment(&tiles, root, "byro_section_navigation")?;
    Ok(())
}

pub(crate) fn raster(assets: &crate::hud::HudAssets, menu: &mut MenuRenderer) -> GameMenuArt {
    let targets = menu.hit_targets().into_iter().map(|target| GameMenuHitTarget {
        id: target.id,
        name: target.name,
        rect: [target.rect.x, target.rect.y, target.rect.w, target.rect.h],
    }).collect();
    let (width, height) = menu.frame_size();
    GameMenuArt {
        width: width as usize,
        height: height as usize,
        rgba: menu.render_frame(assets).to_vec(),
        targets,
    }
}

pub(crate) fn load(
    assets: &crate::hud::HudAssets,
    path: &str,
    root: &str,
    screen: ScreenTraits,
) -> Result<MenuRenderer> {
    let mut menu = MenuRenderer::load(assets, path, screen)
        .with_context(|| format!("loading original {path}"))?;
    // The original menu manager animates this opacity from zero on opening.
    menu.set_override(root, "alpha", 255.0);
    menu.set_override(root, "visible", 2.0);
    Ok(menu)
}

const OPTIONS_PAGES: [(&str, &str); 5] = [
    ("options_menu", "OptionsMenu"),
    ("gameplay_menu", "GameplayMenu"),
    ("video_menu", "VideoMenu"),
    ("audio_menu", "AudioMenu"),
    ("controls_menu", "ControlsMenu"),
];

fn options_pages(
    assets: &crate::hud::HudAssets,
    screen: ScreenTraits,
    settings: &[byroredux_core::settings::SettingEntry],
) -> Result<Vec<GameMenuArt>> {
    let mut pages: Vec<GameMenuArt> = OPTIONS_PAGES.iter().map(|&(name, root)| {
        let mut menu = load(assets, &format!("menus\\options\\{name}.xml"), root, screen)?;
        if name == "options_menu" {
            menu.set_override(root, "user0", 0.0);
        }
        if name == "gameplay_menu" {
            if let Some(entry) = settings.iter().find(|s|
                s.id == byroredux_core::settings::builtin::SHOW_CROSSHAIR_SETTING_ID)
            {
                if let byroredux_core::settings::SettingValue::Bool(on) = entry.value {
                    menu.set_override_str("gameplay_crosshair_button", "string",
                        if on { "On" } else { "Off" });
                }
            }
        }
        if name == "controls_menu" {
            if let Some(entry) = settings.iter().find(|s| s.id == "controls.invert_y") {
                if let byroredux_core::settings::SettingValue::Bool(on) = entry.value {
                    menu.set_override_str("controls_invert_y_button", "string",
                        if on { "On" } else { "Off" });
                }
            }
            if let Some(entry) = settings.iter().find(|s|
                s.id == "controls.mouse_sensitivity")
            {
                if let (
                    byroredux_core::settings::SettingValue::Number(value),
                    byroredux_core::settings::SettingControl::Slider { min, max, .. },
                ) = (&entry.value, &entry.control) {
                    menu.set_override("controls_sensitivity_slider", "user5",
                        (100.0 * (value - min) / (max - min)).clamp(0.0, 100.0));
                }
            }
            menu.set_override(root, "user0", 1.0);
            menu.set_override("controls_page_1", "visible", 2.0);
            menu.set_override("controls_page_2", "visible", 2.0);
            menu.set_override("controls_page_1", "depth", 3.0);
            menu.set_override("controls_page_2", "depth", 3.0);
        }
        for button in [
            "gameplay_return_button", "video_return_button",
            "audio_return_button", "controls_return_button",
            "controls_return_button_2",
        ] {
            menu.set_override(button, "user20", 1.0);
        }
        Ok(raster(assets, &mut menu))
    }).collect::<Result<Vec<_>>>()?;

    // The original Save/Load XML's list prototype receives real numbered
    // ByroRedux save slots, not a single generic quicksave placeholder.
    let save_dir = crate::save_io::discover_save_dir();
    let slots = byroredux_save::disk::slots_by_recency(&save_dir);
    let new_slot = (0..u32::MAX).find(|candidate| !slots.contains(candidate)).unwrap_or(0);
    for (kind, root, template, pane, return_button) in [
        ("save", "SaveMenu", "save_game_template", "save_list_pane", "save_button_return"),
        ("load", "LoadMenu", "load_game_template", "load_list_pane", "load_return_button"),
    ] {
        let mut menu = load(assets, &format!("menus\\options\\{kind}_menu.xml"), root, screen)?;
        menu.set_override(return_button, "user20", 1.0);
        let rows: Vec<(String, String, u32)> = if kind == "save" {
            std::iter::once((
                format!("byro_save_new_{new_slot}"),
                "New Save".to_string(), new_slot,
            )).chain(slots.iter().take(5).map(|&slot| (
                format!("byro_save_slot_{slot}"),
                format!("Save {slot} - double-click to overwrite"), slot,
            ))).collect()
        } else {
            slots.iter().take(6).map(|&slot| (
                format!("byro_load_slot_{slot}"), format!("Save {slot}"), slot,
            )).collect()
        };
        for (i, (name, text, _slot)) in rows.iter().enumerate() {
            let item = menu.instantiate_template(template, pane, name)?;
            if !menu.set_runtime_target_id(item, 101 + i as i32) {
                anyhow::bail!("original {kind} template produced no row");
            }
            menu.set_override(name, "user0", i as f32);
            menu.set_override(name, "listindex", i as f32);
            menu.set_override(name, "visible", 2.0);
            menu.set_override_str(name, "user3", text);
        }
        pages.push(raster(assets, &mut menu));
    }
    Ok(pages)
}

/// Oblivion ICON subrecords commonly store `icons\\...dds` while menu
/// XML expects paths relative to `textures\\menus`. Resolve that record
/// spelling without synthesizing a replacement icon or copying an asset.
fn installed_item_icon(path: &str) -> String {
    let trimmed = path.trim().replace('/', "\\");
    let relative = trimmed.strip_prefix("textures\\").unwrap_or(&trimmed);
    if relative.to_ascii_lowercase().starts_with("menus\\") {
        relative.to_owned()
    } else if relative.to_ascii_lowercase().starts_with("icons\\") {
        format!("Menus\\{relative}")
    } else if relative.is_empty() {
        String::new()
    } else {
        format!("Menus\\Icons\\{relative}")
    }
}

fn inventory_category(category: &str) -> usize {
    match category {
        "Weapon" | "Ammo" => 0,
        "Armor" | "Apparel" | "Clothing" => 1,
        "Ingredient" | "Aid" | "Apparatus" => 2,
        "Book" | "Scroll" | "Note" => 3,
        _ => 4,
    }
}

fn inventory_pages(
    assets: &crate::hud::HudAssets,
    screen: ScreenTraits,
    snapshot: Option<&InventorySnapshot>,
    scroll: [usize; 5],
) -> Result<Vec<GameMenuArt>> {
    (0..5).map(|category| {
        let mut menu = load(
            assets, "menus\\main\\inventory_menu.xml", "InventoryMenu", screen,
        )?;
        menu.set_override("InventoryMenu", "user0", (category + 1) as f32);
        for field in ["user1", "user2", "user3", "user4"] {
            menu.set_override("InventoryMenu", field, -1.0);
        }
        // The engine can calculate carried weight but doesn't yet expose
        // vanilla-compatible gold, armour and maximum encumbrance fields.
        menu.set_override("InventoryMenu", "user6",
            snapshot.map_or(0.0, |inventory| inventory.total_weight));
        for field in ["user7", "user8", "user9"] {
            menu.set_override_str("InventoryMenu", field, "-");
        }

        let items: Vec<_> = snapshot.into_iter()
            .flat_map(|s| s.items.iter())
            .filter(|item| inventory_category(item.category) == category)
            .collect();
        let offset = scroll[category].min(items.len().saturating_sub(6));
        let rows: Vec<_> = items.iter().skip(offset).take(6).copied().collect();
        menu.set_override("inv_list_contents", "user0", items.len() as f32);
        menu.set_override("inv_scroll_bar", "user7", offset as f32);
        menu.set_override("inv_scroll_bar", "user8", 6.0);
        for (position, item) in rows.into_iter().enumerate() {
            let name = format!("byro_item_{}", item.index);
            let tile = menu.instantiate_template("inv_item_template", "inv_list_contents", &name)?;
            if !menu.set_runtime_target_id(tile, 1000 + position as i32) {
                anyhow::bail!("Oblivion inventory template did not create a clickable row");
            }
            menu.set_override(&name, "visible", 2.0);
            menu.set_override(&name, "user8", 2.0);
            menu.set_override(&name, "listindex", (offset + position) as f32);
            menu.set_override(&name, "user11", item.index as f32);
            menu.set_override_str(&name, "user1", &item.name);
            menu.set_override_str(&name, "user2", &item.value.to_string());
            menu.set_override_str(&name, "user3", &format!("{:.1}", item.weight));
            menu.set_override_str(&name, "user4", "");
            menu.set_override_str(&name, "user5", "");
            menu.set_override_str(&name, "user6", &installed_item_icon(&item.icon_path));
            menu.set_override_str(&name, "user7", &if item.count > 1 {
                item.count.to_string()
            } else {
                String::new()
            });
            menu.set_override(&name, "user10", if item.equipped { 2.0 } else { 0.0 });
        }
        add_original_section_navigation(&mut menu, "InventoryMenu", 0)?;
        Ok(raster(assets, &mut menu))
    }).collect()
}

/// Screen coordinates follow Bethesda's original 960-point vertical design,
/// preserving the XML's cropping/aspect-ratio rules on widescreen displays.
pub(crate) fn authored_screen(size: (u32, u32)) -> Result<ScreenTraits> {
    let (width, height) = size;
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        anyhow::bail!("invalid Oblivion game menu dimensions {width}x{height}");
    }
    Ok(ScreenTraits::new(960.0 * width as f32 / height as f32, 960.0))
}

/// Install the real original Escape, Tab and Options artwork. The active ESM
/// protects non-Oblivion games from being routed through these XML files.
pub(crate) fn install(
    debug_ui: &mut byroredux_debug_ui::DebugUiState,
    args: &[String],
    screen_size: (u32, u32),
) -> Result<bool> {
    let (esm, data) = match crate::cli_args::esm_discovery_root(args) {
        Ok(pair) => pair,
        Err(_) => return Ok(false),
    };
    if !std::path::Path::new(&esm).file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("Oblivion.esm"))
    {
        return Ok(false);
    }
    let assets = crate::hud::HudAssets::oblivion(&data)?;
    let screen = authored_screen(screen_size)?;
    let mut pause = load(&assets, "menus\\options\\pause_menu.xml", "PauseMenu", screen)?;
    for name in [
        "pause_return_button", "pause_save_button", "pause_load_button",
        "pause_options_button", "pause_quit_button",
    ] {
        pause.set_override(name, "user20", 1.0);
    }
    debug_ui.install_game_menu_art(
        raster(&assets, &mut pause),
        inventory_pages(&assets, screen, None, [0; 5])?,
        options_pages(&assets, screen, &[])?,
    ).map_err(anyhow::Error::msg)?;
    log::info!("Original Oblivion in-game MenuXml installed from {}", data.display());
    Ok(true)
}

/// Re-render the original Options and numbered Save/Load list screens from
/// the live SettingsRegistry and the actual existing save directory.
pub(crate) fn refresh_options(
    world: &byroredux_core::ecs::World,
    debug_ui: &mut byroredux_debug_ui::DebugUiState,
    args: &[String],
    screen_size: (u32, u32),
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
    let settings = world.try_resource::<byroredux_core::settings::SettingsRegistry>()
        .map(|r| r.entries().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let assets = crate::hud::HudAssets::oblivion(&data)?;
    let pages = options_pages(&assets, authored_screen(screen_size)?, &settings)?;
    debug_ui.update_original_options(pages).map_err(anyhow::Error::msg)
}

/// Rebuild original item-template rows when opening Tab, after changes, or
/// upon a known inventory update. The game remains the source of item state.
pub(crate) fn refresh_inventory(
    debug_ui: &mut byroredux_debug_ui::DebugUiState,
    args: &[String],
    screen_size: (u32, u32),
    inventory: Option<&InventorySnapshot>,
    scroll: [usize; 5],
) -> Result<()> {
    let (esm, data) = crate::cli_args::esm_discovery_root(args)
        .map_err(anyhow::Error::msg)?;
    if !std::path::Path::new(&esm).file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("Oblivion.esm"))
    {
        return Ok(());
    }
    let assets = crate::hud::HudAssets::oblivion(&data)?;
    let screen = authored_screen(screen_size)?;
    debug_ui.update_original_inventory(inventory_pages(&assets, screen, inventory, scroll)?)
        .map_err(anyhow::Error::msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_mapping_matches_original_five_inventory_tabs() {
        assert_eq!(inventory_category("Weapon"), 0);
        assert_eq!(inventory_category("Armor"), 1);
        assert_eq!(inventory_category("Aid"), 2);
        assert_eq!(inventory_category("Book"), 3);
        assert_eq!(inventory_category("Misc"), 4);
    }

    #[test]
    #[ignore = "requires the owner's installed original Oblivion archives"]
    fn installed_original_menu_templates_smoke() {
        let data = std::env::var_os("BYROREDUX_OBLIVION_DATA")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\steamapps\common\Oblivion\Data"));
        let assets = crate::hud::HudAssets::oblivion(&data).unwrap();
        let screen = authored_screen((1280, 720)).unwrap();
        let mut pause = load(&assets, "menus\\options\\pause_menu.xml", "PauseMenu", screen).unwrap();
        let picture = raster(&assets, &mut pause);
        assert!(picture.rgba.iter().any(|v| *v != 0));
        assert!(picture.targets.iter().any(|t| t.name == "pause_return_button"));
        let inventory = InventorySnapshot {
            items: vec![byroredux_debug_ui::InventoryItemView {
                index: 42, form_id: 0x123, name: "Iron Longsword".into(),
                category: "Weapon", icon_path: String::new(), details: String::new(), count: 1,
                value: 50, weight: 8.0, equipped: false,
                equippable: true, consumable: false,
            }],
            total_weight: 8.0,
        };
        let pages = inventory_pages(&assets, screen, Some(&inventory), [0; 5]).unwrap();
        assert_eq!(pages.len(), 5);
        assert!(pages[0].targets.iter().any(|t| t.name == "byro_item_42"));
        assert!(!pages[1].targets.iter().any(|t| t.name == "byro_item_42"));
        assert!(pages.iter().all(|p| p.rgba.chunks_exact(4).any(|q| q[3] != 0)));
        let options = options_pages(&assets, screen, &[]).unwrap();
        assert_eq!(options.len(), 7);
        assert!(options[0].targets.iter().any(|t| t.name == "options_gameplay_button"));
        assert!(options[5].targets.iter().any(|t| t.name.starts_with("byro_save_new_")));
        assert!(options[6].targets.iter().any(|t| t.name == "load_return_button"));
    }
}
