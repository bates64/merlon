use std::{fs, process::Command};
use clap::Parser;
use anyhow::{Result, bail};
use merlon::baserom::Baserom;
use merlon::decomp_repo::LocalDecompRepo;

/// Creates a new mod.
#[derive(Parser, Debug)]
pub struct Args {
    /// The name of the mod. This will be used as the mod's directory name.
    /// It is recommended that mods be named in the snake-case format.
    name: String,
}

pub fn run(args: Args) -> Result<()> {
    // TODO: load from config file with `config` crate

    // Form
    let baserom = Baserom::get()?;
    let local_decomp_repo = LocalDecompRepo::try_get()?;

    // Create mod dir
    let mod_dir = std::env::current_dir()?.join(args.name);
    if mod_dir.exists() {
        bail!("directory {:?} already exists", mod_dir);
    }
    fs::create_dir(&mod_dir)?;

    // Initialise git repo
    let status = Command::new("git")
        .arg("init")
        .current_dir(&mod_dir)
        .status()?;
    if !status.success() {
        bail!("failed to initialise git repo");
    }

    // Add papermario submodule
    // branch: main
    // commit: 90656fea19ea62412ade3602db78ccdd4d73eb70
    // url: https://github.com/pmret/papermario.git
    // path: papermario
    let mut command = Command::new("git");
    command
        .arg("submodule")
        .arg("add")
        .arg("-b").arg("main");
    if let Some(repo) = local_decomp_repo {
        command.arg("--reference").arg(repo.path());
    }
    command
        .arg("https://github.com/pmret/papermario.git")
        .arg("papermario")
        .current_dir(&mod_dir);
    if !command.status()?.success() {
        bail!("failed to add papermario submodule");
    }

    // Copy baserom
    let baserom_path = baserom.path();
    let baserom_copy_path = mod_dir.join("papermario/ver/us/baserom.z64");
    fs::copy(baserom_path, baserom_copy_path)?;

    // Done
    println!("Created mod directory {:?}", mod_dir);

    Ok(())
}
