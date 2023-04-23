use std::io::prelude::*;
use std::{fs::File, path::Path, io::{BufReader, BufWriter}};
use anyhow::Result;

use serde::{Deserialize, Serialize};

// TODO: use taplo instead of toml to preserve comments etc

/// `merlon.toml` file. This file is used to store metadata about a mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageConfig {
    pub base_commit: String,
}

impl PackageConfig {
    pub fn read_from_file(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut toml_string = String::new();
        reader.read_to_string(&mut toml_string)?;
        let config = toml::from_str(&toml_string)?;
        Ok(config)
    }

    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let toml_string = toml::to_string_pretty(self)?;
        writer.write_all(toml_string.as_bytes())?;
        Ok(())
    }

    pub fn default_for_mod(mod_path: &Path) -> Result<Self> {
        // Get base commit of git submodule
        let submodule_path = mod_path.join("papermario");
        let base_commit = std::process::Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(&submodule_path)
            .output()?
            .stdout;
        let base_commit = String::from_utf8(base_commit)?;
        let base_commit = base_commit.trim().to_owned();
    
        Ok(Self {
            base_commit,
        })
    }
}
