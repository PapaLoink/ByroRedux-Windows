# Oblivion + Fallout: New Vegas — Full Engine-Restoration Parity Checklist

**Last updated:** 2026-11-04
**Tracking PR:** [ByroRedux-Windows #1](https://github.com/PapaLoink/ByroRedux-Windows/pull/1)
**Scope:** Original 2006 **Oblivion** and 2010 **Fallout: New Vegas** on Windows, using legitimate original game installations, in-scope vanilla DLC and the original games' *observable* behaviours. This is a **restoration backlog, not a claim that items are missing or passing**.

## Goal and honest definition of "exactly like the original engine"

The original Gamebryo/Bethesda/Obsidian engine implementations are proprietary. An independent Rust/Vulkan/Rapier engine cannot be established as byte-for-byte identical to their internal code, scheduler, memory layout, executable hooks, Havok middleware or historical GPU drivers. The actionable target is **externally demonstrated behavioural and content parity** for a frozen, documented original game version/configuration: the same content loads, the same inputs cause the same game rules and player-visible outcomes, the same scripts run in the correct order, and saved sessions continue equivalently. Differences (including original bugs, pixel-level rendering and non-portable executable mod hooks) must be identified rather than silently represented as parity.

**Nothing in this checklist is automatically "done" because source code exists, a parser reports 100%, a test passes on synthetic content, or a prior roadmap says closed.** A checkbox closes only with a recorded original-game oracle **and** a passing Redux integration regression, or with a documented user-approved deviation. Some entries are coverage/inventory gates rather than known broken features; do not interpret them as all currently absent.

**Verification classes:** [ ] open / not yet proven; [x] explicitly demonstrated with required evidence. All tasks start open.
**Suggested priority:** P0 core quest/player-state blockers; P1 systemic mechanics/UI; P2 fidelity and breadth; P3 optimisations/optional legacy bugs. Work can proceed in parallel, but an unverified P0 bars the relevant game's full-parity claim.

### Test prerequisites and universal completion criteria

- Use pinned original executable version, distribution, INI, hardware/runtime assumptions, language, clean game files, controller profile and all installed DLC; record hashes without distributing copyrighted content.
- Store original-game *observations*, not redistributed executable code, assets or derived proprietary binary blobs. Provide repeatable keystrokes, starting saves, script/event/actor-value logs where obtainable, and visual/audio captures made from a lawful installation.
- Run at least one **headless deterministic test**, one **live 30+ minute session** and one **save/restart/re-enter** test per subsystem, as applicable; replay with fresh and dirty/new-game state.
- For every game record type, ObScript opcode, menu, quest, animation and audio asset, have an inventory row: `supported + tested`, `supported + unverified`, `partial`, `unsupported`, `not applicable`, or `intentional divergence`. No unknown silently counted as success.
- Maintain one source-backed comparison trace per game scenario; measure numeric tolerances rather than assuming every source-game calculation is exact or deterministic.
- Use distinct `Oblivion` and `FalloutNV` profiles; **do not share a behaviour merely because the record signatures resemble each other**.

## A. Baselines, compatibility census and regression harness — P0

- [ ] **PAR-001** Pin Oblivion executable build, official/retail/Steam differences, patches and DLC load order; capture original configuration and hashes.
- [ ] **PAR-002** Pin New Vegas executable build and all DLC (including Dead Money, Honest Hearts, Old World Blues, Lonesome Road, Gun Runners' Arsenal and Courier's Stash), content order, patches and game configuration.
- [ ] **PAR-003** Maintain clean legal installs of both games and a documented non-network-required offline test harness.
- [ ] **PAR-004** Build side-by-side golden scenarios for title screen, New Game, UI, exploration, quests, combat, AI, inventory and save/load.
- [ ] **PAR-005** Capture all loaded record signatures/subrecord variants per master and DLC, with zero silent parser drops and explicit unknown accounting.
- [ ] **PAR-006** Generate inventories for every vanilla quest, script, CTDA function, gameplay command, UI page, animation family, sound category, spawn/placed reference and gameplay setting.
- [ ] **PAR-007** Pair each inventory item with executed integration gates and coverage reports; fail on new unsupported runtime paths.
- [ ] **PAR-008** Compare original/Redux actor values, quest stages, variables, FormIDs, positions, game time, inventory, package states and menu events at matching checkpoints.
- [ ] **PAR-009** Automate comparative screenshot/video/audio captures with matched resolution/aspect, camera position, weather, time, FOV, HUD and settings.
- [ ] **PAR-010** Define tolerances, flaky-test policy and deterministic seed/time controls; distinguish source-game nondeterminism from Redux regressions.
- [ ] **PAR-011** Create repeatable full-game vanilla quest smoke suites per game, with documented progress checkpoints and blocked-path owners.
- [ ] **PAR-012** Gate unsupported original-game behaviour explicitly; no placeholders that pretend to succeed.

## B. Plugins, archives, data formats, FormIDs and mods — P0/P1

- [ ] **PAR-013** Verify original load-order and master dependency rules, missing masters, circular/invalid plugins, ESP/ESM flags and override precedence for each game.
- [ ] **PAR-014** Verify FormID resolution/remapping, persistent refs, duplicated IDs, dynamically created refs and load-order changes across saves.
- [ ] **PAR-015** Resolve loose files versus BSA in the original's search order, including archive naming, asset substitution and collisions.
- [ ] **PAR-016** Inventory all TES4/FO3/FNV record and subrecord formats (including compressed, versioned and extended records); round-trip meaningful gameplay data.
- [ ] **PAR-017** Validate all cell/worldspace group hierarchies, overridden/deleted/disabled refs, linked refs, enable-parent chains and persistent/temporary ref identities.
- [ ] **PAR-018** Match leveled item/actor lists, chance-none, nested lists, player-level scaling, list combination rules and spawned results.
- [ ] **PAR-019** Verify FormID-keyed GMST/GLOB/FLST values, script-editable global state and game-specific default values.
- [ ] **PAR-020** Verify equipment, race, class, factions, perk, quest, dialogue and encounter data reach live systems instead of merely parsing.
- [ ] **PAR-021** Verify the original NIF/BSA/texture/FaceGen/font/audio/animation corpus under each DLC and at least one representative modded archive.
- [ ] **PAR-022** Build a real-world conformance corpus for Oblivion ESP mods and New Vegas ESP/ESM mods; publish per-mechanism pass/partial/fail, without pretending executable-hook DLLs are compatible.
- [ ] **PAR-023** Verify game-data language/localisation and string-table handling for supported installations.
- [ ] **PAR-024** Validate console and INI settings that the original games or content scripts query; classify unsupported graphics/launcher-only settings.

## C. Cells, worldspaces, traversal, LOD and persistent world — P0/P1

- [ ] **PAR-025** Match interior cell loading, fog, ambient light, ownership, encounter zones, water, room geometry and linked doors.
- [ ] **PAR-026** Match exterior grid/tile loading, worldspace parenting, land heights, vertex colours, path grids/NAVM data, region lists and distant cells.
- [ ] **PAR-027** Test XTEL door target, arrival position/orientation, lock status, animation/sound, transition effects and post-load script event order.
- [ ] **PAR-028** Test Oblivion's worldspace/Oblivion Gate portals, Shivering Isles spaces and New Vegas DLC worldspaces.
- [ ] **PAR-029** Preserve actor position, package progress, alive/dead state, inventory, ownership, scripts and script variables across ordinary tile eviction/re-entry.
- [ ] **PAR-030** Verify persistent references across worldspace changes and save/restart without respawn duplication.
- [ ] **PAR-031** Persist dropped items, moved statics, picked plants, looted containers, unlocked doors and quest-altered references across unloaded cells.
- [ ] **PAR-032** Implement original cell-reset, respawn and cleanup semantics, timeouts and protected/quest exceptions.
- [ ] **PAR-033** Match land/quad LOD, meshes, buildings, trees, grass, water/land transitions and distant object placement in each title.
- [ ] **PAR-034** Verify collision/raycast and prompt selection at door thresholds, stairs, slopes, cliffs and interiors/exteriors.
- [ ] **PAR-035** Implement game-supported fast travel, location discovery/markers, time advance and refusal rules.
- [ ] **PAR-036** Run multi-hour cell-crossing soak scenarios covering NPC travel, combat, quest transitions and deferred saves.

## D. Game loop, time, input, player movement and camera — P0/P1

- [ ] **PAR-037** Match input priority and edge semantics for gameplay versus HUD, menus, dialogue, consoles, cinematic states and paused simulations.
- [ ] **PAR-038** Match keyboard/mouse mapping, sensitivity, invert-Y, rebinding, held actions, key repeat, controller mappings and original defaults.
- [ ] **PAR-039** Match first-/third-person cameras, FOV, zoom, head tracking, camera collisions, weapon view, perspective switch and sitting/sleeping camera behaviour.
- [ ] **PAR-040** Match crouch/sneak, walking/running, jumping, swim, underwater, falling, fatigue/AP drain and movement speeds by game rules.
- [ ] **PAR-041** Match encumbrance and item-carry rules for movement, sprint/fast travel or other original restrictions.
- [ ] **PAR-042** Implement game-time scale, calendar, weekday, time-of-day, waiting and sleeping with script/time-step consistency.
- [ ] **PAR-043** Match pausing and menu-time advancement: original inventory, VATS, dialogue, pip-boy, journal and console.
- [ ] **PAR-044** Verify physics/game/script timing independence from render FPS, including legacy tick cadence and event ordering.
- [ ] **PAR-045** Restore correct gamepad and mouse-UI cursor ownership through alt-tab, focus loss, resizing and DPI changes.

## E. Physics, collisions, interaction and object simulation — P1

- [ ] **PAR-046** Compare NIF Havok collision translation with original static/compound/phantom collision and collision filtering.
- [ ] **PAR-047** Implement/validate fully dynamic dropped weapons, armour, clutter and movable furniture with appropriate mass, friction and impact response.
- [ ] **PAR-048** Match player/NPC/creature capsule or ragdoll contact, step-up, gravity, knockdown, ragdoll transition and wake/recovery.
- [ ] **PAR-049** Match weapon traces, projectile sweeps, collision ownership, line-of-sight blockers and trigger volume enter/leave.
- [ ] **PAR-050** Verify ownership and activation of doors, containers, furniture, books, levers, altars, flora, traps and scripted activators.
- [ ] **PAR-051** Implement grab/drag/drop/throw/manipulate interactions where supported, with original script hooks.
- [ ] **PAR-052** Implement original per-game lockpicking, security/skill/perk/lock-level gates, key ownership and failure/breakage behaviours.
- [ ] **PAR-053** Implement New Vegas terminal hacking, lockout, skill gates and computer/terminal interactions.
- [ ] **PAR-054** Verify water buoyancy, swim depth, current, surface contact, physics splash and underwater sound/movement.
- [ ] **PAR-055** Verify ragdoll bone mapping, animation transitions and collision against authored joint constraints.

## F. Actors, character visuals, animations and equipment — P1

- [ ] **PAR-056** Verify race/sex/age-specific skeletons, morphs, FaceGen, facial expressions, body parts and hair for all vanilla actor races.
- [ ] **PAR-057** Match spawned actors' initial inventories/outfits/equipped meshes, creature attacks, scale and actor ownership.
- [ ] **PAR-058** Match first-person arms/weapons, third-person gear layering, weapon/sheath placement, armour/clothing body-part hides and dual models.
- [ ] **PAR-059** Implement locomotion state graphs: idle, walk, run, turn, crouch, strafe, swim, fall, sit, sleep, mount and scripted idles.
- [ ] **PAR-060** Implement attack/block/parry/dodge/aim/reload/hit/stagger/death animation families as applicable per game.
- [ ] **PAR-061** Match animation blending, root motion, foot placement, ragdoll handoffs, per-clip event timestamps and emitter attachment.
- [ ] **PAR-062** Match NPC lip sync, FaceGen dialogue expressions, looking-at, gestures and camera-facing during conversations.
- [ ] **PAR-063** Verify animated NIF controllers, morphs, UV/alpha animations, particle emitters, blood decals and effect shaders.
- [ ] **PAR-064** Compare all vanilla actor/creature animation families and DLC additions; record missing runtime clips.

## G. Audio, music and voices — P1/P2

- [ ] **PAR-065** Match sound-record attenuation, priority, spatialisation, reverb, listener transitions and interior/exterior ambience.
- [ ] **PAR-066** Match original dialogue voice selection, FNV/Oblivion voice file naming, line timing, lip sync and subtitle display.
- [ ] **PAR-067** Implement region ambient, weather/thunder/wind, dungeon/settlement ambience and incidental sound schedules.
- [ ] **PAR-068** Match music playlist/folder variant selection, combat/exploration transitions, silence policy and crossfades.
- [ ] **PAR-069** Match per-material footsteps for player/NPCs, weapon/armour/ragdoll impacts, footsteps on water/grass/stone.
- [ ] **PAR-070** Match UI button, inventory, equip, spell, Pip-Boy and VATS sounds with original menu timing.
- [ ] **PAR-071** Verify sound effects issued by scripts including loop, stop, attach to source, volume change and priority.
- [ ] **PAR-072** Compare fade, underwater filter, pause sound, music restart and game-window focus changes.
- [ ] **PAR-073** Gate full voice/music decoding with legal original files and end-to-end audible evidence.

## H. Rendering, lighting, weather and original visual appearance — P1/P2

- [ ] **PAR-074** Establish separate targets for **original-look parity** and **optional modern-lighting quality**; neither should imply the other.
- [ ] **PAR-075** Match geometry placement, scale, camera projection, culling, normal/tangent orientation, UVs and skinned rendering.
- [ ] **PAR-076** Match classic NIF material alpha blending/testing, alpha sorting, environment/cubemap mapping, glow, normal/specular maps and particle colours.
- [ ] **PAR-077** Match original Oblivion daylight/interior lights, bloom/HDR, exposure, night visibility and fog curves under a legacy profile.
- [ ] **PAR-078** Match New Vegas's exterior colour grading, weather/sky, sunlight, dust/fog, worldspace-specific ambient lighting and desert distance.
- [ ] **PAR-079** Match water surface/refraction/reflection/underwater visuals, spray, waves and shore transitions.
- [ ] **PAR-080** Match original shadows/decals, dynamic light attachment, sun movement and indoor light behaviour; verify source-game settings influence.
- [ ] **PAR-081** Inventory and reproduce game-specific image effects: magic shaders, night-eye/night-vision, VATS tint, hit effects, radiation/poison, weather and screen fades.
- [ ] **PAR-082** Run direct-only, shadow-visibility, indirect-only, volumetric-only and final captures to isolate visible grain and temporal instability.
- [ ] **PAR-083** Compare native TAA and FSR upscaling; measure SVGF shadow/GI residuals, disocclusion, SSAO noise and shimmer at parked/moving cameras.
- [ ] **PAR-084** Stabilise shadow penumbrae and GI energy against reference scenes under a fixed RT tier; avoid deleting intentional dither without evidence.
- [ ] **PAR-085** Measure memory/VRAM, texture streaming, material shader variants, open-world traversal and load hitches on reference hardware.
- [ ] **PAR-086** Verify all weather/region variants, night/day changes and effects when the player travels through the two games.

## I. ObScript, conditions, globals and event lifecycle — P0

- [ ] **PAR-087** Enumerate **every compiled SCDA expression/statement/operator** and confirm interpretation for each game dialect, including numeric negation and short-circuit semantics.
- [ ] **PAR-088** Enumerate every original vanilla Oblivion and New Vegas script command/function and its call-site count; classify parser, evaluator, host action, unsupported and untested.
- [ ] **PAR-089** Match local/quest/reference/global variable identity and lifetime, default initialization, reference variables and save persistence.
- [ ] **PAR-090** Match GameMode/MenuMode tick cadence, OnLoad, OnActivate, OnAdd, OnEquip, OnDrop, OnHit, OnDeath, OnTrigger/enter/leave and result-script ordering.
- [ ] **PAR-091** Match all condition functions and exact AND/OR precedence, actor/quest/reference context, null handling and numeric comparisons.
- [ ] **PAR-092** Wire source-game actor-value script codes to real actor data (Oblivion's hard-coded IDs require an AVIF-free resolver).
- [ ] **PAR-093** Match gameplay side-effects of script commands: movement, activating refs, teleport, player controls, AI packages, combat, factions, inventory, weather and audio.
- [ ] **PAR-094** Match scripted magic/chemical effects, scripted event hooks and persistent effect timers.
- [ ] **PAR-095** Preserve script stack/variable/event state through save/load, unload/reload and mod load-order updates to the extent original-content semantics demand.
- [ ] **PAR-096** Make unsupported calls/error outcomes visible with the failing script/FormID/line or opcode; never fabricate successful effects.
- [ ] **PAR-097** Run complete original script corpus tests from each master and DLC, including famous failure-prone quest and repeatable event scripts.
- [ ] **PAR-098** Test console-invoked equivalents (`player.additem`, `setstage`, `getav`, `setav`, `coc`, etc.) against their original semantics.
- [ ] **PAR-099** Verify active effect expiry, wait/rest/time-skip and cell unloading do not cause repeated or skipped script events.

## J. Quest state, dialogue and cutscenes — P0

- [ ] **PAR-100** Match QUST start/restart/completion/failure, stages, result scripts, objectives, journal text, markers and displayed notifications.
- [ ] **PAR-101** Match INFO/DIAL topic eligibility, priority, greeting triggers, response selection, follow-ups and speaker conditions.
- [ ] **PAR-102** Match dialogue-result effects, topic-discovery/unlock, barter/training branches and quest-stage side effects.
- [ ] **PAR-103** Implement original conversation camera, looking/gestures, subtitles, audio/lip sync, timed responses and menu input.
- [ ] **PAR-104** Match essential/protected/disabled NPC and scripted death or resurrection impact on quests.
- [ ] **PAR-105** Match in-world cinematic controls, forced movement, fades, camera transitions and original movie playback.
- [ ] **PAR-106** Match scripted quest objects, activation blockers, re-entry paths and game-specific story props.
- [ ] **PAR-107** Verify **all vanilla quest paths**, including alternate choices, cancelled/failed paths, branch reversals, restarts and DLC quests; never equate one vertical slice to full-game completion.
- [ ] **PAR-108** Snapshot quest/dialogue state before and after loads, fast travel, time skip, cell resets and DLC transitions.
- [ ] **PAR-109** Gate one full primary-story route and multiple side-quest paths for each title before declaring playable parity.

## K. AI packages, navigation, perception and society — P0/P1

- [ ] **PAR-110** Match per-game AI package priority, conditional filtering, schedules, fallback/re-evaluation and package override from scripts.
- [ ] **PAR-111** Implement all used classic PACK procedures: Sandbox, Wander, Travel, Follow, Escort, Guard, Patrol, Eat, Sleep, Find, UseWeapon, UseItem, Flee, Dialogue, CastMagic and special targets with authored semantics.
- [ ] **PAR-112** Match Oblivion path grids and New Vegas navmesh navigation, blocked-route recovery, doors, obstacles, crowded paths and path re-planning.
- [ ] **PAR-113** Match perception: light, sound, sneaking, line-of-sight, actor values, awareness, detection meter and aggro triggers.
- [ ] **PAR-114** Match combat decision-making: hostile warnings, ranged vs melee selection, cover, fallback, pursuit, fleeing, assistance and faction targeting.
- [ ] **PAR-115** Match sleeping/eating/sitting/using furniture and corresponding idle/animation/sound constraints.
- [ ] **PAR-116** Match companion following/commands, wait/follow distances, teleport catch-up, equipment and combat assistance.
- [ ] **PAR-117** Match faction relationships, detection of trespassing/theft/assault/murder, witnessed crime and dialogue changes where game-specific.
- [ ] **PAR-118** Preserve AI goal progress across cell streaming, fast travel, save/load, script interruption and scheduler ticks.
- [ ] **PAR-119** Verify deterministic daily NPC routes in named vanilla settlements and at least one dungeon/faction encounter in each title.
- [ ] **PAR-120** Validate creature-specific locomotion/combat/activation, including special encounter scripts and DLC creatures.

## L. Core RPG economy, inventory and actor state — P0/P1

- [ ] **PAR-121** Match every original item category, weight, value, condition, inventory instance, container ownership, stealing flags and auto-unequip rules.
- [ ] **PAR-122** Match loot/pickup/drop/transfer, partial stack operations, drag/drop prompts and corpse/inventory menu behaviour.
- [ ] **PAR-123** Match buy/sell/barter, merchant gold/restock, prices/skill modifiers, theft/fences, training and speech/disposition effects.
- [ ] **PAR-124** Implement equipment condition/durability, repair rules, inventory filters, weight caps and weapon/armour effects per game.
- [ ] **PAR-125** Match actor health/magicka/fatigue/action points and relevant damage, regeneration, incapacitation and death transitions.
- [ ] **PAR-126** Match long-duration magic/chemical effects, disease/affliction, addiction, poisons, cures and time advancement.
- [ ] **PAR-127** Match racial/class/creature/faction starting stats, actor scaling, authored skills and NPC/creature auto-calculation.
- [ ] **PAR-128** Match respawn and loot generation, encounter tables and any per-game RNG/level-scaled outcomes.
- [ ] **PAR-129** Match crime reporting, factions, reputation and hostility as separate per-game systems.
- [ ] **PAR-130** Gate a full economy loop (loot → equip → repair → barter → steal/fine → save/reload) in each title.

## M. Oblivion-only restoration — P0/P1

- [ ] **OBL-001** Wire all eight attributes, 21 skills, actor values, birthsign/race/class modifiers and accurate initial Player/NPC/creature stat derivation.
- [ ] **OBL-002** Implement skill-use and major/minor skill levelling, level-up via sleep, attribute multipliers, trainers and skill caps with original results.
- [ ] **OBL-003** Match Health/Magicka/Fatigue calculation, regeneration, spell-cost effectiveness, fatigue influence and vampire/disease effects.
- [ ] **OBL-004** Implement original New Game lifecycle: opening cinematic, name/race/sex/appearance, prison escape tutorial, birthsign/class choices and sewer-exit review.
- [ ] **OBL-005** Implement schools, known spells, spell effects/archetypes, cast delivery (self/touch/target), magicka consumption, resistance, reflect/absorb and active duration.
- [ ] **OBL-006** Implement spellmaking, enchanting, alchemy, ingredient discovery, apparatus/skills, poisons/potions and soul gems/trapping.
- [ ] **OBL-007** Match melee/block/ranged/hand-to-hand combat, weapon types, skill-based damage and fatigue, shield/armour/equipment degradation.
- [ ] **OBL-008** Match speechcraft persuasion wheel, disposition, NPC reactions, fame/infamy, factions, guild advancement and expelled-state conditions.
- [ ] **OBL-009** Match witnessed-crime calculations, guards/arrest/bribe/jail/fines/confiscation and stealth/pickpocket/trespass consequences.
- [ ] **OBL-010** Implement Oblivion Gates, Sigil Stones, associated persistent gate state, scripted opening/closing and main-quest mechanics.
- [ ] **OBL-011** Match Doomstones, shrines/altars, blessings, vampirism, diseases, sleep/wait and quest-specific scene devices.
- [ ] **OBL-012** Match mounted horses, stables, horse ownership, mounted camera and collision and horse-related scripting.
- [ ] **OBL-013** Match Cyrodiil local/world maps, discovered destinations, map markers, journal active/completed/failed quest entries and fast travel.
- [ ] **OBL-014** Match all Oblivion XML menus from real archives (title, pause, HUD, inventory, stats, magic, map, options, dialogue, books, barter, spell creation, lockpicking, loading screens and character creation).
- [ ] **OBL-015** Complete the original Options actions; unsupported controls must be visibly classified, not presented as functional.
- [ ] **OBL-016** Run original Knights of the Nine, Shivering Isles and other installed official DLC story/mechanics compatibility gates.
- [ ] **OBL-017** Match original level-scaled NPC/creature and loot behaviours, encounter triggers, NPC schedules and magic AI in the base game and DLC.
- [ ] **OBL-018** Validate a complete original main-quest route including prison/new-game, Kvatch, Oblivion Gates and ending.
- [ ] **OBL-019** Validate full Knights/guild/faction quest branches, Arena, Dark Brotherhood, Thieves Guild, Mages/Fighters Guild and alternate failure states.

## N. New Vegas-only restoration — P0/P1

- [ ] **FNV-001** Match Courier creation, Doc Mitchell intro, sex/race/face, SPECIAL allocation, tagged skills, traits, tutorial and first quest sequence.
- [ ] **FNV-002** Match S.P.E.C.I.A.L., 13 skills, derivation formulas, HP/AP/carry, level caps, XP, skill points, perk selection, traits, respec and DLC level-cap increases.
- [ ] **FNV-003** Match weapon types, iron sights, ADS accuracy, firing rates, spread, critical hits, reload cycles, jam/condition, recoil and weapon skill prerequisites.
- [ ] **FNV-004** Match ammo types/variants, clips/magazines, ammo switching, weapon mods, repair/jury rigging, weapon condition and projectile/area damage.
- [ ] **FNV-005** Match Damage Threshold and Damage Resistance calculations, armor penetration, creature/weapon special effects, per-limb damage and crippling.
- [ ] **FNV-006** Complete VATS scan/chance formula (measured against original), AP costs, target switching, limb selection, shot queue, pause, criticals, execution playback, camera and return to real-time play.
- [ ] **FNV-007** Match companion AI and companion wheel, orders, perks, death rules, recruit/dismiss, wait locations and inter-companion conflict conditions.
- [ ] **FNV-008** Match faction reputation (Fame/Infamy combinations), Karma, disguises, trespass/stealing reactions, faction hostility, bounties and dialogue reputation gates.
- [ ] **FNV-009** Match dialogue speech checks, skill checks, barter checks, companion/faction conditions, quest branching and end-game faction decisions.
- [ ] **FNV-010** Match radiation accumulation, poisoning/diseases where authored, chem effects, addiction, cures and medical/doctor interactions.
- [ ] **FNV-011** Match optional Hardcore mode: hunger, thirst, sleep deprivation, altered healing/rad rules, ammo weight and companion mortality, and the normal-mode alternatives.
- [ ] **FNV-012** Match crafting/reloading benches, workbenches, recipes, ingredients, campfire cooking, ammo crafting, repair and breakdown.
- [ ] **FNV-013** Implement original Pip-Boy 3000 menu categories, tabs, stats, inventory, data map/quests/radio and keyboard/mouse navigation from installed UI.
- [ ] **FNV-014** Match HUD compass/markers, health/AP/limb/condition/radiation/weapon displays, enemy brackets, crosshair and subtitle/dialogue surfaces.
- [ ] **FNV-015** Match gambling and casino rules: blackjack, roulette, slots, player bans, chips, caps and casino scripts/quests.
- [ ] **FNV-016** Match lockpicking/hacking/speech/barter mini-games and alternate combat/dialogue solutions as applicable in New Vegas.
- [ ] **FNV-017** Match radio station/channel selection, DJ/quest-dependent broadcasts, track rotation and 2D/3D audio mixing.
- [ ] **FNV-018** Match player houses, container ownership, faction territory, world map markers and fast-travel constraints.
- [ ] **FNV-019** Match all DLC mechanics, worldspace transitions and independent quests for Dead Money, Honest Hearts, Old World Blues and Lonesome Road, plus GRA/Courier's Stash items.
- [ ] **FNV-020** Validate complete original New Vegas main-quest faction branches (NCR, Legion, Mr. House, Independent), branching side quests and original endings.

## O. UI, front-end, accessibility and player feedback — P0/P1

- [ ] **PAR-131** Inventory *all* original XML menu documents and their scripts/functions/host callbacks for each game.
- [ ] **PAR-132** Implement a menu manager: named menus, stack order, modal ownership, game pause flags, HUD layering, cursor/focus/keyboard/scroll and non-contradictory `MenuMode` reporting.
- [ ] **PAR-133** Match menu pixel layouts using legal installed source fonts/texture atlases at aspect ratios including 4:3 and 16:9, scaling correctly through resize.
- [ ] **PAR-134** Ensure every clickable or keyboard target triggers canonical gameplay state, or reports unsupported; prohibit inert-looking functional controls.
- [ ] **PAR-135** Match all inventory categorisation, sorting, tooltips, equip/drop/use, repairs, condition bar and carried-weight displays.
- [ ] **PAR-136** Match stats/skills/quests/journal/map/notes/messages, quest markers, objective notifications and selected spell/weapon states.
- [ ] **PAR-137** Complete native dialogue, barter, training, lockpicking, container, message box, confirmation and character creation interfaces.
- [ ] **PAR-138** Match save/load/continue/new/quit/options menus with game-specific labels, slot previews, screenshot/time metadata and corruption/compatibility behaviour.
- [ ] **PAR-139** Match original fade/loading videos/screens/tips and UI sound transitions for entering/leaving cells and worldspaces.
- [ ] **PAR-140** Match HUD timers, stealth detection, compass, location discovery, health/magicka/fatigue/AP and status effect indicators.
- [ ] **PAR-141** Verify script-driven UI opens/closes/queries return consistent results for the shared menu registry.
- [ ] **PAR-142** Match supported subtitle/localisation/font encodings and all visible game UI feedback.

## P. Saves, loads, reset state and original interoperability — P0/P1

- [ ] **PAR-143** Define whether the objective includes **reading/writing original .ess save files**; document that ByroRedux snapshots are currently a separate format.
- [ ] **PAR-144** Implement/verify snapshots for all player stats, character appearance, quests, aliases, script vars, inventory instances, AI packages, reference changes and world state.
- [ ] **PAR-145** Match source-game save restrictions during dialogues, VATS, combat, quests, travel, loading, dead/bleedout and UI states.
- [ ] **PAR-146** Validate quicksave/quickload, numbered/manual/autosave/exit-save semantics and original menu feedback where game-specific.
- [ ] **PAR-147** Verify save loading after process restart, changed masters/load order, disabled mods, missing DLC, corruption and schema evolution.
- [ ] **PAR-148** Verify reference-identity correctness for created refs, dead corpses, unlocked doors, looted containers and changed actors.
- [ ] **PAR-149** Verify game-clock and delayed events survive quickload; queued script effects neither double-run nor disappear.
- [ ] **PAR-150** Verify interior/exterior/cell crossing immediately after load, persistent reference restoration and streamer epoch reset.
- [ ] **PAR-151** Define full/partial old-save migration or a clear incompatibility policy with fail-safe recovery; never delete user saves.
- [ ] **PAR-152** Perform extended soak of save/reload at every major game phase and verify no progressive state loss.

## Q. Script extenders, mod portability and diagnostics — P1/P2

- [ ] **PAR-153** Inventory OBSE/OBSE plugin APIs and xNVSE/NVSE/JIP LN NVSE common script-facing calls used by the targeted mod corpus; map to supported/port/unsupported dispositions.
- [ ] **PAR-154** Verify per-mod record overrides, leveled lists, archive pairing, loose assets and plugin load-order precedence against originals.
- [ ] **PAR-155** Test representative original UI mods (Oblivion DarNified UI, New Vegas UI frameworks) against the actual MenuXml host, not only the image parser.
- [ ] **PAR-156** Test scripts from representative quest mods, weapon mods, companions and gameplay overhauls, including extender dependencies.
- [ ] **PAR-157** Provide a documented native SDK or compatibility service for functionality that can be ported; do not promise arbitrary x86 executable DLL hooks will load unchanged.
- [ ] **PAR-158** Implement script/mod runtime warnings with mod ID, FormID, command/method, effect and impact severity; avoid quiet fallbacks.
- [ ] **PAR-159** Record original-game bug-compatibility decisions (replicate, fix intentionally, or unsupported) for behaviour-sensitive mods and vanilla quests.
- [ ] **PAR-160** Respect original game asset redistribution terms; run content tests using only the user's installed files.
- [ ] **PAR-161** Provide human-readable startup/load reports of all required masters, missing archives, refused records, declined scripts and unsupported menus.

## R. Stability, performance, release and validation sign-off — P1/P2

- [ ] **PAR-162** Run Vulkan validation and stress testing on representative GPUs; match original-visual profile and optional modern profile separately.
- [ ] **PAR-163** Benchmark frame time and pacing on fixed indoor, city, open-world, combat and scene-transition routes under both games.
- [ ] **PAR-164** Monitor memory/VRAM, BSA resource caching, async loading, shader compilation and UI texture allocations across multi-hour sessions.
- [ ] **PAR-165** Eliminate input lockups, stuck menus, repeated movie/music, duplicate resize/swapchain stalls and fatal missing-content crashes.
- [ ] **PAR-166** Test loss of focus, display resize, fullscreen/windowed, alt-tab and clean shutdown with save integrity.
- [ ] **PAR-167** Match original gameplay state at simulated slow/fast frame rates, suspend/resume and time-skipped intervals.
- [ ] **PAR-168** Verify source patches, update compatibility, error diagnostics, default INI options, game launcher and install detection.
- [ ] **PAR-169** Publish per-feature progress metrics automatically: total records, scripts, quests, menus, original comparison gates, unknowns and toleranced failures for each game.
- [ ] **PAR-170** Document unsupported original behaviours and the reproduction/triage process before releasing an 'original parity' claim.

## S. Final release gates — must be met separately for **each** game

- [ ] **GATE-01 OBL:** Clean original Oblivion + official DLC install inventories: no unexplained record/asset/quest/script/menu types or runtime drops.
- [ ] **GATE-02 OBL:** Complete original Oblivion New Game, prison tutorial and character creation with original-progression state.
- [ ] **GATE-03 OBL:** Main story, major guilds, Shivering Isles/other official DLC and representative alternate/failed quests pass original-vs-Redux trace gates.
- [ ] **GATE-04 OBL:** Attributes, levelling, magic, AI, combat, crime, dialogue, original menu UI, map and save/reload pass player-observable comparison.
- [ ] **GATE-05 FNV:** Clean New Vegas + all official DLC data inventory and character creation/tutorial pass.
- [ ] **GATE-06 FNV:** All four faction paths and endings, DLC stories, companions, quests and dialogue branches pass original-vs-Redux state/event trace gates.
- [ ] **GATE-07 FNV:** Combat, firearms, VATS, limb/DT/DR, Hardcore mode, reputation, crafting, casinos and Pip-Boy pass original-vs-Redux tests.
- [ ] **GATE-08 BOTH:** 30-minute and extended save/reload/streaming soaks are deterministic within declared tolerances, with no quest or player-state loss.
- [ ] **GATE-09 BOTH:** Original-look renderer has documented picture/audio differences; modern renderer is explicitly a separate mode if not faithful.
- [ ] **GATE-10 BOTH:** All remaining deviations have individual reasons and user-approved dispositions. If any required original behaviour is incomplete, the title must still be marked **partial**, not "exact".

---

## Engineering order and linkage

| Wave | Priority | Implement and gate | Primary references |
| --- | --- | --- | --- |
| 0 | P0 | Game-version oracle capture, coverage census, test harness | [ROADMAP](../../ROADMAP.md), [compatibility](../engine/game-compatibility.md) |
| 1 | P0 | Oblivion actor values / New Vegas combat-VATS / ObScript / quest correctness | [character rules](../engine/charal.md), [scripting](../engine/scripting.md), [VATS PRD](../prd/vats.md) |
| 2 | P0/P1 | NPC AI, dialogue, game-specific gameplay loops and original menus | [AI packages](../engine/npc-spawn-ai-packages.md), [game UI PRD](../prd/game-ui-integration.md) |
| 3 | P1 | Physics/interactions, animation, audio, economy, world simulation | [feature matrix](../feature-matrix.md), [stream state](../../byroredux/src/cell_loader/stream_snapshot.rs) |
| 4 | P1/P2 | Legacy-look renderer, grain/noise, distant LOD, stable performance | [reference lighting PRD](../prd/reference-lighting.md), [renderer evaluation](../engine/renderer-evaluation.md) |
| 5 | P1/P2 | Save-format decisions, real mod corpus, validation sign-off | [save/load](../engine/save-load-roundtrip.md), [legacy mod PRD](../prd/legacy-mod-compatibility.md) |

### Decomposition rule

Do not submit the entire restoration as one code PR. For every checkbox that requires implementation, create a scoped child issue/PR with: game, original-version oracle, start-state fixture, observed original outcome, current ByroRedux outcome, files/systems involved, regression test, and pass/fail evidence. Link the corresponding `PAR-###`, `OBL-###`, `FNV-###` or `GATE-###` ID. Mark the checkbox done **only once the tested behaviour is correct**, not once the code compiles.

**See also:** [Initial evidence audit](ENGINE_RESTORATION_AUDIT_2026-11-04.md). The audit contains priority findings and source locations; this document is the exhaustive *work-item taxonomy*. A truly exhaustive inventory must be generated from the two original games' entire installed record/script/asset/UI/DLC corpora during Wave 0; this file cannot magically enumerate every individual quest, opcode, FormID, texture or bug without running that census.
