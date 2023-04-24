use std::{fs, process::Command};
use clap::Parser;
use anyhow::{Result, bail};
use merlon::baserom::Baserom;
use merlon::decomp_repo::LocalDecompRepo;
use merlon::package_config::PackageConfig;

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
    let mod_dir = std::env::current_dir()?.join(&args.name);
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

    // Add papermario as a git submodule
    let mut command = Command::new("git");
    command
        .arg("submodule")
        .arg("add")
        .arg("-b").arg("main");
    if let Some(repo) = local_decomp_repo.as_ref() {
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
    println!("Copying baserom...");
    let baserom_path = baserom.path();
    let baserom_copy_path = mod_dir.join("papermario/ver/us/baserom.z64");
    fs::copy(baserom_path, baserom_copy_path)?;

    // Copy template files
    fs::write(mod_dir.join(".gitignore"), include_str!("../templates/gitignore"))?;
    fs::create_dir(mod_dir.join(".vscode"))?;
    fs::write(mod_dir.join(".vscode/c_cpp_properties.json"), include_str!("../templates/.vscode/c_cpp_properties.json"))?;
    fs::write(mod_dir.join(".vscode/extensions.json"), include_str!("../templates/.vscode/extensions.json"))?;
    fs::write(mod_dir.join(".vscode/settings.json"), include_str!("../templates/.vscode/settings.json"))?;
    fs::write(mod_dir.join(".vscode/tasks.json"), include_str!("../templates/.vscode/tasks.json"))?;

    // Write merlon.toml
    println!("Creating merlon.toml...");
    let merlon_toml_path = mod_dir.join("merlon.toml");
    PackageConfig::default_for_mod(&mod_dir)?.write_to_file(&merlon_toml_path)?;

    // Create empty asset directory of the same name as the mod
    println!("Creating empty asset directory...");
    fs::create_dir_all(&mod_dir.join("assets").join(&args.name))?;

    // Run install script
    if inquire::Confirm::new("Run install.sh?").with_default(true).prompt()? {
        let status = Command::new("./install.sh")
            .current_dir(&mod_dir.join("papermario"))
            .status()?;
        if !status.success() {
            eprintln!("install.sh failed, you may need to run it manually.");
            eprintln!("If you see an error like 'Sorry, this is not a GIT repository', you can ignore it.");
        }
    }

    // Done!
    println!("");
    println!("Created mod: {:?}", mod_dir.file_stem().unwrap_or_default());
    println!("To build and run this mod, run the following commands:");
    println!("");
    println!("    cd {:?}", mod_dir);
    println!("    merlon run");
    println!("");

    Ok(())
}
