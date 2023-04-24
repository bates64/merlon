use std::path::PathBuf;
use std::process::Command;
use clap::Parser;
use anyhow::{Result, bail};

use crate::pack::get_and_check_mod_dir;

#[derive(Parser, Debug)]
pub struct Args {
    /// Mod directory.
    #[arg(short, long)]
    mod_dir: Option<PathBuf>,

    /// Whether to skip configuring (useful if you've already configured).
    #[arg(long)]
    skip_configure: bool,

    /// Path to output ROM to.
    #[arg(short, long)]
    output: Option<PathBuf>,
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

    // Copy output file if needed
    let rom = submodule_dir.join("ver/us/build/papermario.z64");
    if let Some(output) = args.output {
        std::fs::copy(rom, &output)?;
        Ok(output.into())
    } else {
        Ok(rom)
    }
}
