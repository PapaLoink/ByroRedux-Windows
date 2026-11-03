//! The Fallout 3 / New Vegas title screen, drawn from the game's own
//! `menus\options\start_menu.xml` (`--main-menu` with `--hud`'s corpus).
//!
//! The XML is only half of the menu: the game's code faded it in, built
//! the option list from `lb_item_template` and moved the highlight. This
//! module is that half — it sets the same traits the XML's comments say
//! code sets (`StartMenu.alpha`, `main_title.alpha`, the list box's
//! `_alpha`/`_highlight_y`/`_selected_height`, each item's `_y`,
//! `listindex`, `string`, `_selected`), rasterizes through the shared
//! MenuXml renderer and uploads the frame as the UI overlay, as the HUD
//! does ([`crate::hud`]). Pointer and keys arrive from the window events;
//! a chosen option comes back as a [`StartMenuAction`] for the App.
//!
//! The menus are authored on a 960-unit-tall screen (`pause_background`
//! centres itself at `y` 480), so the frame is that tall and as wide as the
//! window's aspect makes it; the overlay quad scales it to the window.

use byroredux_menuxml::{MenuRenderer, ScreenTraits};
use winit::keyboard::KeyCode;

use crate::hud::HudAssets;

const MENU_PATH: &str = "menus\\options\\start_menu.xml";
/// The authored screen height of the FO3/FNV menus.
const MENU_HEIGHT: f32 = 960.0;
/// `StartMenu._item_height`.
const ITEM_HEIGHT: f32 = 48.0;
const LIST: &str = "main_container";

/// What choosing an option asks the App to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StartMenuAction {
    Continue,
    NewGame,
    Load,
    Settings,
    Credits,
    Downloads,
    Quit,
}

/// One title-screen option.
struct Option_ {
    /// The game setting holding its label.
    label: &'static str,
    /// That setting's built-in value. These settings are engine defaults —
    /// `FalloutNV.esm` carries none of them — so a plugin's GMST overrides
    /// the text and this is what shows otherwise (the values the 1.4.0.525
    /// exe's static initializers construct each `Setting` with).
    default: &'static str,
    action: StartMenuAction,
    /// Listed only when a save exists.
    needs_save: bool,
    /// Selectable (an option this engine cannot carry out yet is listed
    /// dimmed).
    enabled: bool,
}

/// The main-menu options, in the order the game lists them. The original
/// builds one table for the title screen and the pause menu (StartMenu.cpp,
/// the 0x7CC6E0 initializer) — label setting, action, and a menu mask: 1
/// title screen, 2 pause menu. These are the mask-1 rows: `sSave` and
/// `sHelp` are pause-only. Continue and Load are listed only when a save
/// exists (`FindFirstFile` on the save folder in the 0x7CBAF0 setup).
const OPTIONS: [Option_; 7] = [
    Option_ { label: "sContinue", default: "Continue", action: StartMenuAction::Continue, needs_save: true, enabled: true },
    Option_ { label: "sNew", default: "New", action: StartMenuAction::NewGame, needs_save: false, enabled: true },
    Option_ { label: "sLoad", default: "Load", action: StartMenuAction::Load, needs_save: true, enabled: true },
    Option_ { label: "sSettings", default: "Settings", action: StartMenuAction::Settings, needs_save: false, enabled: true },
    // No credits roll or DLC list exists yet: listed, dimmed.
    Option_ { label: "sCrew", default: "Credits", action: StartMenuAction::Credits, needs_save: false, enabled: false },
    Option_ {
        label: "sDownloads",
        default: "Downloadable Content",
        action: StartMenuAction::Downloads,
        needs_save: false,
        enabled: false,
    },
    Option_ { label: "sQuit", default: "Quit", action: StartMenuAction::Quit, needs_save: false, enabled: true },
];

pub(crate) struct StartMenu {
    renderer: MenuRenderer,
    assets: HudAssets,
    texture_handle: u32,
    width: u32,
    height: u32,
    /// The listed options, top to bottom: indices into [`OPTIONS`].
    listed: Vec<usize>,
    /// Index into `listed`.
    selected: usize,
    /// The list box's evaluated rect (menu units), refreshed each render.
    list: [f32; 3],
    dirty: bool,
    upload_buffer: Vec<u8>,
}

impl StartMenu {
    /// Load the title screen when `--main-menu` runs on the FO3/FNV corpus.
    /// `None` (logged) otherwise; the App then falls back to its own menu.
    pub(crate) fn launch(
        ctx: &mut byroredux_renderer::vulkan::context::VulkanContext,
        world: &byroredux_core::ecs::World,
        args: &[String],
    ) -> Option<Self> {
        let (assets, _, _) = crate::hud::open_hud_assets(args)?;
        if !assets.profile().is_fallout() {
            return None;
        }
        let (window_w, window_h) = ctx.swapchain_extent();
        let width = (MENU_HEIGHT * window_w as f32 / window_h.max(1) as f32).round() as u32;
        let height = MENU_HEIGHT as u32;
        let screen = ScreenTraits::new(width as f32, height as f32);
        let mut renderer =
            match MenuRenderer::load_with_profile(&assets, MENU_PATH, screen, assets.profile().menu_profile()) {
                Ok(renderer) => renderer,
                Err(error) => {
                    log::warn!("start menu: {MENU_PATH} did not load ({error}) — using the native menu");
                    return None;
                }
            };
        let saves = crate::save_io::has_saves(world);
        let listed: Vec<usize> = (0..OPTIONS.len()).filter(|&i| saves || !OPTIONS[i].needs_save).collect();
        // What `StartMenu`'s code set once the fade-in finished. `user0`:
        // the original sets it when the aspect ratio is over 4:3 (1.3333).
        renderer.set_override("StartMenu", "alpha", 255.0);
        renderer.set_override("StartMenu", "user0", f32::from(u8::from(width as f32 / height as f32 > 1.3333)));
        renderer.set_override("main_title", "alpha", 255.0);
        renderer.set_override(LIST, "_alpha", 255.0);
        renderer.set_override(LIST, "_enabled", 1.0);
        renderer.set_override(LIST, "_number_of_visible_items", listed.len() as f32);
        renderer.set_override(LIST, "_selected_height", ITEM_HEIGHT);
        renderer.set_override("lb_scrollbar", "_number_of_items", listed.len() as f32);
        renderer.set_override("lb_scrollbar", "_number_of_visible_items", listed.len() as f32);
        for (row, &option) in listed.iter().enumerate() {
            let option = &OPTIONS[option];
            let name = item_name(row);
            if let Err(error) = renderer.instantiate_template("lb_item_template", LIST, &name) {
                log::warn!("start menu: option list could not be built ({error}) — using the native menu");
                return None;
            }
            let label = byroredux_scripting::obscript_state::game_setting_string(world, option.label)
                .unwrap_or_else(|| option.default.to_owned());
            renderer.set_override_str(&name, "string", &label);
            renderer.set_override(&name, "_y", row as f32 * ITEM_HEIGHT);
            renderer.set_override(&name, "listindex", row as f32);
            renderer.set_override(&name, "_enabled", f32::from(u8::from(option.enabled)));
        }
        let registered = {
            let upload_ctx = byroredux_renderer::upload_ctx!(ctx);
            ctx.texture_registry
                .register_rgba(upload_ctx, width, height, &vec![0u8; (width * height * 4) as usize])
        };
        let texture_handle = match registered {
            Ok(handle) => handle,
            Err(error) => {
                log::warn!("start menu: overlay texture registration failed ({error:#})");
                return None;
            }
        };
        log::info!(
            "start menu: loaded {MENU_PATH} ({width}x{height}, texture {texture_handle}, {} options{})",
            listed.len(),
            if saves { "" } else { ", no saves" }
        );
        let mut menu = Self {
            renderer,
            assets,
            texture_handle,
            width,
            height,
            listed,
            selected: 0,
            list: [0.0; 3],
            dirty: true,
            upload_buffer: Vec::new(),
        };
        let first = (0..menu.listed.len()).find(|&row| menu.option(row).enabled).unwrap_or(0);
        menu.select(first);
        Some(menu)
    }

    /// The option listed at `row`.
    fn option(&self, row: usize) -> &'static Option_ {
        &OPTIONS[self.listed[row]]
    }

    /// Highlight the option at `row` (an enabled one; disabled rows are
    /// skipped by the callers).
    fn select(&mut self, row: usize) {
        if let Some(previous) = Some(self.selected).filter(|&p| p != row) {
            self.renderer.set_override(&item_name(previous), "_selected", 0.0);
        }
        self.selected = row;
        self.renderer.set_override(&item_name(row), "_selected", 1.0);
        self.renderer.set_override(LIST, "_highlight_y", row as f32 * ITEM_HEIGHT);
        self.dirty = true;
    }

    /// The enabled row under the pointer, given as fractions of the window.
    fn option_at(&self, x: f32, y: f32) -> Option<usize> {
        let (x, y) = (x * self.width as f32, y * self.height as f32);
        let [left, top, width] = self.list;
        if x < left || x > left + width || y < top {
            return None;
        }
        let row = ((y - top) / ITEM_HEIGHT) as usize;
        (row < self.listed.len() && self.option(row).enabled).then_some(row)
    }

    /// The pointer moved to (`x`, `y`), fractions of the window: hovering
    /// an option highlights it, as the game's mouse-over did.
    pub(crate) fn pointer_moved(&mut self, x: f32, y: f32) {
        if let Some(index) = self.option_at(x, y) {
            if index != self.selected {
                self.select(index);
            }
        }
    }

    /// A click at (`x`, `y`): the option under it, if any.
    pub(crate) fn click(&mut self, x: f32, y: f32) -> Option<StartMenuAction> {
        let row = self.option_at(x, y)?;
        self.select(row);
        Some(self.option(row).action)
    }

    /// Up/Down move the highlight over the enabled options; Enter or Space
    /// chooses.
    pub(crate) fn key(&mut self, key: KeyCode) -> Option<StartMenuAction> {
        let step: isize = match key {
            KeyCode::ArrowUp | KeyCode::KeyW => -1,
            KeyCode::ArrowDown | KeyCode::KeyS => 1,
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space => {
                return Some(self.option(self.selected).action);
            }
            _ => return None,
        };
        let enabled: Vec<bool> = (0..self.listed.len()).map(|row| self.option(row).enabled).collect();
        self.select(step_to_enabled(&enabled, self.selected, step));
        None
    }

    /// Rasterize and upload when something changed; the handle to
    /// composite either way.
    pub(crate) fn render_and_upload(
        &mut self,
        ctx: &mut byroredux_renderer::vulkan::context::VulkanContext,
    ) -> u32 {
        if !self.dirty {
            return self.texture_handle;
        }
        self.dirty = false;
        let pixels = self.renderer.render_frame(&self.assets);
        // The XML draws no backdrop: over black, so the paused world behind
        // does not show through.
        self.upload_buffer.clear();
        self.upload_buffer.extend(pixels.chunks_exact(4).flat_map(|px| {
            let a = u16::from(px[3]);
            let over = |c: u8| ((u16::from(c) * a + 127) / 255) as u8;
            [over(px[0]), over(px[1]), over(px[2]), 255]
        }));
        if let Some(traits) = self.renderer.probe(LIST, &["x", "y", "width"]) {
            for (slot, (_, value)) in self.list.iter_mut().zip(traits) {
                *slot = value.as_num();
            }
        }
        let upload_ctx = byroredux_renderer::upload_ctx!(ctx);
        if let Err(error) = ctx.texture_registry.write_rgba_inplace(
            upload_ctx,
            self.texture_handle,
            self.width,
            self.height,
            &self.upload_buffer,
        ) {
            log::error!("start menu: texture upload failed: {error:#}");
        }
        self.texture_handle
    }
}

fn item_name(index: usize) -> String {
    format!("main_item_{index}")
}

/// The next enabled row from `from` in direction `step` (±1), wrapping
/// round the list; `from` itself when no other row is enabled.
fn step_to_enabled(enabled: &[bool], from: usize, step: isize) -> usize {
    let count = enabled.len() as isize;
    let mut row = from as isize;
    for _ in 0..count {
        row = (row + step).rem_euclid(count);
        if enabled[row as usize] {
            return row as usize;
        }
    }
    from
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_title_screen_lists_what_the_original_lists() {
        let labels: Vec<&str> = OPTIONS.iter().map(|option| option.label).collect();
        assert_eq!(labels, ["sContinue", "sNew", "sLoad", "sSettings", "sCrew", "sDownloads", "sQuit"]);
        let need_saves: Vec<&str> = OPTIONS.iter().filter(|o| o.needs_save).map(|o| o.label).collect();
        assert_eq!(need_saves, ["sContinue", "sLoad"], "listed only when a save exists");
    }

    #[test]
    fn arrows_skip_dimmed_options_and_wrap() {
        // New, Settings, Credits (dimmed), Downloads (dimmed), Quit.
        let enabled = [true, true, false, false, true];
        assert_eq!(step_to_enabled(&enabled, 1, 1), 4, "Settings → Quit over the dimmed rows");
        assert_eq!(step_to_enabled(&enabled, 4, 1), 0, "Quit → New, wrapping");
        assert_eq!(step_to_enabled(&enabled, 0, -1), 4, "New → Quit, wrapping up");
        assert_eq!(step_to_enabled(&[false, true, false], 1, 1), 1, "nowhere else to go");
    }
}
