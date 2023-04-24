use std::path::{PathBuf};
use std::{fs, process::Command};
use clap::Parser;
use anyhow::{Result, bail};
use merlon::mod_dir::ModDir;

#[derive(Parser, Debug)]
pub struct Args {
    /// Mod package to apply.
    input: PathBuf,
}

pub fn run(mod_dir: &mut ModDir, args: Args) -> Result<()> {
    let submodule_dir = mod_dir.submodule_dir();

    // 1. Decrypt
    // 2. Extract
    // 3. Apply patches

    let input_name = args.input.file_stem().map(|stem| stem.to_string_lossy().to_string());
    if let Some(input_name) = input_name {
        // Output paths
        let output_dir = mod_dir.path().join(".merlon").join("packages").join(&input_name);
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

        // List the tar
        Command::new("tar")
            .arg("-tvf")
            .arg(&tar_path)
            .status()?;

        // Decompress tar into patch directory
        let status = Command::new("tar")
            .arg("-xjvf")
            .arg(&tar_path)
            .arg("-C").arg(&output_dir)
            .arg("patches")
            .status()?;
        if !status.success() {
            bail!("failed to decompress {}", tar_path.display());
        }

        // Apply patches
        let mut patch_files = fs::read_dir(&patches_dir)?
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().map(|ext| ext == "patch").unwrap_or(false))
            .collect::<Vec<_>>();
        patch_files.sort_unstable();
        let status = Command::new("git")
            .arg("am")
            .arg("--3way")
            .args(patch_files.iter().map(|path| path.to_string_lossy().to_string()))
            .current_dir(&submodule_dir)
            .status()?;
        if !status.success() {
            bail!("failed to cleanly apply patches - run `cd papermario && git am --abort` to abort the merge");
        }

        Ok(())
    } else {
        bail!("invalid input filename");
    }
}
