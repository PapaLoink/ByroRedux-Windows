# Oblivion intro and main menu

Status: PARTIAL (2026-11-03). The approved front end has a local implementation;
original-game presentation and interaction parity remain unverified. The scope is in
[Oblivion Front End](../prd/oblivion-front-end.md).

## Native Windows startup regression (3 November 2026)

The reported launcher invocation uses `--esm Oblivion.esm` from the installed
Oblivion Data directory. `Path::parent` returns an empty path for that filename;
Windows rejects canonicalizing it, so front-end initialization failed before
loading the original configuration or presentation resources. Archive discovery
now returns `.` for a bare plugin filename. Synthetic coverage checks both
`Oblivion.esm` and `FalloutNV.esm`, preserving relative subdirectory discovery.

The subsequent `PapyrusPlayerEntity` panic was secondary: `event_loop.exit()`
requests deferred exit, while the renderer still exists and scene startup has
not run. Window dispatch and `about_to_wait` now return immediately when the
event loop is exiting, preventing gameplay scheduling during failed startup.
Original configuration failures now identify the directory or INI path.

This fixes engine startup handling; it does not verify game rules, New Game,
Bethesda save compatibility, DLC or multiplayer. New Vegas front-end behavior
is unsupported by this Oblivion-specific route; its shared filename resolution
is covered synthetically, not against an installed New Vegas game.

Native verification on 3 November 2026 (Rust 1.96.0, MSVC 2022, Vulkan SDK
1.4.363.0, RTX 2060):

- `cargo test --release --locked -p byroredux --bin byroredux`: 3,465 passed,
  0 failed, 80 existing ignores. Local transcript:
  `%TEMP%/byroredux-startup-fix-tests.log`.
- The existing ignored `front_end_original_menu_and_movie_resources_smoke`
  was run explicitly against the installed Oblivion resources: passed. It
  checks authored Options/Credits/Exit targets, visible original menu pixels
  and a decoded startup frame, not original-runtime parity.
- `Start-ByroRedux-Windows.ps1 -Action Build`: native release build passed.
- The rebuilt executable, with the launcher's exact arguments and Data working
  directory, remained running for 15 seconds, uploaded movie frames, advanced
  from the first original startup clip to `2k games.bik`, then handled a normal
  window close with exit code 0 and clean Vulkan teardown. No front-end error
  or panic. `%TEMP%/byroredux-startup-success.stderr.log`.
- A launch with `--esm missing-startup-regression/Oblivion.esm` preserved the
  directory diagnostic and exited without a scheduler/player-resource panic.
  `%TEMP%/byroredux-startup-failure.stderr.log`.

No installed game files, original configuration or saves were changed. Full
intro completion, audio/visual comparison, menu input and subsequent gameplay
were not established by these bounded startup checks.

## Original-content evidence

### Menu font spacing regression (3 November 2026)

The reported menu screenshot showed irregular horizontal gaps and top-aligned
lowercase letters. The FNT reader incorrectly used glyph ascent as horizontal
advance and right-side spacing as vertical offset. The field meanings at
glyph offsets `0x24..0x34` are width, height, left spacing, right spacing and
ascent. Horizontal advance is width plus both spacing values; drawing applies
left spacing and aligns glyphs using ascent and a common baseline. This is an
independent correction based on installed numeric metrics and
[DavidJCobb's format/behavior documentation](https://github.com/DavidJCobb/NorthernUI/blob/master/obse_plugin_example/ReverseEngineered/UI/FontManager.h),
not copied executable code. Oblivion's first baseline uses twice the font's
largest descent. The original font atlas is still loaded at runtime.

Observed source: installed `Oblivion - Misc.bsa`,
`fonts/Kingthings_Regular.fnt` and its named `.tex` atlas. Menu text uses
`menus/options/main_menu.xml` and `menus/prefabs/button_floating.xml`.
For this installed font, corrected advances measure Options = 123,
Credits = 118, Exit = 71 and space = 11 authored pixels. The capital O and
lowercase n share a bottom baseline; p extends below it.

Synthetic regressions cover independent ascent, positive/negative side spacing,
zero advance, blank glyph advance, baseline placement, non-square atlas axes
and font-driven button widths. MenuXml: 66 passed. Installed Oblivion corpus:
5 passed, including the new original-font metric check. New Vegas uses the
shared FNT decoder and has synthetic/profile regression coverage; its original
font appearance and vertical alignment have not been verified locally.

The requested TCRF reference image could not be loaded through web access or
the browser security challenge. This is not a claim of a pixel comparison with
that reference or of complete original-game typography parity.

Live Windows visual check (3 November 2026): the rebuilt release executable
reached the original-resource menu using the installed Oblivion Data directory.
A fresh 1280-wide window capture was compared with the reporter's earlier
1280-wide distorted menu screenshot. `Options`, `Credits`, `Exit` and `Version`
now draw with contiguous letter spacing and a shared baseline rather than the
previous broken gaps and displaced lowercase glyphs; the authored menu
positions and original atlas artwork remain in use. The bottom-left `Version`
is literal text in the installed `main_menu.xml`, not a newly inferred version
number. The map movie continued looping, and the log contained no front-end
failure or panic during this check. This verifies the reported font regression
visually, not pixel parity with the TCRF image or complete menu compatibility.
The running executable was not rebuilt or modified during this check.

All paths below were verified on 2026-11-03 under
`C:/Program Files (x86)/Steam/steamapps/common/Oblivion`.
The installation and personal configuration were read only. Original XML was
inspected locally in `%TEMP%/byroredux-oblivion-front-end-survey`; its contents
are not included in source control. These are observations of installed data,
not claims that the engine already reproduces original runtime behavior.

| Source | Observed value or resource |
|---|---|
| `Oblivion_default.ini`, `SIntroSequence` | `bethesda softworks HD720p.bik`, `2k games.bik`, `game studios.bik`, `Oblivion Legal.bik`, in that order |
| Same file, `SMainMenuMovieIntro` | `Oblivion iv logo.bik` |
| Same file, `SMainMenuMovie` | `Map loop.bik` |
| Same file, `SMainMenuMusic` / `SMainMenuMusicTrack` | `Special/TES4Title.mp3`, verified under `Data/Music` |
| Same file, `SCreditsMenuMovie` | `CreditsMenu.bik` |
| Personal `Oblivion.ini`, `SOblivionIntro` | `OblivionIntro.bik`, verified under `Data/Video` |
| `Oblivion - Misc.bsa` | `menus/options/main_menu.xml`, root `MainMenu` |
| Authored XML tile names | `main_continue`, `main_new`, `main_load`, `main_options`, `main_credits`, `main_exit`, `press_start`, `button_layout`, `version` |
| XML artwork references | `Menus/Loading/loading_background.dds`, `tes_oblivion_logo_bink.dds`, `tes_oblivion_logo_final.dds` |
| `Oblivion - Textures - Compressed.bsa` | Original logo/background DDS resources verified in the `textures/menus` and smaller authored texture sets |

The source XML also contains a second file named `copy of main_menu.xml`;
that name is not a reason to select it over `main_menu.xml`.
The HUD's `hud_main_menu.xml` is a different document and must not be used as
the title screen.

### Decoder feasibility

The installed host has `ffmpeg.exe` and `ffprobe.exe`. Metadata probes observed:

| Original video | Video | Audio | Dimensions | Rate | Duration seconds |
|---|---|---|---|---|---|
| `Oblivion iv logo.bik` | `binkvideo` | none reported | 1280 x 720 | 2997/100 | 13.947281 |
| `Map loop.bik` | `binkvideo` | none reported | 1280 x 720 | 2997/100 | 25.025025 |
| `OblivionIntro.bik` | `binkvideo` | `binkaudio_dct`, stereo, 44100 Hz | 1280 x 720 | 2997/100 | 115.482149 |

A one-second decode of the installed Bethesda startup clip to FFmpeg's null
output succeeded. Full-length audiovisual playback, synchronization, looping,
skip timing and original input behavior remain unverified. No proprietary Bink
DLL, executable code or copied video is a proposed engine dependency.

## Existing seams and gaps

| Seam | Evidence and intended use |
|---|---|
| MenuXml rendering | `crates/menuxml/src/menu.rs`: load the original document with `MenuProfile::oblivion`, fonts and DDS assets; preserve authored layout |
| Authored menu interaction | MenuXml currently has no pointer hit-test/input dispatch API. Add evaluated tile hit testing and route the existing input contract, rather than draw substitute buttons |
| Archive resolution | Reuse the provider and font/texture loading patterns in `byroredux/src/hud.rs`; avoid another BSA reader |
| Movie decoding | No existing Bink player. Proposed bounded worker streams RGBA frames and audio from the installed host FFmpeg; do not decode whole movies into RAM or transcode assets to disk |
| Presentation | Reuse the renderer's overlay upload/presentation path; front-end rendering must not require a loaded world |
| Audio | Reuse `byroredux-audio` for title music and decoded movie audio; one playback clock controls presentation |
| World startup | `App::resumed` currently calls `setup_scene` immediately. Defer scene startup and new-game lifecycle effects until a supported menu action commits |
| Save/load | Reuse `byroredux-save` slot listing and the live load executor. Engine saves and original Bethesda saves must be identified distinctly |
| New Game | The Oblivion profile has no authored `new_game_worldspace/grid`; the existing CLI emits a missing-target diagnostic. A debug sample interior is not a canonical new game |

## Proposed lifecycle

`Startup logos -> title animation -> main menu -> supported action`.
Movie completion and skipping advance through the same transition function.
The map loops behind the original menu and title music uses the existing audio
service. Credits play the original credits movie and return to the menu.
Decoder children, queues and audio handles are cancelled/drained on skip, close
and transitions. Simulation remains stopped until a gameplay action commits.
Direct cell/grid, benchmark, headless and Studio launches retain their current
routes; the canonical Windows player entry selects the front end explicitly.

Bound input paths, decoded dimensions, queued frame/audio bytes, child lifetime
and stderr. Close, resize and focus events must remain responsive. The
previous shader-preparation work does not make full Vulkan initialization
asynchronous, so startup GPU initialization must be measured separately.

## Verification still required before implementation decisions are finalized

- Original skip keys, press-start behavior, menu tile IDs, visibility/enabled
  traits and focus/highlight behavior: inspect authored traits and compare with
  a lawfully observed original-game session.
- New Game: identify the original starting record, placement, quest startup and
  character-creation handoff; a known prison cell alone does not prove this.
- Original save-format support is a separate goal; show only validated engine
  saves until interoperability is actually implemented and tested.
- Validate the original menu at native aspect ratios and decode full clips
  with audio before claiming intro/main-menu parity.
