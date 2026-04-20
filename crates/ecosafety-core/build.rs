// FILE ./crates/ecosafety-core/build.rs
//
// Environment preflight for ecosafety-core.
//
// Invariants:
//   - CARGO_HOME must be set.
//   - CARGO_HOME must point to a path under /mnt/oss.
//   - /mnt/oss must exist and be a directory.
// If any check fails, compilation aborts with a clear diagnostic.
//
// This is a non-actuating, std-only build script.

use std::env;
use std::path::{Path, PathBuf};
use std::process;

fn main() {
    // We always want this check to run; if the environment changes,
    // a rebuild should re-evaluate it.
    println!("cargo:rerun-if-env-changed=CARGO_HOME");

    // 1. Ensure OSSFS mount exists.
    let oss_root = Path::new("/mnt/oss");
    if !oss_root.exists() || !oss_root.is_dir() {
        eprintln!();
        eprintln!("error: ecosafety-core requires an OSSFS mount at `/mnt/oss`.");
        eprintln!("       `/mnt/oss` was not found or is not a directory.");
        eprintln!();
        eprintln!("hint: mount your 16E VFS at `/mnt/oss` before building,");
        eprintln!("      and ensure it is writable from this container/session.");
        hard_fail();
    }

    // 2. Read CARGO_HOME and ensure it is set.
    let cargo_home = match env::var("CARGO_HOME") {
        Ok(val) => PathBuf::from(val),
        Err(_) => {
            eprintln!();
            eprintln!("error: CARGO_HOME is not set for this build.");
            eprintln!("       ecosafety-core must be compiled with CARGO_HOME on `/mnt/oss`.");
            eprintln!();
            eprintln!("hint: export CARGO_HOME=/mnt/oss/cargo");
            eprintln!("      and ensure its `bin` dir is on PATH before running `cargo`.");
            hard_fail();
            return;
        }
    };

    // 3. Normalize CARGO_HOME path and verify it is under /mnt/oss.
    if !is_under(&cargo_home, oss_root) {
        eprintln!();
        eprintln!(
            "error: CARGO_HOME must reside under `/mnt/oss`, but is currently set to:\n       {}",
            cargo_home.display()
        );
        eprintln!();
        eprintln!("hint: export CARGO_HOME=/mnt/oss/cargo");
        eprintln!("      and re-run your build so toolchains and caches live on OSSFS.");
        hard_fail();
    }

    // Optional: emit a note so logs clearly show the bound paths.
    println!("cargo:warning=ecosafety-core preflight OK: CARGO_HOME={}", cargo_home.display());
}

/// Abort compilation with a non-zero exit code.
fn hard_fail() -> ! {
    // Using process::exit is standard for build scripts; cargo will surface
    // the error message written to stderr above.
    process::exit(1)
}

/// Return true if `child` is inside `parent` in the path hierarchy.
///
/// This function compares components rather than naive string prefixes to
/// avoid false positives like `/mnt/ossfs` being treated as under `/mnt/oss`.
fn is_under(child: &Path, parent: &Path) -> bool {
    // Early-out: parent must be absolute to make this check meaningful.
    if !parent.is_absolute() {
        return false;
    }

    // Canonicalize when possible, but don't fail the build if it errors;
    // fall back to the original path components.
    let parent_norm = canonical_or_clone(parent);
    let child_norm = canonical_or_clone(child);

    let mut parent_comps = parent_norm.components();
    let mut child_comps = child_norm.components();

    // Collect components into vectors for comparison.
    let parent_vec: Vec<_> = parent_comps.collect();
    let child_vec: Vec<_> = child_comps.collect();

    if parent_vec.len() > child_vec.len() {
        return false;
    }

    // Check that all parent components match the first N components of child.
    parent_vec
        .iter()
        .zip(child_vec.iter())
        .all(|(p, c)| p == c)
}

fn canonical_or_clone(p: &Path) -> PathBuf {
    match p.canonicalize() {
        Ok(c) => c,
        Err(_) => p.to_path_buf(),
    }
}
