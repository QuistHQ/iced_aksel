//! Copies the canonical `README.md` into `OUT_DIR` so `src/lib.rs` can embed it
//! as the crate-level documentation with a single, layout-independent path.
//!
//! The README lives at the workspace root, which is *outside* this package, so
//! `include_str!("../../README.md")` would resolve while building in the
//! workspace but fail when building the packaged crate (`cargo publish`/docs.rs)
//! because relative paths that escape the package root are not archived.
//! Cargo does however ship the `readme` file itself inside the crate at the
//! package root, so either layout always has exactly one of these candidates.

use std::env;
use std::fs;
use std::path::Path;

/// Where to find the README, in priority order.
const CANDIDATES: [&str; 2] = [
    // Published/docs.rs layout: README.md is archived at the package root.
    "README.md",
    // Local workspace layout: README.md sits at the repository root.
    "../README.md",
];

fn main() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let readme = CANDIDATES
        .iter()
        .map(|candidate| Path::new(manifest_dir).join(candidate))
        .find(|path| path.is_file())
        .unwrap_or_else(|| {
            panic!("could not find README.md next to the crate ({manifest_dir}) or at the workspace root ({manifest_dir}/..)")
        });

    // Rebuild when the README we actually consumed changes.
    println!(
        "cargo:rerun-if-changed={}",
        readme.to_string_lossy().replace('\\', "/")
    );

    let contents = fs::read_to_string(&readme)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", readme.display()));

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    fs::write(Path::new(&out_dir).join("README.md"), contents)
        .unwrap_or_else(|error| panic!("failed to write README.md to {out_dir}: {error}"));
}
