//! Utilities for interfacing with N64 emulators.

use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, bail};
use pyo3::prelude::*;

use crate::rom::Rom;

/// Runs the given ROM in an emulator.
#[pyfunction]
pub fn run_rom(rom: &Rom) -> Result<()> {
    let emulator = find_emulator()?;
    Command::new(emulator)
        .arg(rom.path())
        .status()
        .map(|_| ())
        .map_err(Into::into)
}

fn find_emulator() -> Result<PathBuf> {
    const EMULATOR_PATHS: &[&str] = &[
        "/usr/bin/cen64",
        "/usr/bin/ares",
        "/Applications/ares.app/Contents/MacOS/ares",
        "/usr/bin/mupen64plus",
        "/usr/bin/retroarch",
        "C:\\Program Files (x86)\\Project64 2.3\\Project64.exe",
        "C:\\Program Files (x86)\\Project64 3.0\\Project64.exe",
    ];

    for path in EMULATOR_PATHS {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    bail!("no known emulator installed");
}
