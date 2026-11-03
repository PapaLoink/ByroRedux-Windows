# Windows blend-pipeline startup preparation

Status (3 November 2026): **implemented and tested with original content in one
Oblivion interior**. This is a scoped renderer freeze mitigation, not a claim of
general gameplay or whole-game stability.

## Evidence and cause

The local Windows debugger capture at
`%TEMP%/byroredux-hang-diag/cdb-stacks.log` placed the main thread inside
`nvgpucomp64` during blend graphics-pipeline creation. Previously,
`VulkanContext::prewarm_blend_pipelines` only discovered keys in the current
frame's raster batches. Turning toward a previously unseen material could
therefore compile the full geometry shader synchronously during rendering.

The verification scene is the installed `Oblivion.esm` CELL
`AnvilHeinrichOakenHallsHouse`, FormID `000308C7`, with original resources read
in place from `Oblivion - Meshes.bsa`,
`Oblivion - Textures - Compressed.bsa`, and `Oblivion - Sounds.bsa` under
`C:/Program Files (x86)/Steam/steamapps/common/Oblivion/Data`.
No game content is copied into this repository.

## Change

After scene setup and the initial zero-time scheduler tick, startup collects
distinct authored blend states from every loaded `AlphaBlend` component and
every particle emitter. This includes off-screen meshes and emitters with no
live particles. It retains the authored source/destination factors, material
wireframe state and the same refractive-material predicate used by batching.
Unsupported device wireframe is normalized by the existing pipeline cache.

Required states are prepared before `Engine ready — entering game loop`.
Startup errors propagate and stop entry into gameplay. Each successful variant
is immediately saved through the existing local driver-cache mechanism.
The gameplay clock is reset after preparation so loading time is not delivered
as a gameplay delta. Existing frame-time creation remains available for content
that arrives later.

## Verification

- Native Windows Rust 1.96.0 / MSVC / Vulkan SDK release build: passed using
  `../Start-ByroRedux-Windows.ps1 -Action Build` from the engine checkout.
- `cargo test --release --locked -p byroredux --bin byroredux blend_preparation`:
  both synthetic regression tests passed (hidden material states, deduplication,
  glass/wireframe distinction, dormant emitter discovery).
- Existing duplicate-window-resize tests: 3 passed.
- Renderer frame-parameter tests, including the shared glass predicate's
  consumers: 83 passed. The source contract enforcing the named multilayer
  material constant was updated for the extracted predicate and passed.
- Broader renderer library suite: 1,336 passed, 197 failed, 1 ignored. Remaining
  failures include LF-only self-scan markers and multiline source assertions
  against CRLF checkout files. The full renderer suite is **not green**; its
  detailed local output is `%TEMP%/byroredux-renderer-tests-windows.log`.
- Broader application `render::` tests: 239 passed, 2 ignored, 2 failed.
  The failures are unchanged source-scanning tests:
  `groundcover_hot_path_maps_are_not_siphash` cannot find its LF-only test marker,
  and `every_supplemental_texture_slot_is_written_exactly_once` cannot find the
  LF-only module closing delimiter in CRLF source. Neither failure is a GPU or
  blend-preparation failure; no unrelated source-formatting changes were made.
- Original-content moving-camera smoke: 3,600 frames, `renderer-stepped`,
  `orbit`, default FSR3 Quality, 1280x720 output / 853x480 render. Exit 0 and clean
  Vulkan teardown; 60.3 wall FPS, 16.59 ms p95, 574.82 ms maximum frame.
  The separate maximum-frame scheduler hitch is not claimed fixed.
  The benchmark records `pipeline_compile_ms=0.00`; both pipeline creation
  calls occurred before gameplay, with no later creation calls in the log.
- First preparation measured 72,496.836 ms for `(6,7,false,false)` and
  45,716.113 ms for `(6,7,false,true)`. This confirms expensive compilation
  rather than assuming that a debugger sample alone proves its duration.
- Warm interactive restart reused both variants in 4.586 ms and 4.316 ms.
  The live log continued reporting frames and zero pipeline-compilation time
  for over 100 seconds. Computer control was stopped on the user's physical
  Escape key; the session was left under user control.

Local diagnostic logs (not committed):
`%TEMP%/byroredux-blend-preparation-orbit.log` and
`%TEMP%/byroredux-blend-preparation-interactive.log`.

Reproduction arguments for the finite run, using absolute paths to the three
archives and ESM above:

```text
--cell AnvilHeinrichOakenHallsHouse --bench-frames 3600
--bench-mode renderer-stepped --bench-camera orbit
```

## Limits

Cold or invalidated shader caches still require preparation time during startup;
the initial window may be unresponsive during that synchronous work. This
change moves that work ahead of gameplay and persists its results; it does not
make NVIDIA shader compilation itself faster.

New blend states introduced by later cell loads, streaming, dynamically created
objects, material edits, or a render-pass format change can still reach the
existing frame-time creation path. Those cases and native TAA were not covered
by this moving-camera run. Whole-game stability, original visual fidelity,
gameplay parity and multiplayer remain separately unverified.
