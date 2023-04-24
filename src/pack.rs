use std::path::{PathBuf, Path};
use std::{fs, process::Command};
use clap::Parser;
use anyhow::{Result, anyhow, bail};
use heck::AsKebabCase;
use merlon::package_config::PackageConfig;

#[derive(Parser, Debug)]
pub struct Args {
    /// Mod directory.
    ///
    /// Defaults to the current directory.
    #[clap(short, long)]
    mod_dir: Option<PathBuf>,

    /// The output file to write to.
    ///
    /// If not specified, the default is `MODNAME-YYYY-MM-DD.merlon`, where `MODNAME` is the name of the current directory.
    #[clap(short, long)]
    output: Option<PathBuf>,
}

pub fn run(args: Args) -> Result<()> {
    let mod_dir = get_and_check_mod_dir(args.mod_dir)?;
    let package_name = get_mod_package_name(&mod_dir)?;
    let submodule_dir = mod_dir.join("papermario");
    let config = PackageConfig::read_from_file(&mod_dir.join("merlon.toml"))?;

    let output_name = args.output
        .as_ref()
        .map(|path| path.file_stem().map(|stem| stem.to_string_lossy().to_string()))
        .unwrap_or_else(|| {
            let date = chrono::Local::now().format("%Y-%m-%d");
            Some(format!("{package_name}-{date}"))
        });

    if let Some(output_name) = output_name {
        let output_path = args.output.map(|p| Ok(p)).unwrap_or_else(|| {
            let path = std::env::current_dir();
            path.map(|mut path| {
                path.push(&output_name);
                path.set_extension("merlon");
                path
            })
        })?;

        // Warn if output filename is not a .merlon file
        if output_path.extension().unwrap_or_default() != "merlon" {
            eprintln!("warning: output filename does not end in .merlon");
        }

        // Output paths
        let output_dir = mod_dir.join(".merlon").join("packages").join(output_name);
        let patches_dir = output_dir.join("patches");
        let tar_path = output_dir.join("patches.tar.bz2");
        let encrypted_path = output_dir.join("patches.enc");
        fs::create_dir_all(&patches_dir)?;

        // Write changes since `main` to directory
        let status = Command::new("git")
            .arg("format-patch")
            .arg(format!("{}..HEAD", config.base_commit))
            .arg("-o").arg(&patches_dir.canonicalize()?)
            .arg("--minimal")
            .arg("--binary")
            .arg("--ignore-cr-at-eol")
            .arg("--function-context") // Maybe?
            .arg("--keep-subject")
            .arg("--no-merges")
            .arg("--no-stdout")
            .arg("--")
            .arg("src")
            .arg("include")
            .arg("assets") // Original assets should be gitignored
            .arg("ver/us")
            .current_dir(&submodule_dir)
            .status()?;
        if !status.success() {
            bail!("failed git format-patch to directory {}", patches_dir.display());
        }

        // TODO: Add a license into the tar, to protect the changes only

        // Compress patch directory into a tar
        let status = Command::new("tar")
            .arg("-cjvf")
            .arg(&tar_path)
            .arg(&patches_dir)
            .status()?;
        if !status.success() {
            bail!("failed to compress patches to tar {}", tar_path.display());
        }

        // Encrypt the tar using baserom as hash
        let status = Command::new("openssl")
            .arg("enc")
            .arg("-aes-256-cbc")
            .arg("-md").arg("sha512")
            .arg("-pbkdf2")
            .arg("-iter").arg("100000")
            .arg("-salt")
            .arg("-in").arg(&tar_path)
            .arg("-out").arg(&encrypted_path)
            .arg("-pass").arg(format!("file:{}", submodule_dir.join("ver/us/baserom.z64").display()))
            .status()?;
        if !status.success() {
            bail!("failed to encrypt tar to {}", encrypted_path.display());
        }

        // Copy to output path
        fs::copy(&encrypted_path, &output_path)?;
        println!("Wrote package to {}", output_path.display());
        Ok(())
    } else {
        bail!("output filename cannot be empty");
    }
}

/// Checks a mod directory is valid. If `mod_dir` is `None`, the current directory is used.
pub fn get_and_check_mod_dir(mod_dir: Option<PathBuf>) -> Result<PathBuf> {
    // TODO: use `git rev-parse --show-toplevel` to get the root of the git repo
    let mod_dir = mod_dir.map(|p| Ok(p)).unwrap_or_else(|| std::env::current_dir())?;

    // Check directory is a git repo
    let status = Command::new("git")
        .arg("rev-parse")
        .current_dir(&mod_dir)
        .status()?;
    if !status.success() {
        bail!("directory {:?} is not a git repository", mod_dir);
    }

    // Check directory has papermario submodule
    let status = Command::new("git")
        .arg("submodule")
        .arg("status")
        .arg("papermario")
        .current_dir(&mod_dir)
        .status()?;
    if !status.success() {
        bail!("directory {:?} does not have a papermario submodule", mod_dir);
    }

    // Check papermario submodule is up to date
    let status = Command::new("git")
        .arg("submodule")
        .arg("status")
        .arg("--cached")
        .arg("papermario")
        .current_dir(&mod_dir)
        .status()?;
    if !status.success() {
        eprintln!("warning: papermario submodule in directory {:?} is not up to date", mod_dir);
    }

    Ok(mod_dir)
}

pub fn get_mod_package_name(mod_dir: &Path) -> Result<String> {
    let name = mod_dir.file_name().ok_or_else(|| anyhow!("mod directory has no name"))?;
    let name = name.to_string_lossy();
    Ok(format!("{}", AsKebabCase(name)))
}
