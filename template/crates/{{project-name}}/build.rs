//! Embeds build provenance (git sha, describe, timestamp, rustc) as `VERGEN_*` env vars.
//! Outside a git checkout the values fall back to a sentinel that `build_info` maps to
//! `unknown`, so the build never fails because of missing history.

use vergen_gitcl::{Build, Emitter, Gitcl, Rustc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Emitter::default()
        .default_on_error()
        .add_instructions(&Build::all_build())?
        .add_instructions(&Gitcl::all_git())?
        .add_instructions(&Rustc::all_rustc())?
        .emit()?;
    Ok(())
}
