use std::{path::{Path, PathBuf}};

use anyhow::{Result, bail};
use inquire::{CustomType, error::InquireResult};

#[derive(Debug, Clone)]
pub struct Baserom {
    path: PathBuf,
}

impl Baserom {
    pub fn get() -> Result<Self> {
        // TODO: get from config file or env e.g. https://github.com/mehcode/config-rs/blob/master/examples/simple/main.rs
        Ok(Self::inquire()?)
    }

    pub fn inquire() -> InquireResult<Baserom> {
        CustomType::<Baserom>::new("Path to base ROM:")
            .with_error_message("Make sure the path exists, is Paper Mario (N64), USA release, is unmodified, and is in z64 format")
            .with_help_message("Provide a legally-obtained Paper Mario (N64) ROM")
            .prompt()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::fmt::Display for Baserom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Paper Mario (USA) <{}>", self.path.display())
    }
}

impl std::str::FromStr for Baserom {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path = PathBuf::from(shellexpand::tilde(s).to_string());
        Self::try_from(path)
    }
}

impl TryFrom<PathBuf> for Baserom {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            bail!("base ROM does not exist");
        }

        // TODO: check rom z64 and hash is USA
        // Can take code from ztar-rod?

        Ok(Self {
            path,
        })
    }
}
