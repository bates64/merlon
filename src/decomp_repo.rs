use std::{path::{Path, PathBuf}};

use anyhow::{Result, bail};
use inquire::{CustomType, error::InquireResult};

/// An existing clone of the decomp repo.
#[derive(Debug, Clone)]
pub struct LocalDecompRepo {
    path: PathBuf,
}

impl LocalDecompRepo {
    pub fn try_get() -> Result<Option<Self>> {
        // TODO: get from config file or env e.g. https://github.com/mehcode/config-rs/blob/master/examples/simple/main.rs
        Ok(Self::inquire()?)
    }

    pub fn inquire() -> InquireResult<Option<LocalDecompRepo>> {
        CustomType::<LocalDecompRepo>::new("Path to existing decomp repository:")
            .with_help_message("This is optional, press ESC to clone a new copy")
            .prompt_skippable()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Display for LocalDecompRepo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

impl std::str::FromStr for LocalDecompRepo {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(shellexpand::tilde(s).to_string());
        Self::try_from(path)
    }
}

impl TryFrom<PathBuf> for LocalDecompRepo {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        if !path.is_dir() {
            bail!("path is not a directory");
        }

        // TODO: some heuristic to check if it's a decomp clone?

        Ok(Self {
            path,
        })
    }
}
