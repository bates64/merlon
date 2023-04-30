#![cfg_attr(feature = "gui", windows_subsystem = "windows")]

use clap::Parser;
use anyhow::{Result, Context, bail};
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

    /// Run the current package in an emulator.
    Run(merlon::package::init::BuildRomOptions),

    /// Build the current package into a ROM.
    Build(merlon::package::init::BuildRomOptions),

    /// Update all dependencies, including packages and the decomp.
    Update,

    /// Add a dependency to the current package.
    Add(merlon::package::init::AddDependencyOptions),

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
            for warning in package.manifest()?.metadata().validate() {
                eprintln!("warning: {}", warning);
            }
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
                    // If the package is initialised, sync it so the patches dir updates
                    if InitialisedPackage::is_initialised(&package)? {
                        let initialised = InitialisedPackage::try_from(package.clone())?;
                        initialised.setup_git_branches()?;
                    }

                    let exported = package.export_distributable(export_args)?;
                    println!("Exported distributable: {}", exported);
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
                let rom = distributable.apply(apply_args.options)?;
                println!("Patched: {}", rom);
                Ok(())
            },
            SubCommand::Open(open_args) => {
                let distributable = Distributable::try_from(open_args.distributable)
                    .context("failed to open distributable file")?;
                let package = distributable.open_to_dir(open_args.options)
                    .context("failed to open distributable to package directory")?;
                println!("{}", package.copyright_notice()?);
                println!("Opened {} to directory {}", package, package.path().display());
                Ok(())
            },
            SubCommand::Run(build_args) => {
                if let Some(package) = package {
                    let initialised: InitialisedPackage = package.try_into()?;
                    let rom = initialised.build_rom(build_args)?;
                    merlon::emulator::run_rom(&rom)?;
                    Ok(())
                } else {
                    bail!("cannot run package: not in a package directory.");
                }
            },
            SubCommand::Build(build_args) => {
                if let Some(package) = package {
                    let initialised: InitialisedPackage = package.try_into()?;
                    let rom = initialised.build_rom(build_args)?;
                    println!("Built: {}", rom);
                    println!("You can run this ROM with `merlon run`.");
                    println!("Warning: do not distribute this ROM. To distribute this package, use `merlon export`.");
                    Ok(())
                } else {
                    bail!("cannot build package: not in a package directory.");
                }
            },
            SubCommand::Update => {
                if let Some(package) = package {
                    let initialised: InitialisedPackage = package.try_into()?;
                    initialised.update_decomp()?;
                    initialised.setup_git_branches()?;
                    Ok(())
                } else {
                    bail!("cannot update package: not in a package directory.");
                }
            }
            SubCommand::Add(add_args) => {
                if let Some(package) = package {
                    let mut initialised: InitialisedPackage = package.try_into()?;

                    // Make sure everything is OK to edit
                    if initialised.is_git_dirty()? {
                        bail!("papermario repo has uncommitted changes, please commit or stash them first");
                    }

                    // Add the dependency
                    let id = initialised.add_dependency(add_args)?;
                    let package = initialised.registry().get_or_error(id)?;
                    println!("Added dependency: {}", package);
                    initialised.setup_git_branches()
                        .context("failed to setup git branches with dependency, there might be a merge issue")
                } else {
                    bail!("cannot add dependency: not in a package directory.");
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
