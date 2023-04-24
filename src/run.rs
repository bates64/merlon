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

pub fn run(args: Args) -> Result<()> {
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

    // Run
    let emulator = find_emulator()?;
    Command::new(emulator)
        .arg("ver/us/build/papermario.z64")
        .current_dir(&submodule_dir)
        .status()?;

    Ok(())
}

pub fn find_emulator() -> Result<PathBuf> {
    const EMULATOR_PATHS: &[&str] = &[
        "/usr/bin/cen64",
        "/usr/bin/ares",
        "/Applications/ares.app/Contents/MacOS/ares",
        "/usr/bin/mupen64plus",
        "/usr/bin/retroarch",
        "C:\\Program Files (x86)\\Project64 2.3\\Project64.exe",
    ];

    for path in EMULATOR_PATHS {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    bail!("no known emulator installed");
}
