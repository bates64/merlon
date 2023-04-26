#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

use clap::Parser;
use anyhow::{Result, bail};
use merlon::package::{Package, InitialisedPackage, Distributable};
use std::path::PathBuf;

mod new;

/// Mod package manager for the Paper Mario (N64) decompilation.
/// 
/// Merlon allows you to create mod packages by editing the decomp source code, and to export packages
/// into distributable `.merlon` patch files that can be applied to a base ROM with `merlon apply`.
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

    /// The directory of the package to operate on.
    /// 
    /// If not set, a `merlon.toml` file will be searched for in the current directory and its parents.
    #[arg(short, long)]
    directory: Option<PathBuf>,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Create a new package.
    New(new::Args),

    /// Initialise this package for editing and building.
    Init(merlon::package::init::InitialiseOptions),

    /// Export this package as a `.merlon` file for distribution.
    Export(merlon::package::distribute::ExportOptions),

    /// Apply a distributable to a base ROM.
    Apply(ApplyArgs),

    /// Open a distributable's source code.
    Open(OpenArgs),

    /// Run current mod in an emulator.
    Run(merlon::package::init::BuildRomOptions),

    /// Build current mod into a ROM.
    Build(merlon::package::init::BuildRomOptions),

    /// Launch the GUI.
    #[cfg(feature = "gui")]
    Gui,
}

#[derive(Parser, Debug)]
struct ApplyArgs {
    #[clap(flatten)]
    pub options: merlon::package::distribute::ApplyOptions,

    pub distributable: PathBuf,
}

#[derive(Parser, Debug)]
struct OpenArgs {
    #[clap(flatten)]
    pub options: merlon::package::distribute::OpenOptions,

    pub distributable: PathBuf,
}

#[cfg(feature = "gui")]
fn main() -> Result<()> {
    pretty_env_logger::init();

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
    pretty_env_logger::init();
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
        // Get package from args, or current directory if not specified.
        let mut package = if let Some(directory) = self.directory.as_ref() {
            // If a directory is provided and its invalid, error.
            Some(Package::try_from(directory.clone())?)
        } else {
            // Otherwise, try to find the current package directory
            Package::current()?
        };

        if let Some(package) = &mut package {
            package.manifest()?.metadata().print_validation_warnings();
        }

        // Run subcommand.
        match self.subcmd {
            SubCommand::New(new_args) => {
                if let Some(package) = &mut package {
                    bail!("cannot create new package: already in a package: {}", package);
                } else {
                    new::run(self.directory, new_args)
                }
            },
            SubCommand::Init(init_args) => {
                if let Some(package) = package {
                    InitialisedPackage::initialise(package, init_args)?;
                    Ok(())
                } else {
                    bail!("cannot initialise package: not in a package directory.");
                }
            },
            SubCommand::Export(export_args) => {
                if let Some(package) = package {
                    let exported = package.export_distributable(export_args)?;
                    println!("Exported package: {}", exported);
                    Ok(())
                } else {
                    bail!("cannot export package: not in a package directory.");
                }
            },
            SubCommand::Apply(apply_args) => {
                let distributable = Distributable::try_from(apply_args.distributable)?;
                distributable.open_scoped(apply_args.options.baserom.clone(), |package| {
                    println!("{}", package.copyright_notice()?);
                    Ok(())
                })?;
                let rom_path = distributable.apply(apply_args.options)?;
                println!("Patched ROM: {}", rom_path.display());
                Ok(())
            },
            SubCommand::Open(open_args) => {
                let distributable = Distributable::try_from(open_args.distributable)?;
                let package = distributable.open_to_dir(open_args.options)?;
                println!("{}", package.copyright_notice()?);
                println!("Opened {} to directory {}", package, package.path().display());
                Ok(())
            },
            SubCommand::Run(build_args) => {
                if let Some(package) = package {
                    let initialised: InitialisedPackage = package.try_into()?;
                    let rom_path = initialised.build_rom(build_args)?;
                    merlon::emulator::run_rom(&rom_path)?;
                    Ok(())
                } else {
                    bail!("cannot run package: not in a package directory.");
                }
            },
            SubCommand::Build(build_args) => {
                if let Some(package) = package {
                    let initialised: InitialisedPackage = package.try_into()?;
                    let rom_path = initialised.build_rom(build_args)?;
                    println!("Output ROM: {}", rom_path.display());
                    println!("You can run this ROM with `merlon run`.");
                    println!("Warning: do not distribute this ROM. To distribute mods, use `merlon pack`.");
                    Ok(())
                } else {
                    bail!("cannot build package: not in a mod directory.");
                }
            },
            #[cfg(feature = "gui")]
            SubCommand::Gui => main_gui(),
        }
    }
}

#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Args::command().debug_assert()
}
