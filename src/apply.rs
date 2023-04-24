use std::path::{PathBuf};
use std::{fs, process::Command};
use clap::Parser;
use anyhow::{Result, bail};
use merlon::package_config::PackageConfig;

use crate::pack::get_and_check_mod_dir;

#[derive(Parser, Debug)]
pub struct Args {
    /// Mod directory to apply to.
    ///
    /// Defaults to the current directory.
    #[clap(short, long)]
    mod_dir: Option<PathBuf>,

    /// Mod package to apply.
    input: PathBuf,
}

pub fn run(args: Args) -> Result<()> {
    let mod_dir = get_and_check_mod_dir(args.mod_dir)?;
    let submodule_dir = mod_dir.join("papermario");
    let _config = PackageConfig::read_from_file(&mod_dir.join("merlon.toml"))?;

    // 1. Decrypt
    // 2. Extract
    // 3. Apply patches

    let input_name = args.input.file_stem().map(|stem| stem.to_string_lossy().to_string());
    if let Some(input_name) = input_name {
        // Output paths
        let output_dir = mod_dir.join(".merlon").join("packages").join(&input_name);
        let patches_dir = output_dir.join("patches");
        let tar_path = output_dir.join("patches.tar.bz2");
        let encrypted_path = args.input;
        fs::create_dir_all(&patches_dir)?;

        // Decrypt tar using baserom as hash
        let status = Command::new("openssl")
            .arg("enc")
            .arg("-d") // decrypt
            .arg("-aes-256-cbc")
            .arg("-md").arg("sha512")
            .arg("-pbkdf2")
            .arg("-iter").arg("100000")
            .arg("-salt")
            .arg("-in").arg(&encrypted_path)
            .arg("-out").arg(&tar_path)
            .arg("-pass").arg(format!("file:{}", submodule_dir.join("ver/us/baserom.z64").display()))
            .status()?;
        if !status.success() {
            bail!("failed to decrypt {}", encrypted_path.display());
        }

        // Decompress tar into patch directory into a tar
        let status = Command::new("tar")
            .arg("-cjvf")
            .arg(&tar_path)
            .arg(&patches_dir)
            .status()?;
        if !status.success() {
            bail!("failed to decompress {}", tar_path.display());
        }

        // Apply patches
        let status = Command::new("git")
            .arg("am")
            //.arg("--3way")
            //.arg("--ignore-whitespace")
            //.arg("--whitespace=nowarn")
            .arg(&patches_dir)
            .current_dir(&submodule_dir)
            .status()?;
        if !status.success() {
            bail!("failed to cleanly apply patches - run `git am --abort` to abort the merge");
        }
        println!("Applied patches from {}", &input_name);
        Ok(())
    } else {
        bail!("invalid input filename");
    }
}
