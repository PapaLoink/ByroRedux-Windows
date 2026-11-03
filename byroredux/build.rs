// On Windows MSVC, the main executable otherwise reserves only 1 MiB of
// stack. The native Oblivion smoke test overflowed during Vulkan renderer
// initialization (immediately after SSAO pipeline creation). A 16 MiB stack
// reserve was independently validated with a two-frame original-asset run.
//
// The /STACK value reserves *virtual address space*; Windows commits stack
// pages on demand. Apply this only to the main Windows executable, not to
// dependencies, other platforms, or Rust test binaries.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-link-arg-bin=byroredux=/STACK:16777216");
    }
}
