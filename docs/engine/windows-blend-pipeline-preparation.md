# Windows blend-pipeline preparation

Status (3 November 2026): **partial compatibility verification**. Startup,
late-loaded cell/material discovery and asynchronous preparation are implemented.
Original-content smoke tests cover the interior below, native TAA, and a later
Anvil Castle Great Hall debug load under FSR3 Quality. This mitigates blend
pipeline compilation stalls; it does not establish whole-game stability.

## Extension to loaded cells and streamed materials

The host scans authored blend states before the first simulation tick and again
following streaming, debug/save/door loads and equipment/appearance imports each
tick. Hidden meshes and dormant emitters remain included. A renderer preflight
also checks final draw states before acquiring a swapchain image, covering
late/generated material changes without entering synchronous frame compilation.

Missing normalized keys are deduplicated and compiled on one named worker.
The worker owns a private Vulkan pipeline cache seeded from the main cache,
persists each successful variant, and returns pipelines for installation after
join. Cache merging runs only after completion; caches are never shared across
threads. Partial results are destroyed on errors or panic, and failures terminate
loading rather than rendering substitute opaque materials.

While work is pending, the host yields to winit at 16 ms intervals with the title
`ByroRedux | Preparing shaders...`. Simulation and drawing pause, input is cleared,
and focus loss and native close remain handled. Resize is deferred to the latest
window dimensions, including camera aspect, until compilation completes.
Gameplay time resets before resuming; prior mouse capture is restored only when
the window still has focus. Close hides the window immediately; Vulkan teardown
must wait for the driver call to return, since compilation cannot be cancelled.
Resize and context destruction join the worker before destroying its render pass,
pipeline layout or device. No proprietary resources are copied or modified.

### Extension verification

- Native Windows Rust 1.96.0/MSVC release build passed using
  `../Start-ByroRedux-Windows.ps1 -Action Build`.
- Complete release-profile renderer library suite: 1,536 passed, 0 failed,
  1 existing ignore (`cargo test --release --locked -p byroredux-renderer --lib`).
- Complete release-profile application binary suite: 3,458 passed, 0 failed,
  79 existing ignores (`cargo test --release --locked -p byroredux --bin byroredux`).
  These final runs include all changes and the worker lifecycle regressions.
- Synthetic regressions cover later cell states, material edits, dormant emitters,
  normalization/deduplication, nonblocking polling with a deliberately held worker,
  failure propagation and acquire/resize/teardown ordering.
- Native TAA: 300 stepped orbit frames in `AnvilHeinrichOakenHallsHouse`,
  `pipeline_compile_ms=0.00` (32.0 wall FPS, 31.02 ms p95). This run overlapped
  compilation of the application tests; its FPS is not a performance baseline.
- Final native TAA rerun: 120 stepped orbit frames, exit 0, clean Vulkan teardown,
  `pipeline_compile_ms=0.00`; 34.3 wall FPS, 37.79 ms p95. This also overlapped
  test compilation and is a correctness smoke, not an isolated FPS comparison.
- FSR3 Quality: startup keys `(6,7,false,false)` and `(6,7,false,true)` compiled
  on the worker in 5.784 ms and 5.962 ms. A subsequent localhost debug request
  loaded original `Oblivion.esm` CELL `AnvilCastleGreatHall`, FormID `000308CC`,
  and discovered additive state `(6,0,false,false)` before drawing. The worker
  prepared it in 1.881 ms. Subsequent stats reported zero pipeline compilation;
  the Windows process responsiveness probe passed. The debug-loaded scene's
  camera/visual correctness is outside this shader test.
- The test instance exited through `engine.quit` and logged clean Vulkan teardown.
- Diagnostic logs remain local in `%TEMP%/byroredux-shader-*.log`; the smoke
  used `%TEMP%/byroredux-shader-streaming-cache` via `BYRO_CACHE_DIR` without
  invalidating the user's driver cache.

These warm-driver runs do **not** reproduce a fresh 45–72 second NVIDIA compile.
The held-worker synthetic regression verifies the nonblocking poll; OS paint,
resize and close throughout a genuinely long cold compile still need manual
verification. Exterior boundary streaming is covered by the common discovery
path and synthetic states, but has not had a new original-content traversal run.

The remaining sections record the earlier synchronous startup fix and evidence.

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

The broader-suite failure counts below are historical results from the shader
fix session. They were subsequently resolved in the
[native Windows test baseline](windows-tests.md): the complete release-profile
workspace run now passes with 11,705 passed, 0 failed and 382 existing ignores.

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

The earlier startup-only limitation is superseded by the worker path described
above. Shader compilation itself is not faster. During pending preparation the
last presented image is retained (the initial window may have no game image);
only the native window title provides preparation status. Full Vulkan startup,
opaque/compute pipeline creation, render-pass recreation and non-shader cell
loading work may still block the event thread. Whole-game fidelity, gameplay
parity, exterior traversal and multiplayer remain separately unverified.
