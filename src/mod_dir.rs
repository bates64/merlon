use std::{path::{Path, PathBuf}, process::{Command, Stdio}};
use anyhow::{Result, anyhow, bail};
use heck::AsKebabCase;

use crate::package_config::Config;

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

    pub fn kebab_case_name(&self) -> Result<String> {
        let name = self.path().file_name().ok_or_else(|| anyhow!("mod directory has no name"))?;
        let name = name.to_string_lossy();
        Ok(format!("{}", AsKebabCase(name)))
    }

    pub fn config_path(&self) -> PathBuf {
        self.path().join("merlon.toml")
    }

    pub fn config(&self) -> Result<Config> {
        Config::read_from_file(&self.config_path())
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

/// Finds the mod's git root, starting from the current directory.
/// If within a submodule, returns the root of the parent repo.
/// Otherwise, returns the root of the current repo.
/// If not in a repo at all, returns an error.
fn git_root() -> Result<PathBuf> {
    // If we're in a submodule, --show-superproject-working-tree will give us the parent repo
    let git_root = Command::new("git")
        .arg("rev-parse")
        .arg("--show-superproject-working-tree")
        .output()?
        .stdout;
    let git_root = String::from_utf8(git_root)?;
    if !git_root.is_empty() {
        let git_root = Path::new(git_root.trim()).canonicalize()?;
        return Ok(git_root);
    }

    // If it returned nothing, we're not in a submodule, so we can just use --show-toplevel
    let git_root = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()?
        .stdout;
    let git_root = String::from_utf8(git_root)?;
    if !git_root.is_empty() {
        let git_root = Path::new(git_root.trim()).canonicalize()?;
        return Ok(git_root);
    }

    bail!("not in a git repository");
}
