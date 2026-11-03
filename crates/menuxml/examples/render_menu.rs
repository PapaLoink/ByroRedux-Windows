//! Render any vanilla FO3/FNV MenuXml menu to a PNG, or list the menus an
//! archive carries — for judging a menu the engine has not drawn yet.
//!
//! ```text
//! cargo run -p byroredux-menuxml --example render_menu -- <Data dir> list
//! cargo run -p byroredux-menuxml --example render_menu -- <Data dir> cat <menus\...xml>
//! cargo run -p byroredux-menuxml --example render_menu -- <Data dir> textures <text>
//! cargo run -p byroredux-menuxml --example render_menu -- <Data dir> texture <path> <out.png>
//! cargo run -p byroredux-menuxml --example render_menu -- <Data dir> font <slot>
//! cargo run -p byroredux-menuxml --example render_menu -- <Data dir> <menus\...xml> <out.png> [tile.trait=value | item=TEXT ...]
//! ```
//!
//! Reads `Fallout - Misc.bsa` (menus) and `Fallout - Textures.bsa` (+
//! `Textures2.bsa`, fonts) from the data folder. The profile is FNV's.
use std::path::Path;

use byroredux_bsa::BsaArchive;
use byroredux_menuxml::menu::MenuAssets;
use byroredux_menuxml::{MenuProfile, MenuRenderer, ScreenTraits};

/// FO3/FNV menus are authored on a 960-unit-tall screen (`pause_background`
/// centres itself at `y` 480); 16:9 makes it 1707 wide.
const WIDTH: u32 = 1707;
const HEIGHT: u32 = 960;

struct Assets {
    misc: BsaArchive,
    textures: Vec<BsaArchive>,
    profile: MenuProfile,
}

impl Assets {
    fn texture_bytes(&self, path: &str) -> Option<Vec<u8>> {
        self.textures.iter().find_map(|archive| archive.extract(path).ok())
    }
}

impl MenuAssets for Assets {
    fn menu_xml(&self, path: &str) -> Option<Vec<u8>> {
        self.misc.extract(path).ok()
    }
    fn texture(&self, path: &str) -> Option<Vec<u8>> {
        self.texture_bytes(path)
    }
    fn font(&self, index: u8) -> Option<Vec<u8>> {
        self.texture_bytes(self.profile.font_paths.get(index as usize - 1)?)
    }
    fn font_texture(&self, path: &str) -> Option<Vec<u8>> {
        self.texture_bytes(path).or_else(|| self.misc.extract(path).ok())
    }
}

fn main() {
    env_logger::init();
    let mut args = std::env::args().skip(1);
    let data = args.next().expect("usage: render_menu <Data dir> list | <menu> <out.png> [tile.trait=value ...]");
    let data = Path::new(&data);
    let misc = BsaArchive::open(data.join("Fallout - Misc.bsa")).expect("Fallout - Misc.bsa");
    let what = args.next().expect("a menu path or `list`");
    if what == "list" {
        for file in misc.list_files() {
            if file.starts_with("menus") {
                println!("{file}");
            }
        }
        return;
    }
    if what == "cat" {
        let path = args.next().expect("a menu path");
        let bytes = misc.extract(&path).expect("menu in Fallout - Misc.bsa");
        print!("{}", String::from_utf8_lossy(&bytes));
        return;
    }
    let textures = ["Fallout - Textures.bsa", "Fallout - Textures2.bsa"]
        .iter()
        .filter_map(|name| BsaArchive::open(data.join(name)).ok())
        .collect();
    let assets = Assets { misc, textures, profile: MenuProfile::fallout_nv() };
    if what == "textures" {
        // `textures <text>`: every texture path containing the text.
        let needle = args.next().unwrap_or_default().to_lowercase();
        for archive in &assets.textures {
            for file in archive.list_files() {
                if file.to_lowercase().contains(&needle) {
                    println!("{file}");
                }
            }
        }
        return;
    }
    if what == "texture" {
        // `texture <archive path> <out.png>`: one decoded texture.
        let path = args.next().expect("a texture path");
        let out = args.next().expect("an output .png path");
        let bytes = assets.texture_bytes(&path).expect("texture in the texture archives");
        let image = byroredux_menuxml::tex::Rgba8::decode_dds(&bytes).expect("a decodable DDS");
        write_png(Path::new(&out), image.width, image.height, &image.pixels).expect("png written");
        println!("{path}: {}x{} -> {out}", image.width, image.height);
        return;
    }
    if what == "font" {
        // `font <slot>`: the header, a few glyph records and the atlas
        // candidates' sizes, for checking the `.fnt` layout on this corpus.
        let slot: usize = args.next().expect("a 1-based font slot").parse().expect("a number");
        let path = assets.profile.font_paths[slot - 1];
        let fnt = assets.texture_bytes(path).expect("the .fnt");
        let f = |at: usize| f32::from_le_bytes(fnt[at..at + 4].try_into().unwrap());
        let name = String::from_utf8_lossy(&fnt[12..296]).split('\0').next().unwrap_or("").to_string();
        println!("{path}: {} bytes, size {} version {} textures {} atlas '{name}'", fnt.len(), f(0), f(4).to_bits(), f(8).to_bits());
        for ch in [b'A', b'N', b'E', b'W', b' ', b'e', b'g', b't', b'i', b'S'] {
            let base = 296 + ch as usize * 56;
            let record: Vec<f32> = (0..14).map(|i| f(base + i * 4)).collect();
            println!("  {:?}: {record:?}", ch as char);
        }
        for candidate in (assets.profile.font_atlas)(&name) {
            match assets.texture_bytes(&candidate) {
                Some(bytes) => {
                    println!("  atlas {candidate}: {} bytes, header {:?}", bytes.len(), &bytes[..16.min(bytes.len())]);
                    match byroredux_menuxml::font::Font::parse(&fnt, &bytes) {
                        Ok(font) => {
                            println!(
                                "    parsed: atlas {}x{}, 'A' {:?}",
                                font.atlas.width,
                                font.atlas.height,
                                font.glyph(b'A')
                            );
                            // `font <slot> <out.png>` also writes the atlas.
                            if let Some(png) = args.next() {
                                write_png(Path::new(&png), font.atlas.width, font.atlas.height, &font.atlas.pixels)
                                    .expect("atlas png written");
                                println!("    atlas written to {png}");
                            }
                        }
                        Err(error) => println!("    parse failed: {error}"),
                    }
                }
                None => println!("  atlas {candidate}: missing"),
            }
        }
        return;
    }
    let out = args.next().expect("an output .png path");
    let mut renderer = MenuRenderer::load_with_profile(
        &assets,
        &what,
        ScreenTraits::new(WIDTH as f32, HEIGHT as f32),
        MenuProfile::fallout_nv(),
    )
    .expect("menu loads");
    let mut items = 0;
    let mut probes = Vec::new();
    for assignment in args {
        // `?tile` prints the tile's evaluated layout traits after the render.
        if let Some(tile) = assignment.strip_prefix('?') {
            probes.push(tile.to_string());
            continue;
        }
        // `item=TEXT` adds a StartMenu-style list item (what the engine code
        // did): `lb_item_template` under `main_container`, its `_y`,
        // `listindex` and `string` set.
        if let Some(text) = assignment.strip_prefix("item=") {
            let name = format!("main_item_{items}");
            renderer
                .instantiate_template("lb_item_template", "main_container", &name)
                .expect("lb_item_template");
            renderer.set_override_str(&name, "string", text);
            renderer.set_override(&name, "_y", items as f32 * 48.0);
            renderer.set_override(&name, "listindex", items as f32);
            items += 1;
            renderer.set_override("main_container", "_number_of_visible_items", items as f32);
            continue;
        }
        let (target, value) = assignment.split_once('=').expect("tile.trait=value");
        let (tile, trait_name) = target.split_once('.').expect("tile.trait=value");
        renderer.set_override(tile, trait_name, value.parse().expect("a number"));
    }
    let pixels = renderer.render_frame(&assets).to_vec();
    write_png(Path::new(&out), WIDTH, HEIGHT, &pixels).expect("png written");
    println!("wrote {out}");
    for tile in probes {
        match renderer.probe(&tile, &["visible", "x", "y", "width", "height", "alpha", "depth", "zoom", "filewidth", "fileheight", "string", "font"]) {
            Some(traits) => println!("{tile}: {traits:?}"),
            None => println!("{tile}: no such tile"),
        }
    }
}

/// A minimal stored-deflate PNG (the crate carries no image dependency).
fn write_png(path: &Path, width: u32, height: u32, rgba: &[u8]) -> std::io::Result<()> {
    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &byte in data {
            crc ^= u32::from(byte);
            for _ in 0..8 {
                crc = if crc & 1 != 0 { 0xEDB8_8320 ^ (crc >> 1) } else { crc >> 1 };
            }
        }
        crc ^ 0xFFFF_FFFF
    }
    let mut raw = Vec::with_capacity(rgba.len() + height as usize);
    for row in rgba.chunks(width as usize * 4) {
        raw.push(0);
        raw.extend_from_slice(row);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in &raw {
        a = (a + u32::from(byte)) % 65_521;
        b = (b + a) % 65_521;
    }
    let mut zlib = vec![0x78, 0x01];
    let mut blocks = raw.chunks(65_535).peekable();
    while let Some(block) = blocks.next() {
        zlib.push(u8::from(blocks.peek().is_none()));
        zlib.extend_from_slice(&(block.len() as u16).to_le_bytes());
        zlib.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    for (tag, body) in [(b"IHDR", &ihdr[..]), (b"IDAT", &zlib[..]), (b"IEND", &[][..])] {
        png.extend_from_slice(&(body.len() as u32).to_be_bytes());
        let mut chunk = tag.to_vec();
        chunk.extend_from_slice(body);
        png.extend_from_slice(&chunk);
        png.extend_from_slice(&crc32(&chunk).to_be_bytes());
    }
    std::fs::write(path, png)
}
