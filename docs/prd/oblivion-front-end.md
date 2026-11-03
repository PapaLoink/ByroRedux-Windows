# Oblivion Front End — Product Requirements

**Status**: APPROVED (2026-11-03). Revision 2 records the owner's explicit
approval in this chat of the recommended scope and installed FFmpeg decoder.
Implementation and verification remain outstanding.

**Scope**: The original Oblivion 2006 startup presentation and interactive main
menu on native Windows, reading the owner's installed videos, music, fonts,
MenuXml and textures at runtime. Gameplay and original save-format parity remain
separate requirements; unavailable actions must be identified honestly.

**Related**: This document states the promise; the designs own its implementation.

| Mechanism | Owning design |
|---|---|
| Installed resource evidence, playback and startup lifecycle | [Oblivion intro and main menu](../engine/oblivion-front-end.md) |
| Original MenuXml renderer and input integration | [UI System](../engine/ui.md) |
| Existing launcher and boot handoff | [Launcher](../engine/launcher.md) |
| Existing saves and recovery | [Save/load round trip](../engine/save-load-roundtrip.md) |

Normative terms **MUST**, **SHOULD** and **MAY** have their RFC 2119 meanings.
Requirement IDs use the `OBF-` prefix.

## 1. Product thesis

The owner wants the Oblivion intro and main menu before playing. The installed
presentation resources have been located, but the engine currently enters a
scene immediately and has no Bink player. The corollary is that the front end
uses original installed content and serves only actions backed by real engine
state; an unrelated scene is never presented as a canonical New Game.

## 2. Terminology

| Term | Meaning |
|---|---|
| Startup sequence | Installed logo/legal movies listed by the game's configuration |
| Title presentation | Installed title animation followed by the looping map movie and title music |
| Main menu | Original `menus/options/main_menu.xml`, with engine-owned actions |
| Engine save | A validated ByroRedux save slot, distinct from an original Bethesda save |
| Unsupported action | An authored control whose full host behavior is not implemented; availability and reason are reported |

## 3. Core model

Survey state at the working checkout on 2026-11-03 is documented in the
[resource evidence and existing seams](../engine/oblivion-front-end.md).
This is a modified working checkout; the survey does not attribute those
changes to committed HEAD.

| State | Presentation | Simulation |
|---|---|---|
| Startup | Original logo/legal clips | Stopped |
| Title | Original title animation | Stopped |
| Main menu | Original MenuXml over the original looping map | Stopped |
| Credits | Original credits movie | Stopped |
| Loading | Existing loading integration for a committed supported action | Stopped |
| Gameplay | Existing world renderer and game systems | Running |

## 4. Goals and non-goals

### 4.1 Goals

- **OBF-G-001.** Player launch MUST reach the original title/menu presentation.
- **OBF-G-002.** Every enabled main-menu control MUST execute a supported engine action.
- **OBF-G-003.** Playback MUST leave native window lifecycle events responsive.

### 4.2 Non-goals

- **OBF-NG-001.** This delivery MUST NOT claim complete tutorial/quest/character-creation parity; the startup mapping is still unverified.
- **OBF-NG-002.** This delivery MUST NOT claim Bethesda save-format interoperability; only tested engine slots may be offered.
- **OBF-NG-003.** This delivery MUST NOT replace the external launcher; that process owns preflight and install selection.

## 5. Requirements by area

### 5.1 Content (`OBF-DATA`)

- **OBF-DATA-001.** All presentation assets MUST resolve from the owner's installation, with no proprietary bytes added to source or packages.
- **OBF-DATA-002.** Startup movie order MUST follow the installed configuration entries recorded in the owning design.
- **OBF-DATA-003.** Main-menu layout and artwork MUST come from the original MenuXml, fonts and textures.
- **OBF-DATA-004.** Missing resources or decoders MUST produce an explicit unsupported/error result, without substitute artwork.

### 5.2 Playback (`OBF-PLAY`)

- **OBF-PLAY-001.** Startup completion MUST advance through the state sequence in the core model.
- **OBF-PLAY-002.** Skip input MUST use that same state transition path. **Phase 0: verify** original skip keys and timing by observing the original game; implementation must record deviations.
- **OBF-PLAY-003.** The installed map movie MUST loop behind the main menu without unbounded frame accumulation.
- **OBF-PLAY-004.** Title music MUST play from the installed `Music/Special/TES4Title.mp3` through the existing audio service.
- **OBF-PLAY-005.** Movie audio and video MUST use one playback clock with measured audiovisual drift.
- **OBF-PLAY-006.** Close MUST cancel decoder children and release playback resources.
- **OBF-PLAY-007.** Window resize and focus events MUST continue to be processed while video decoding is pending.

### 5.3 Menu actions (`OBF-MENU`)

- **OBF-MENU-001.** Pointer activation MUST use evaluated authored tile rectangles and IDs; no replacement button layout may be drawn.
- **OBF-MENU-002.** Keyboard focus MUST follow the original menu's enabled controls. **Phase 0: verify** navigation/highlight traits against the installed document and original behavior.
- **OBF-MENU-003.** Continue MUST select the most recent validated engine save and remain unavailable when none exists.
- **OBF-MENU-004.** Load MUST enumerate validated engine slots without modifying original Bethesda saves.
- **OBF-MENU-005.** Options MUST dispatch supported engine settings through the existing settings service; incomplete original option pages must be reported as partial.
- **OBF-MENU-006.** Credits MUST play the installed credits movie and return to the main menu.
- **OBF-MENU-007.** Exit MUST use the existing orderly shutdown path.
- **OBF-MENU-008.** New Game MUST remain explicitly unsupported until the original start record, placement and quest/chargen handoff are verified. **Phase 0: verify** that mapping through installed records and original behavior, then scope the gameplay continuation separately; never launch the Anvil debug fixture as New Game.

### 5.4 Integration and verification (`OBF-BOOT`)

- **OBF-BOOT-001.** Player-mode startup MUST defer world simulation until a supported gameplay action commits.
- **OBF-BOOT-002.** Explicit cell/grid, benchmark, Studio and headless invocations MUST retain their direct routes.
- **OBF-BOOT-003.** Decoder input, dimensions, frame/audio queue sizes and child lifetimes MUST have explicit tested bounds.
- **OBF-BOOT-004.** Synthetic state-machine and cancellation tests MUST require no original game assets.

## 6. Deliberate deviations and risks

| Area | Proposed difference | Risk / requirement |
|---|---|---|
| Decoder | Optional installed FFmpeg instead of Bethesda's Bink runtime | Host dependency; report absence under OBF-DATA-004 |
| New Game | Explicitly unavailable in the initial presentation delivery | Reduced menu functionality; OBF-MENU-008 prevents a false canonical start |
| Saves | Engine slots only | Original saves must not appear compatible; OBF-MENU-004 |
| Options | Supported existing settings may use an engine-native page | Visual parity remains partial and must be reported under OBF-MENU-005 |
| Startup | Vulkan initialization may precede presentation | Measure GPU startup separately from playback responsiveness; OBF-PLAY-007 |

## 7. Per-game scope and acceptance

### 7.1 Per-game scope

| Game | In scope | Reason |
|---|---|---|
| Oblivion | Yes | Owner's requested original 2006 presentation |
| Fallout 3 | No | Different authored menu corpus and startup resources |
| Fallout: New Vegas | No | Different authored menu corpus and startup resources |
| Skyrim LE/SE/AE | No | Scaleform frontend and different startup behavior |
| Fallout 4 | No | Scaleform frontend and different startup behavior |
| Fallout 76 | No | Online startup outside this milestone |
| Starfield | No | Different frontend and data contracts |

### 7.2 Acceptance

| Scenario | Required data / driver | Pass condition |
|---|---|---|
| State and lifecycle unit tests | Synthetic movie/frame/audio sources, `cargo test -p byroredux --bin byroredux front_end` | Completion, skip, repeat, failure and close transitions pass |
| Decoder limits | Synthetic child output and malformed metadata | Oversized or stalled output terminates within its bound |
| Original startup smoke | Installed Oblivion `Data/Video`, `Data/Music`, Misc and texture BSAs | Startup reaches the authored MainMenu, map loops, music plays, no files are copied into the repo |
| Menu action smoke | Empty and populated isolated engine save directory | Continue availability, Load, Options, Credits and Exit match documented support |
| Window smoke | Decode deliberately delayed on the worker | Resize, focus loss and close remain responsive; no decoder survives shutdown |
| Direct-route regression | Existing native Windows interior and benchmark invocations | Front end is bypassed and existing tests remain green |

## 8. Metrics

| Metric | Definition | Granularity |
|---|---|---|
| Startup latency | Player launch to first original frame and to MainMenu | Per native Windows run |
| Playback pacing | Presented frame interval distribution | Per clip / loop |
| Audiovisual drift | Audio time minus presented video timestamp | Per sampled frame |
| Decoder memory | Peak queued frame and audio bytes | Per playback session |
| Menu coverage | Supported controls divided by authored controls | Per main-menu delivery |

Published results belong in ROADMAP's verification record; local diagnostic
captures remain outside committed source. This PRD defines the measurements,
not unmeasured performance claims.

## 9. Delivery sequence

**Phase 0 — Survey and approval (no engine code).** Verify installed resource
paths and decoder feasibility, inspect the current menu/boot/save seams, resolve
the decisions below and record outstanding original-behavior questions.
Exit: a gate-clean proposed PRD, an evidence design and owner approval.

**Phase 1 — Original startup playback.** Add bounded worker playback and the
front-end lifecycle, using the existing renderer/audio boundaries.
Exit: OBF-DATA, OBF-PLAY and OBF-BOOT synthetic checks plus the original startup smoke pass.

**Phase 2 — Original main menu.** Add authored tile input and supported actions,
preserving direct development routes and the unsupported New Game disposition.
Exit: all acceptance rows pass and the verified/partial support matrix is updated.

## 10. Decisions required

1. **Q1 — Decoder dependency.** Recommended: reuse the installed host FFmpeg
   as an optional bounded decoder process, keeping movie bytes in memory and
   reading game files in place. Alternative: implement or integrate a separate
   licensed decoder, which adds a larger dependency and verification scope.
   **Decided** (2026-11-03): owner approved the installed host FFmpeg route.
2. **Q2 — Initial menu scope.** Recommended: ship original startup/title/menu,
   working Continue/Load for engine saves, supported Options, Credits and Exit;
   leave New Game explicitly unavailable until the canonical start is verified.
   Alternative: include full New Game/chargen/tutorial startup now, expanding
   the work into gameplay initialization and quest-parity requirements.
   **Decided** (2026-11-03): owner approved the recommended narrow delivery.

## 11. Document map

| Need | Go to |
|---|---|
| Installed resource evidence, architecture and unresolved behavior | [Oblivion intro and main menu](../engine/oblivion-front-end.md) |
| Existing MenuXml and UI mechanisms | [UI System](../engine/ui.md) |
| External launcher boundary | [Launcher](../engine/launcher.md) |
| Existing save state and recovery | [Save/load round trip](../engine/save-load-roundtrip.md) |
| Review and owner approval process | [PRD standard](README.md) |
| Published compatibility state | [ROADMAP](../../ROADMAP.md) |

## 12. Revision and review log

| Rev | Date | Change | Review |
|---|---|---|---|
| 1 | 2026-11-03 | Initial scoped proposal based on the installed resource survey | — |
| 2 | 2026-11-03 | Owner approved the recommended decoder and narrow menu scope in chat | owner: approved |
