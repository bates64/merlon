use clap::Parser;
use anyhow::Result;

mod new;
mod pack;
mod apply;
mod run;

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

    /// Apply a mod package.
    Apply(apply::Args),

    /// Run a mod in an emulator.
    Run(run::Args),
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.subcmd {
        SubCommand::New(new_args) => new::run(new_args),
        SubCommand::Pack(package_args) => pack::run(package_args),
        SubCommand::Apply(apply_args) => apply::run(apply_args),
        SubCommand::Run(run_args) => run::run(run_args),
    }
}
