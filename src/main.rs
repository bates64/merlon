#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

use clap::Parser;
use anyhow::{Result, bail};
use merlon::mod_dir::ModDir;
use std::path::PathBuf;

mod new;
mod pack;
mod apply;
mod build;

/// Mod manager for the Paper Mario (N64) decompilation.
/// 
/// Merlon allows you to create mods that can be applied to the decomp source code, and to package mods
/// into `.merlon` files that can be applied to a copy of the decomp source code.
///
/// For assistance with Merlon, join the #merlon channel of the Paper Mario Modding Discord server:
///
/// https://discord.gg/paper-mario-modding-279322074412089344
/// https://discord.com/channels/279322074412089344/1099844075399696394
/// 
/// Copyright 2023 Alex Bates
///
/// This Executable Form is subject to the terms of the Mozilla Public
/// License, v. 2.0. If a copy of the MPL was not distributed with this
/// program, You can obtain one at https://mozilla.org/MPL/2.0/.
#[derive(Parser, Debug)]
#[command(name = "Merlon", author, version, about, long_about)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,

    /// The directory of the mod to operate on.
    /// 
    /// Defaults the current git repository directory, or the current directory if not in a git repository.
    /// Merlon mod directories can be identified by the presence of a `merlon.toml` file.
    #[arg(short, long)]
    directory: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Create a new mod.
    ///
    /// This will create a new git repository in the specified directory.
    /// The repository will have a `papermario` submodule, which will be set to the latest commit on the `main` branch.
    New(new::Args),

    /// Package current mod for distribution.
    ///
    /// Mods are distributed as `.merlon` files, which are encrypted, compressed tarballs of git patches.
    /// Merlon files are encrypted using the base ROM as the key.
    /// The patches are applied to the `papermario` submodule in the mod's directory.
    Pack(pack::Args),

    /// Apply a mod package to the current mod.
    Apply(apply::Args),

    /// Run current mod in an emulator.
    Run(build::Args),

    /// Build current mod into a ROM.
    Build(build::Args),

    /// Launch the GUI.
    #[cfg(feature = "gui")]
    Gui,
}

#[cfg(feature = "gui")]
fn main() -> Result<()> {
    // If TERM is not set, or MERLON_GUI=1, run the GUI.
    let is_gui = std::env::var("TERM").is_err() || matches!(std::env::var("MERLON_GUI"), Ok(v) if v == "1");

    if is_gui {
        main_gui()
    } else {
        main_cli()
    }
}

#[cfg(not(feature = "gui"))]
fn main() -> Result<()> {
    main_cli()
}

fn main_cli() -> Result<()> {
    let args = Args::parse();
    args.run()
}

#[cfg(feature = "gui")]
fn main_gui() -> Result<()> {
    use klask::Settings;

    klask::run_derived::<Args, _>(Settings::default(), |args| {
        if let Err(error) = args.run() {
            // TODO: better error handling, e.g. nativefiledialog
            eprintln!("{}", error);
            std::process::exit(1);
        }
    });
    Ok(())
}

impl Args {
    pub fn run(self) -> Result<()> {
        // Get mod directory from args, or current directory if not specified.
        let mut mod_dir = if let Some(directory) = self.directory {
            // If a directory is provided and its invalid, error.
            Some(ModDir::try_from(directory)?)
        } else {
            // Otherwise, try to find the current mod directory, but it's OK if its None.
            ModDir::current().ok()
        };

        if let Some(mod_dir) = &mut mod_dir {
            // TODO: make sure the submodule is up to date.
            mod_dir.config()?.package.print_validation_warnings();
        }

        // Run subcommand.
        match self.subcmd {
            SubCommand::New(new_args) => {
                if let Some(mod_dir) = &mut mod_dir {
                    bail!("cannot create new mod: already in a mod directory: {}", mod_dir.path().display());
                } else {
                    new::run(new_args)
                }
            },
            SubCommand::Pack(package_args) => {
                if let Some(mod_dir) = &mut mod_dir {
                    pack::run(mod_dir, package_args)
                } else {
                    bail!("cannot package mod: not in a mod directory.");
                }
            },
            SubCommand::Apply(apply_args) => {
                if let Some(mod_dir) = &mut mod_dir {
                    apply::run(mod_dir, apply_args)
                } else {
                    bail!("cannot apply mod: not in a mod directory.");
                }
            },
            SubCommand::Run(run_args) => {
                if let Some(mod_dir) = &mut mod_dir {
                    let rom_path = build::build_mod(mod_dir, run_args)?;
                    merlon::emulator::run_rom(&rom_path)?;
                    Ok(())
                } else {
                    bail!("cannot run mod: not in a mod directory.");
                }
            },
            SubCommand::Build(build_args) => {
                if let Some(mod_dir) = &mut mod_dir {
                    let rom_path = build::build_mod(mod_dir, build_args)?;
                    println!("Output ROM: {}", rom_path.display());
                    println!("You can run this ROM with `merlon run`.");
                    println!("Warning: do not distribute this ROM. To distribute mods, use `merlon pack`.");
                    Ok(())
                } else {
                    bail!("cannot build mod: not in a mod directory.");
                }
            },
            #[cfg(feature = "gui")]
            SubCommand::Gui => main_gui(),
        }
    }
}
