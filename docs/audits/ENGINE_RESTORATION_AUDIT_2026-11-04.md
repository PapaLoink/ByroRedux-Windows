# Engine Restoration Audit — Oblivion & Fallout: New Vegas

**Date:** 2026-11-04
**Target:** ByroRedux-Windows, compared with the player-observable behaviour of Bethesda's **The Elder Scrolls IV: Oblivion (2006)** and Obsidian/Bethesda's **Fallout: New Vegas (2010)**.
**Status:** AUDIT / RESTORATION BACKLOG — **not a declaration of feature parity**
**Change type:** Documentation and acceptance criteria only. No engine code, game data, save files, or third-party assets are changed in this PR.

> **Executive finding:** ByroRedux can read and display large portions of both games' original ESM/BSA/NIF content. That is **asset compatibility**, not yet **gameplay or executable compatibility**. Original-quest execution, progression, AI, weapon combat/VATS, menu host behaviour, physics interactions and cross-session fidelity require separate verification. Neither title should be advertised as a drop-in replacement for its original executable.

## 1. Evidence, scope and confidence

This is a **source-and-repository audit**, not a side-by-side certification. Findings are grounded in the current workspace's implementation, the dated measurements in [ROADMAP.md](../../ROADMAP.md), the [feature matrix](../feature-matrix.md), and the PRDs and notes referenced below. Older documents sometimes describe gaps that newer code has closed; this audit prioritises actual code when available. "Implemented" is not equivalent to "validated against the original executable."

**Local working-tree boundary.** During the 2026-11-04 review the Windows working tree contained ~31 pre-existing modified/untracked entries, including an in-progress **Oblivion-only** MenuXml integration in `byroredux/src/oblivion_game_menu.rs` and `byroredux/src/oblivion_secondary_menu.rs`. Those changes are **not included in this documentation PR**. Notes about them describe the inspected local state; reverify them once the corresponding implementation is committed. The default branch may still have the generic egui menu route.

**Installed-game boundary.** Oblivion is present in the inspected Windows Steam directory. No local New Vegas installation was visible at that location during this audit. New Vegas findings draw on source, repository fixtures and existing bench records rather than a fresh on-device original-executable comparison.

**Evidence classes:** `CODE` = directly checked source; `MEASURED-REPO` = existing test/benchmark/census cited with its date; `DOC` = project documentation or proposed design; `UNVERIFIED` = original-executable parity or complete gameplay behaviour not yet demonstrated.

### What is already substantive

| Capability | Current evidence | Important limit |
| --- | --- | --- |
| BSA/NIF asset parsing | `MEASURED-REPO`: ROADMAP reports 9,612/9,612 clean Oblivion NIFs (2026-09-07) and 20,746/20,746 clean New Vegas NIFs (2026-08-28). | A fully parsed mesh can still render, animate, collide or behave incorrectly. |
| Interior and exterior worlds | `MEASURED-REPO`: installed-data interior scenes, exterior grid/terrain and streaming smoke paths exist for both games. | Full-quest traversal and every worldspace boundary are not certified. |
| Rendering | `CODE`: Vulkan ray queries, stochastic shadows, GI, SVGF, TAA, FSR 3.1, fog, water and lighting. | Not the source games' renderer or visual output. |
| NPC representation | `CODE`: classic-game body/skeleton/FaceGen, inventory/outfits, walk animation, AI packages, Rapier locomotion. | Combat intelligence, package conditions and animation coverage differ. |
| Legacy scripting | `CODE`: SCDA/ObScript interpreter, quest lifecycle, condition engine, effects and event hooks. | Unmodelled calls decline; script coverage does not prove vanilla quest completion. |
| Save/load | `CODE`: validated snapshots, rotating slots, atomically written files, live reload and corruption handling. | Uses ByroRedux format, not original Oblivion or New Vegas saves. |
| Stream continuity | `CODE`: `byroredux/src/cell_loader/stream_snapshot.rs` and `reference_state.rs` park/restore a specified subset of actor/script state. | Some transient state is deliberately discarded; not total state parity. |

## 2. Severity-ranked findings

**P0 = prevents core vanilla-game parity; P1 = major functional/compatibility deficit; P2 = visual, presentation, coverage or quality risk.**

| ID | Priority | Game | Finding | Evidence | Player impact |
| --- | --- | --- | --- | --- | --- |
| ER-01 | **P0** | Oblivion | The `CharacterRulesProfile::OBLIVION` branch uses `NpcStatModel::None` and `RulesetBuilder::None`; Oblivion-specific progression/actor-value population is not fully connected. | `CODE`: `crates/core/src/character/profile.rs`; `crates/core/src/character/tes.rs`; `docs/feature-matrix.md`. | Incorrect or absent attributes, derived pools, skill growth, levelling and values in the Stats menu. |
| ER-02 | **P0** | Both | The original compiled script dialect and its full effect/runtime host contracts are not complete. The dated repository census reported **~81% Oblivion / ~84% FNV script call-site lines modelled**, *not* equivalent percentages of fully playable quests. | `CODE + DOC`: `docs/engine/scripting.md` § M47.3; runtime `crates/scripting/src/`; feature matrix. | Vanilla quests, actions and object/dialogue events can decline or behave differently. |
| ER-03 | **P0** | New Vegas | VATS has mode, AP, targeting and queue plumbing, but lacks original menu, full playback/camera, limb health/crippling and measured original-to-hit percentages. | `DOC + CODE`: `docs/prd/vats.md`; VATS and combat code. | Defining New Vegas gameplay loop not reproduced. |
| ER-04 | **P0** | Both | General weapon combat and NPC decisions remain narrower than the original engines. Combat AI describes straight-line chasing, cooldown hits, limited reaction/animation and no full blocking, ranged or magic attack behaviour. | `CODE`: `byroredux/src/systems/combat_ai.rs`, `combat.rs`; feature matrix. | Combat encounters do not match source-game tactics, weapon types or animations. |
| ER-05 | **P1** | Both | Menu host and gameplay data contracts are incomplete. A MenuXml image or clickable tab is not by itself a functioning original menu; the generic/native fallback persists outside authored paths. | `CODE + DOC`: `crates/menuxml`, `crates/debug-ui`, `docs/prd/game-ui-integration.md`. | Original interactions, stacked modal menus, scripting queries and input focus can disagree. |
| ER-06 | **P1** | Oblivion | WIP Stats/Magic/Map integration relies on incompletely populated live stats; Magic lists truncate at six entries and selection is not wired to ordinary player casting; map uses a fixed installed Cyrodiil DDS without a live local map or complete markers. | `CODE (local WIP only)`: `byroredux/src/oblivion_secondary_menu.rs`; `oblivion_game_menu.rs`. | Original art appears without full original mechanics. |
| ER-07 | **P1** | New Vegas | The WIP original pause/inventory/MenuXml artwork integration is explicitly gated to `Oblivion.esm`; no equivalent fully integrated original New Vegas player menus/Pip-Boy path is evidenced. | `CODE (local WIP only)`: `oblivion_game_menu::install`; `docs/engine/ui.md`. | New Vegas retains non-original menu surfaces or unserved actions. |
| ER-08 | **P1** | Both | Lockpicking, terminal/hacking and several physics/object interactions are incomplete; Rapier is not a Havok-compatible simulation. Dynamic weapon/item physics are an explicit gap. | `CODE`: `byroredux/src/lockpick.rs`; `docs/feature-matrix.md` Physics. | Doors, containers, scripted interactions and object manipulation can diverge. |
| ER-09 | **P1** | Both | AI package procedures and conditions are partial. Classic-game package locomotion works, but CastMagic, special Find targets, some UseWeapon conditions and full combat/navigation semantics are missing. | `CODE + DOC`: `docs/engine/npc-spawn-ai-packages.md`; feature matrix. | NPC routines and encounters do not consistently reproduce schedules or responses. |
| ER-10 | **P1** | Both | Save format is ByroRedux-specific; arbitrary original-game save import/export and complete load-order/mod semantics are not proven. Native executable-hook mods are not directly compatible. | `CODE + DOC`: `crates/save`, `docs/engine/save-load-roundtrip.md`, `docs/prd/legacy-mod-compatibility.md`. | Original saves and many community mods cannot be assumed to work. |
| ER-11 | **P1** | Oblivion | Character-creation/tutorial and the real original-new-game journey have incomplete fidelity, despite front-end movie/menu loading. | `DOC`: `docs/engine/oblivion-front-end.md`; feature matrix. | A booting title screen is not a full original game start. |
| ER-12 | **P1** | Both | Dialogue response/menu presentation and some world-facing scripts are partial; unavailable ObScript operations include original disposition/topic gates and game-specific story machinery. | `CODE + DOC`: `docs/engine/scripting.md`; `docs/engine/dialogue-trees.md`. | Quest conversations may advance incorrectly or lack original options. |
| ER-13 | **P2** | Both | Stochastic lighting/SSAO, temporal accumulation and FSR can show visible moving grain, shimmer or blur. This is not equivalent to the original games' visual style. | `CODE`: `crates/renderer/shaders/composite.frag`, `triangle.frag`, `ssao.comp`, `svgf_temporal.comp`, `svgf_atrous.comp`; `docs/prd/reference-lighting.md`. | Distracting noise, material/lighting mismatch and weak visual parity. |
| ER-14 | **P2** | Both | Distant-object/terrain LOD, regional audio, footsteps, visual effects and weather retain game-specific coverage gaps. | `DOC + CODE`: `docs/feature-matrix.md`; `docs/prd/distant-lod-completion.md`; `docs/engine/renderer.md`. | Pop-in, misplaced distant assets, incomplete audio and atmospherics. |
| ER-15 | **P2** | Both | Benchmark results describe ByroRedux scenes, **not** a matched comparison to either original executable; renderer RT quality changes adaptively with GPU budget. | `MEASURED-REPO`: ROADMAP 2026-10-22 Prospector scene **83.8 FPS native TAA / 101.9 FPS FSR Quality** on RTX 4070 Ti at 1280×720; `crates/renderer/src/vulkan/scene_buffer/ray_budget.rs`. | Performance/quality regressions cannot be explained by a single FPS headline. |

### Corrections to avoid stale or overstated findings

1. **Streaming is not wholly unimplemented.** `stream_snapshot.rs` now captures and restores selected actor positions, travel/escort/package/seat state, and `reference_state.rs` parks various script and actor properties. The older *proposed* `docs/engine/stream-boundary-state-continuity.md` is not authoritative about what is missing today. Still audit boundary and save/reload continuity with real NPCs.
2. **Legacy scripting is not zero.** A growing SCDA interpreter, result-script execution and active effects exist; the issue is **missing behaviours and game-level parity**, not absence of an interpreter.
3. **VATS is not absent.** Its data/selection model is substantial; it is not yet the complete user-visible original loop.
4. **The local Oblivion menu updates do not belong to this PR.** The archive-backed tests exercised rendering and hit targets; an empty-world XML test cannot prove live player stats, a real inventory icon or actual quest/map mechanics.
5. **A 100% clean NIF parse rate is not a 100% compatibility rate.** Animation, collision, visual materials, runtime script behaviour and UI contracts are separate acceptance dimensions.

## 3. Rendering/noise audit — reproducible isolation

The original games use their own historic raster-based graphics and lighting choices. ByroRedux instead uses ray-query shadows/reflections, bounded GI, SVGF temporal/spatial denoising, TAA and FSR 3.1. **These are not output-equivalent systems**, even when consuming identical assets.

Distinct likely contributors to the grain users report:

- **Ray-traced direct/GI sampling:** few stochastic rays cause bright/dark pixel variance; temporal history can fail at motion, disocclusion and lighting changes.
- **SVGF denoising:** may leave high-frequency residuals or create unstable large-scale smearing; the source explicitly discusses removing moving-camera one-sample-per-pixel grain.
- **SSAO sampling:** finite stochastic occlusion taps can leave small-scale noise.
- **Composite dithering:** `composite.frag` adds `preResolveDither()`; the uploaded `volume_params` carries a `1/1024` amplitude. Intended to be subtle; do not assume it causes heavy grain without testing.
- **FSR 3.1 Quality default:** renders below output resolution, then reconstructs. May amplify or smear pre-existing noise; native TAA is a useful diagnostic control.
- **Adaptive ray budget:** quality tiers change with measured lighting time; the same scene may trade stability for speed on a different GPU.

**Required capture matrix (same camera, scene, exposure, output size and warmup):**

| Variant | Purpose | Acceptance |
| --- | --- | --- |
| `r.upscaler taa` and `ctx.upscaler` | Native-resolution temporal control and actual configuration. | Capture and log mode, render extent and present extent. |
| `render.debug indirect_only` | Isolate indirect/GI contribution. | Identify temporal noise, missing energy and moving-camera disocclusion. |
| `render.debug direct_only` / `render.debug shadow_visibility` | Separate direct lighting and shadow noise. | Shadow edges do not crawl or visibly pulse at a parked camera. |
| `render.debug final` | Return to final image for paired comparisons. | Image should not show dominant film-static-like motion on flat surfaces. |
| FSR Quality vs TAA, consistent RT tier | Determine whether upscale amplifies existing variance. | No unacceptable texture shimmer or unstable history; publish artifact pairs. |

Use `docs/engine/renderer-evaluation.md`, `docs/prd/reference-lighting.md`, `docs/engine/shadow-pipeline-tradeoffs.md` and `docs/prd/distant-lod-completion.md` as the test framework. Do **not** remove dithering or change ray budgets on intuition; first isolate which pass contributes the visible grain. Add a separate **legacy visual-fidelity profile** only if original-art-direction parity is an explicit product goal.

## 4. Restoration workstreams and definition of done

### R1 — Reference-oracle tests for each original game (**first priority**)

- [ ] Assemble **legally installed, user-supplied** Oblivion and New Vegas versions with deterministic start state, load order, resolution and camera/control traces; do not commit Bethesda assets.
- [ ] Capture original-versus-ByroRedux runs of a new game, player stats, one dialogue/quest, indoor/outdoor travel, inventory, combat, saving and restart.
- [ ] Record **observed values and event order**, not only screenshots or parse success; distinguish known originals, measured engine state and unsupported behaviours.
- [ ] Publish per-game regression fixtures and an explicit pass/fail/unsupported outcome. New Vegas Windows live parity remains **unverified** until a local install is available.

### R2 — Oblivion actor values, skills and levelling (**ER-01**)

- [ ] Connect the legacy hard-coded Oblivion attribute/skill IDs to the canonical actor-value resolver, without requiring AVIF records that Oblivion does not ship.
- [ ] Replace `RulesetBuilder::None` and `NpcStatModel::None` only after sourced test cases validate player/NPC/creature population.
- [ ] Test Health/Magicka/Fatigue derivation, regeneration, skill use, major/minor skills, the level-up trigger and sleep confirmation against the original.
- [ ] Verify the original Stats UI reads this state and displays sensible values in live gameplay rather than zero defaults.

### R3 — Vanilla ObScript and quest correctness (**ER-02, ER-12**)

- [ ] Re-run the installed SCDA command census for **both** titles, categorised by real call-site frequency and affected vanilla quest.
- [ ] Implement/validate the most common declined *semantics*: actor values, disposition/topic state, selected story devices, script-driven combat, game-specific quest/object effects, menu and fast-travel operations.
- [ ] Gate command failures with explicit diagnostics; never silently treat a declined conditional as true.
- [ ] Prove complete opening, middle, and completion of one representative story quest per game and one dialogue-result-script chain.

### R4 — New Vegas combat/VATS and Oblivion combat/magic (**ER-03, ER-04, ER-09**)

- [ ] Validate player firearm/ammunition/projectile/reload/damage semantics and NPC ranged targeting on FNV; test Oblivion melee, blocking, spell casting and hit reactions.
- [ ] Complete VATS input/menu, body-part targeting and percent chance validation, AP costs, queued playback/camera, limb conditions, crippling and associated script events.
- [ ] Replace straight-line combat chasing with authored navigation and combat-state behaviour where the originals require it.
- [ ] Test target loss, death, alert propagation, cover/path obstacles, and weapon/magic animation with source assets.

### R5 — Original MenuXml, dialogue and player-facing controls (**ER-05–07, ER-11**)

- [ ] Commit and integrate the Oblivion-only MenuXml work separately; test it with **live populated world state** (not empty-world art smoke tests).
- [ ] Give native MenuXml its real menu stack, modal focus, hover/selection, scrolling, keyboard/controller and script-query semantics.
- [ ] Implement full inventory-icon archive mapping, multi-page lists, usable spell effects/casting, actual map position/local maps/markers and journal navigation.
- [ ] Wire original Options controls to settings with actual backends; keep unsupported settings visibly disabled/reported.
- [ ] Build an independent **New Vegas** original menu/Pip-Boy route using the correct installed UI assets; do not reuse Oblivion-only artwork or assume the same ID semantics.
- [ ] Add character creation/tutorial, original dialogue/notification and save-slot metadata parity gates. Do not imply original Bethesda save-file interoperability.

### R6 — World simulation, physics, audio and content edge cases (**ER-08–10, ER-14**)

- [ ] Compare physics/collision, actor traversal, ragdoll and dynamic dropped-item behaviour with the originals.
- [ ] Provide original lockpick/terminal gameplay where applicable; test permission, perk, inventory and minigame outcomes.
- [ ] Expand package runtime coverage for special targets, spell use and conditional actions, with genuine schedule tests.
- [ ] Verify streaming with moving actors and scripted objects across normal tile eviction, worldspace changes, save/restart and quest transitions; retain successful state-restoration tests.
- [ ] Check terrain/object/tree LOD, water, weather, sounds, footsteps and distant audio on both games.
- [ ] Define mod compatibility by **mechanism** (record overrides, assets, script functions, UI and native extenders), not by a global "supports mods" flag.

### R7 — Graphics parity and quantitative quality (**ER-13, ER-15**)

- [ ] Establish matched camera/exposure/time-of-day pairs against original executables and separately record modern-lighting quality (different objectives).
- [ ] Run the grain/noise matrix above, isolating GI, direct/shadow, AO, denoiser and upscaler quality.
- [ ] Gate temporal stability, disocclusions and GI energy to a published reference/threshold; record adaptive ray tier.
- [ ] Publish repeatable FPS/frame-time and visual-quality numbers on named hardware. Avoid interpreting a ByroRedux-only scene benchmark as original-engine speedup.

## 5. Suggested pull-request decomposition

This audit is intended as the **tracking PR**, not a giant single implementation PR. After accepting the scope, split fixes so their tests and game-specific regressions stay reviewable:

1. **Character:** Oblivion actor-value IDs, population, ruleset and progression.
2. **Scripts:** per-game declined-call coverage and quest oracles.
3. **Combat:** FNV firearms/VATS, Oblivion melee and magic, NPC tactics.
4. **UI:** original Oblivion menus, independently original New Vegas menus, host registry and events.
5. **World:** physics/locks/interactions, NPC streaming and quest persistence.
6. **Renderer:** grain isolation, temporal stability, LOD and visual-parity profiles.
7. **Compatibility:** save-format boundaries, mod mechanisms and original-executable smoke coverage.

A subsystem is **not "restored"** until a real-game behaviour test demonstrates its original-facing contract; a green parser test, empty-scene MenuXml raster, or isolated unit test alone is insufficient.

## 6. Existing source documents

- [Project roadmap](../../ROADMAP.md) — dated parser/bench/test measurements and open milestones.
- [Feature matrix](../feature-matrix.md) — game-by-game implementation status; some statements predate current local menu work.
- [Compatibility overview](../engine/game-compatibility.md) — file format and installed-game provenance; reconcile dated counts with ROADMAP.
- [Oblivion front-end](../engine/oblivion-front-end.md) — original title-screen/Windows bring-up, verification limits.
- [Scripting](../engine/scripting.md) — ObScript coverage, unsupported calls, quest runtime.
- [NPC AI packages](../engine/npc-spawn-ai-packages.md) — source data to running behaviour; date-sensitive.
- [Character rules](../engine/charal.md), [Oblivion rules](../engine/charal-oblivion-ruleset.md), [FO3/FNV rules](../engine/charal-fnv-fo3-ruleset.md).
- [Game UI PRD](../prd/game-ui-integration.md) — original menus, host calls, menu registry and modal behaviour.
- [VATS PRD](../prd/vats.md) — partial FNV/FO3 VATS and acceptance gates.
- [Legacy mod compatibility PRD](../prd/legacy-mod-compatibility.md) — per-mechanism compatibility contract.
- [Reference lighting PRD](../prd/reference-lighting.md), [Renderer evaluation](../engine/renderer-evaluation.md), [Distant LOD PRD](../prd/distant-lod-completion.md).
- [Streaming snapshot code](../../byroredux/src/cell_loader/stream_snapshot.rs), [durable reference state](../../byroredux/src/cell_loader/reference_state.rs).
- [Save/load round trip](../engine/save-load-roundtrip.md) — ByroRedux persistence guarantees and format limits.

## 7. Acceptance of **this documentation PR**

- [x] A dated comparison names the original games and distinguishes asset fidelity from runtime parity.
- [x] P0/P1/P2 findings have concrete code/documentation pointers and user-visible consequences.
- [x] Original-game tests that have **not** been run are not represented as passes.
- [x] Grain, performance, original UI, saves, mod compatibility and streaming limitations are included.
- [x] Restoration work is split into independently testable follow-ups.
- [ ] Engineering owners and original-executable ground-truth fixtures are assigned in subsequent issues/PRs.
- [ ] Follow-up implementations meet the per-workstream original-facing acceptance gates.

**Do not merge this document as evidence that the underlying P0/P1 defects are fixed.**
