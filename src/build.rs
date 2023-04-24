use std::path::PathBuf;
use std::process::Command;
use clap::Parser;
use anyhow::{Result, bail};

use crate::pack::get_and_check_mod_dir;

#[derive(Parser, Debug)]
pub struct Args {
    /// Mod directory.
    #[clap(short, long)]
    mod_dir: Option<PathBuf>,

    /// Whether to skip configuring (useful if you've already configured).
    #[clap(long)]
    skip_configure: bool,
}

pub fn build_mod(args: Args) -> Result<PathBuf> {
    let mod_dir = get_and_check_mod_dir(args.mod_dir)?;
    let submodule_dir = mod_dir.join("papermario");

    // Configure
    // TODO: only do this if we have to (i.e. file tree changed) - maybe ask git?
    if !args.skip_configure {
        let status = Command::new("./configure")
            //.arg("--non-matching")
            //.arg("--debug")
            .arg("--shift")
            .arg("us")
            .current_dir(&submodule_dir)
            .status()?;
        if !status.success() {
            bail!("failed to configure");
        }
    }

    // Build
    let status = Command::new("ninja")
        .current_dir(&submodule_dir)
        .status()?;
    if !status.success() {
        bail!("failed to build");
    }
    Ok(submodule_dir.join("ver/us/build/papermario.z64"))
}
