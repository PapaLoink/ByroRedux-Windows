# Native Windows test baseline

**Verified 3 November 2026:** the native Windows release-profile workspace tests
pass with Rust 1.96.0, Visual Studio 2022 MSVC and Vulkan SDK 1.4.363.0.

```powershell
# From the engine checkout, with the native MSVC/Vulkan build environment set
# up as in the workspace's Start-ByroRedux-Windows.ps1 launcher:
cargo test --workspace --release --locked --no-fail-fast
```

Result: **11,705 passed, 0 failed, 382 ignored**, across 114 test targets,
including unit, integration and documentation tests; Cargo exited 0. No tests
were newly ignored or disabled. Existing ignored tests retain their explicit
real-data, graphics, manual or other prerequisites. This run used the release
profile; a separate debug-profile run and Linux/macOS execution were not done.

The final local transcript is
`%TEMP%/byroredux-windows-workspace-tests-final.log` (not committed).

## Causes and fixes

| Failure | Cause | Fix and evidence |
| --- | --- | --- |
| Most of the 197 initially recorded renderer failures | Source-contract tests search LF-delimited text, but `core.autocrlf=true` produced CRLF checkout files | Explicit LF checkout rules for source/shader/documentation inputs. Shared test-module scanners additionally accept LF, CRLF and mixed markers without copying their input. All 12 scanner tests pass; renderer suite: 1,533 passed, 0 failed, 1 existing ignore. |
| Renderer allocator and descriptor allowlists | Slash-separated allowlist names were compared with native Windows path strings | Compare `Path` components; preserve the same allowlists and counts. |
| Record-parser exemption guard | Native backslashes prevented repository-relative filenames matching exemption keys | Normalize the guard's filenames to `/`; no new parser exemptions. |
| Save-schema fingerprint | Repository-relative Windows filename separators changed the source fingerprint | Normalize filename spelling before hashing. Existing fingerprint and `FORMAT_MAJOR` remain unchanged and the guard passes. |
| Detector/launcher CLI and boot-argument tests | Unix `/tmp` paths were assumed fully absolute, and joined paths were assumed to contain `/` | Use native absolute fixture paths and native joins while keeping flag-order and strict-parsing assertions. |
| Steam library discovery fixture | Unescaped Windows backslashes in the synthetic VDF were interpreted as escape sequences | Escape the fixture's path strings; game detection: 59 passed. |
| Texture-tool preflight | Canonical Windows `\\?\C:\...` ancestors did not match sysinfo's ordinary `C:\` mount points | Canonicalize mount points before comparing ancestry. A Windows canonical-temp-path regression test and both previously failing pipeline tests pass; texture tool: 19 passed. |
| Scaleform archive navigation | Virtual archive paths were passed through OS-dependent `Url::from_file_path` / `to_file_path`, which require host drive semantics on Windows | Build virtual `file:///` URLs with URL path segments and percent-decode archive paths independently of the host OS. UI: 118 passed, 0 failed, 3 existing ignores. Tests retain relative imports and query/fragment handling and reject remote hosts, drive paths, NUL and invalid UTF-8. |

## Checkout policy

`.gitattributes` makes relevant source inputs LF on both Windows and Unix,
independently of `core.autocrlf`. The vendored FidelityFX SDK is exempt and
retains its previous checkout behavior. Binary shaders, archives and other
binary fixtures are not converted.

The existing local checkout was refreshed by converting only CRLF terminators
in tracked files covered by the new rules. Other file contents were preserved;
there is no bulk source-formatting diff and no staged change. Existing clones
whose unchanged source files still contain CRLF need that working-tree newline
refresh too; adding attributes or renormalizing the index alone does not rewrite
their existing working-tree bytes.

A separate temporary Git repository verified an actual fresh checkout with
`core.autocrlf=true`: synthetic Rust, shader and Markdown inputs remained LF,
a binary fixture was byte-identical, and the exempt SDK fixture retained its
previous Windows checkout behavior.

## Scope

This proves the enabled tests pass on native Windows. It does not establish
original-game visual fidelity, complete Oblivion gameplay or DLC parity,
extended graphics stability, or multiplayer support. No installed game files,
user saves or game configuration were changed for this work.
