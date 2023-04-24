use clap::Parser;
use anyhow::Result;
use std::env;

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
#[command(author, version, about, long_about)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Create a new mod.
    ///
    /// This will create a new git repository in the specified directory.
    /// The repository will have a `papermario` submodule, which will be set to the latest commit on the `main` branch.
    New(new::Args),

    /// Package a mod for distribution.
    ///
    /// Mods are distributed as `.merlon` files, which are encrypted, compressed tarballs of git patches.
    /// Merlon files are encrypted using the base ROM as the key.
    /// The patches are applied to the `papermario` submodule in the mod's directory.
    Pack(pack::Args),

    /// Apply a mod package to another mod.
    Apply(apply::Args),

    /// Run a mod in an emulator.
    Run(build::Args),

    /// Build a mod into a ROM.
    Build(build::Args),
}

#[cfg(feature = "gui")]
fn main() -> Result<()> {
    let is_gui = env::var("TERM").is_err() || matches!(env::var("MERLON_GUI"), Ok(v) if v == "1");

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
        match self.subcmd {
            SubCommand::New(new_args) => new::run(new_args),
            SubCommand::Pack(package_args) => pack::run(package_args),
            SubCommand::Apply(apply_args) => apply::run(apply_args),
            SubCommand::Run(run_args) => {
                let rom_path = build::build_mod(run_args)?;
                merlon::emulator::run_rom(&rom_path)?;
                Ok(())
            },
            SubCommand::Build(build_args) => {
                let rom_path = build::build_mod(build_args)?;
                println!("Output ROM: {}", rom_path.display());
                println!("You can run this ROM with `merlon run`.");
                println!("Warning: do not distribute this ROM. To distribute mods, use `merlon pack`.");
                Ok(())
            },
        }
    }
}
