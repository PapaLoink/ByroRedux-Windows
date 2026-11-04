# Oblivion New Game — Product Requirements

**Status**: PROPOSED (2026-11-03). Revision 1 records the owner's request for
the correct New flow. This proposal has not been approved or implemented.

**Scope**: Restore Oblivion (2006) New Game from the main menu through the
opening cinematic, initial character creation and original prison tutorial,
including its later character decisions and final sewer-exit review. Deliver
in verified stages. This does not establish full-game, save-interoperability
or multiplayer parity. New Vegas remains a separate required game target.

**Related**: this document states the promise; the existing designs own the how.

| Mechanism | Owning design |
|---|---|
| New lifecycle, original resources and evidence | [Oblivion front end](../engine/oblivion-front-end.md) |
| Original interactive character menus | [UI](../engine/ui.md) |
| Initial character rules | [Oblivion ruleset](../engine/charal-oblivion-ruleset.md) |
| Original quest/script progression | [Scripting](../engine/scripting.md) |
| Engine persistence and recovery | [Save/load](../engine/save-load-roundtrip.md) |

Normative terms **MUST**, **SHOULD** and **MAY** have their RFC 2119 meanings.
Requirement IDs use the `ONG-` prefix.

## 1. Product thesis

The owner wants New to follow the original game's sequence. A prison-cell
render alone is insufficient: the opening movie, creation decisions and
scripted tutorial must connect through real player state. Each verified
transition must have one owner and must commit only once.

## 2. Terminology

| Term | Meaning |
|---|---|
| Opening movie | Installed `Video/OblivionIntro.bik`, distinct from title/logo movies |
| Initial creation | Player name, race, sex and appearance before ordinary prison play |
| Tutorial decisions | Birthsign/class and final review at their original progression points |
| Session commit | Acceptance of selected character state into the new gameplay session |
| Original comparison | Reproduction in the owner's Oblivion.exe 1.2.0416 with recorded inputs/results |

## 3. Core model

Intended lifecycle: main menu → New confirmation (if present in the original)
→ opening movie → initial creation → prison play → original tutorial
decisions → final review → exterior play. Exact load/fade boundaries, skip
keys, confirmation behavior and quest sequencing are **Phase 0: verify**.

At the modified working checkout on 2026-11-03, New shows an engine confirmation
then starts the prison at its original marker. It does not play the opening
movie or offer interactive character creation. Script menu requests are
currently logged/drained. This is working-tree evidence, not a clean-HEAD claim.

Read-only local evidence: Oblivion.exe 1.2.0416; installed
`Data/Video/OblivionIntro.bik` is 1280×720 Bink video with stereo 44100 Hz
audio, duration 115.482149 s. The existing front-end evidence locates
`Oblivion.esm` CELL 0001FBB9 and REFR 00032AB5 (CGPlayerStartMarker).
The movie exists; full playback and its transition into creation are unverified.

The [original PC manual](https://assets.ctfassets.net/rporu91m20dc/35hpuLKJ2wsgsaWgKmsqaW/1ba5c2c99d147348094c06d0c8649b79/manual_oblivion-goty_pc_en-us.pdf)
describes initial naming/race selection, later class selection and the final
opportunity to revise character choices. It does not verify executable timing.

## 4. Goals and non-goals

### 4.1 Goals

- **ONG-G-001.** New MUST traverse the verified original opening sequence.
- **ONG-G-002.** Initial creation MUST produce the player's selected state.
- **ONG-G-003.** Tutorial decisions MUST follow original script progression.

### 4.2 Non-goals

- **ONG-NG-001.** This work MUST NOT claim complete Oblivion gameplay parity; later world/quest systems retain their own gates.
- **ONG-NG-002.** This work MUST NOT implement New Vegas creation with Oblivion menus or rules.
- **ONG-NG-003.** This work MUST NOT claim Bethesda save-format interoperability from an engine-save round trip.
- **ONG-NG-004.** This work MUST NOT add multiplayer tutorial policies; single-player evidence comes first.

## 5. Requirements by area

### 5.1 Opening (`ONG-OPEN`)

- **ONG-OPEN-001.** Confirmation/cancel behavior MUST match an original comparison. **Phase 0: verify** exact dialog and whether it depends on existing session state.
- **ONG-OPEN-002.** Confirmed New MUST play the configured original opening movie through the existing bounded decoder.
- **ONG-OPEN-003.** Movie completion and supported skipping MUST converge on one creation transition. **Phase 0: verify** original skip inputs.
- **ONG-OPEN-004.** Title music MUST stop at the observed original boundary. **Phase 0: verify** fade/timing.
- **ONG-OPEN-005.** Missing or failed resources MUST return an actionable error without committing a gameplay session.
- **ONG-OPEN-006.** Close/cancel MUST release decoder and audio resources without creating a save.

### 5.2 Character creation (`ONG-CHAR`)

- **ONG-CHAR-001.** Creation menus MUST resolve installed original MenuXml, fonts and artwork. **Phase 0: verify** exact archive paths, authored controls and visibility conditions.
- **ONG-CHAR-002.** Name entry MUST update the player name using the original validation behavior. **Phase 0: verify** limits and confirmation rules.
- **ONG-CHAR-003.** Race selection MUST use playable RACE records from the selected Oblivion load order.
- **ONG-CHAR-004.** Sex selection MUST apply the selected race's original character data.
- **ONG-CHAR-005.** Appearance controls MUST update the actual original-resource player preview. **Phase 0: verify** face/body/hair/eye data and supported morph decoding.
- **ONG-CHAR-006.** Creation acceptance MUST commit the selected values once.
- **ONG-CHAR-007.** Unavailable required creation behavior MUST remain explicitly unsupported rather than silently accept a default character.

### 5.3 Tutorial (`ONG-QUEST`)

- **ONG-QUEST-001.** Prison placement MUST resolve the winning original start marker from the selected load order.
- **ONG-QUEST-002.** Opening dialogue MUST be driven by the original conditions and scripts. **Phase 0: verify** relevant quests, stages, references and dialogue identities.
- **ONG-QUEST-003.** Birthsign selection MUST occur at its original scripted point and apply original BSGN data. **Phase 0: verify** trigger and confirmation.
- **ONG-QUEST-004.** Class selection MUST occur at its original scripted point and apply original class/custom-class rules. **Phase 0: verify** trigger and suggestion logic.
- **ONG-QUEST-005.** Sewer-exit review MUST permit the same original character revisions. **Phase 0: verify** exact fields and recomputation semantics.
- **ONG-QUEST-006.** Save/load MUST preserve committed creation and tutorial state without replaying completed one-shot transitions.

### 5.4 Shared-engine integrity (`ONG-CORE`)

- **ONG-CORE-001.** The flow MUST preserve the selected game identity and load order.
- **ONG-CORE-002.** Synthetic regression tests MUST cover duplicate activation, cancel, skip/completion, decoder failure and commit boundaries.
- **ONG-CORE-003.** Shared parser/rules changes MUST have separate Oblivion and New Vegas regression cases.
- **ONG-CORE-004.** Original files and personal saves MUST remain unchanged during verification.

## 6. Deliberate deviations and risks

The engine already depends on the installed FFmpeg decoder for original movies;
reuse it rather than introduce proprietary runtime code. Appearance, menu
callbacks and tutorial scripting are substantive missing behavior. A movie
followed by the existing prison shortcut is an intermediate implementation,
not acceptance of this promise. Unsupported portions remain labeled under
ONG-CHAR-007 until their own gates pass. No generated assets or invented
quest/dialogue replacements are acceptable.

## 7. Per-game scope and acceptance

| Game | Scope | Reason |
|---|---|---|
| Oblivion | In | Original 2006 New flow requested by the owner |
| Fallout 3 | Out | Separate character-creation sequence |
| Fallout: New Vegas | Out of this flow | Separate required project target; its menus/rules must remain isolated |
| Skyrim LE/SE/AE | Out | Separate creation/startup sequence |
| Fallout 4 | Out | Separate creation/startup sequence |
| Fallout 76 | Out | Online startup outside this milestone |
| Starfield | Out | Separate creation/startup sequence |

Acceptance scenarios use the owner's installed Oblivion 1.2.0416 data and
explicitly recorded plugin order; no official DLC is presumed present.

1. Select New, cancel where the original permits it, and verify the menu stays usable with no new session/save.
2. Complete the installed opening movie, then repeat with every verified skip input; both reach creation exactly once.
3. Choose a different name/race/sex/appearance, accept, and verify the preview and prison player retain the selection.
4. Follow original prison dialogue/tutorial and verify birthsign/class requests against recorded original inputs and outputs.
5. Review choices at the sewer exit; verify final stats, exterior transition and an engine save/load round trip.
6. Test a missing movie/required menu without changing installed files, using independent temporary fixtures; verify recovery with no partial session commit.
7. Run the native Windows application tests and relevant shared Oblivion/New Vegas synthetic regressions.

## 8. Metrics

Track passed lifecycle transitions, creation controls with actual state effects,
tutorial checkpoints compared with the original, and maximum decoder queue
usage. Separate implemented, synthetically tested and original-compared results.
Report counts by game; publish milestone numbers in ROADMAP only after gates pass.

## 9. Delivery sequence

**Phase 0 — Evidence.** Record confirmation/skip behavior, original menu paths,
character resources, quest identifiers/stages and load order. Use existing
readers and read-only original comparisons, without engine implementation.
Exit: each unverified claim has evidence or an explicit blocking gap.

**Phase 1 — Opening lifecycle.** Wire confirmation, title-audio shutdown and
original movie completion/skip to an explicit creation handoff. Keep simulation
held until creation can commit. Exit: synthetic lifecycle/failure/cancellation
tests and installed-movie playback pass; do not ship a default-character bypass.

**Phase 2 — Initial creation.** Implement original menus and actual character
preview/state, then commit into the prison start. Exit: acceptance scenarios
1–3 pass with original comparison and native Windows evidence.

**Phase 3 — Tutorial continuation.** Connect original dialogue/quest/menu
requests, birthsign/class and final review through existing rules and scripts.
Exit: acceptance scenarios 4–7 pass; unresolved checkpoints remain partial.

## 10. Decisions required

1. **Q1 — Delivery scope.** Recommend the full original flow in the phases above,
   beginning with opening lifecycle and initial creation. Alternative: opening
   movie only, explicitly leaving character creation/tutorial unsupported; that
   narrower scope does not satisfy the complete New-flow target.

## 11. Document map

The [front-end design](../engine/oblivion-front-end.md) owns transition evidence;
[UI](../engine/ui.md) owns original-menu interaction; the
[Oblivion ruleset](../engine/charal-oblivion-ruleset.md) owns character state;
[scripting](../engine/scripting.md) owns original progression; and
[save/load](../engine/save-load-roundtrip.md) owns persistence. The already
approved [front-end PRD](oblivion-front-end.md) retains its presentation scope.

## 12. Revision and review log

| Rev | Date | Change | Review |
|---|---|---|---|
| 1 | 2026-11-03 | Proposed original New lifecycle, creation and tutorial gates after working-tree gap review | — |
