use std::{path::{Path, PathBuf}, process::{Command, Stdio}};
use anyhow::{Result, bail};

use crate::package::Manifest;

/// Mod directory.
#[derive(Debug, Clone)]
pub struct ModDir {
    path: PathBuf,
}

impl ModDir {
    /// Gets the mod directory from the current directory.
    pub fn current() -> Result<Self> {
        git_root()?.try_into()
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn submodule_dir(&self) -> PathBuf {
        self.path().join("papermario")
    }

    pub fn us_baserom_path(&self) -> PathBuf {
        self.submodule_dir().join("ver/us/baserom.z64")
    }

    pub fn config_path(&self) -> PathBuf {
        self.path().join("merlon.toml")
    }

    pub fn config(&self) -> Result<Manifest> {
        Manifest::read_from_file(&self.config_path())
    }
}

/// Checks a path is valid mod directory.
impl TryFrom<PathBuf> for ModDir {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        // Check directory is a git repo
        let status = Command::new("git")
            .arg("rev-parse")
            .current_dir(&path)
            .status()?;
        if !status.success() {
            bail!("directory {:?} is not a git repository", path);
        }

        // Check directory has papermario submodule
        let status = Command::new("git")
            .arg("submodule")
            .arg("status")
            .arg("papermario")
            .current_dir(&path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
        if !status.success() {
            bail!("directory {:?} does not have a papermario submodule", path);
        }

        // Check papermario submodule is up to date
        let status = Command::new("git")
            .arg("submodule")
            .arg("status")
            .arg("--cached")
            .arg("papermario")
            .current_dir(&path)
            .stdout(Stdio::null())
            .status()?;
        if !status.success() {
            eprintln!("warning: papermario submodule in directory {:?} is not up to date", path);
        }

        // Check directory has merlon.toml
        if !path.join("merlon.toml").exists() {
            bail!("directory {:?} does not have a merlon.toml file", path);
        }

        Ok(ModDir {
            path,
        })
    }
}
