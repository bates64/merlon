use std::io::prelude::*;
use std::{fs::File, path::Path, io::{BufReader, BufWriter}};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
pub use uuid::Uuid as Id; // note: implements Copy
pub use semver::{Version, VersionReq};

pub mod name;
use name::Name;

use super::Package;

// TODO: use taplo instead of toml to preserve comments etc

/// `merlon.toml` file. This file is used to store metadata about a mod.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Package metadata
    #[serde(rename = "package")]
    metadata: Metadata,

    /// Direct dependencies (not transitive)
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    id: Id,
    name: Name,
    version: Version,
    authors: Vec<String>,
    description: String,
    license: String,
    keywords: Vec<String>,
}

impl Metadata {
    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn version(&self) -> &Version {
        &self.version
    }

    pub fn set_version(&mut self, version: Version) {
        self.version = version;
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    /// Validate package metadata, returning a list of errors
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        // TODO: use newtypes for these, like Name
        // TODO: validate version
        if self.authors.is_empty() {
            errors.push("authors cannot be empty".to_owned());
        }
        if self.description.is_empty() {
            errors.push("description cannot be empty".to_owned());
        }
        if self.description.len() > 100 {
            errors.push("description must be less than 100 characters".to_owned());
        }
        if self.license.is_empty() {
            errors.push("license cannot be empty".to_owned());
        }
        // TODO: validate license
        for keyword in &self.keywords {
            const VALID_KEYWORDS: &[&str] = &["qol", "cheat", "bugfix", "cosmetic", "feature"];
            if !VALID_KEYWORDS.contains(&keyword.as_str()) {
                errors.push(format!("invalid keyword: {} (valid keywords: {:?})", keyword, VALID_KEYWORDS));
            }
        }
        errors
    }

    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    pub fn print_validation_warnings(&self) {
        for error in self.validate() {
            eprintln!("warning: {}", error);
        }
    }

    pub fn authors(&self) -> &Vec<String> {
        &self.authors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Dependency {
    /// Dependency on another Merlon package.
    Package {
        id: Id,
        version: VersionReq,
    },
    /// Dependency on the Paper Mario decompilation.
    Decomp {
        /// Git commit hash
        rev: String,
    },
}

impl From<&Metadata> for Dependency {
    fn from(metadata: &Metadata) -> Self {
        let version = metadata.version();
        Self::Package {
            id: metadata.id(),
            version: VersionReq {
                comparators: vec![
                    semver::Comparator {
                        op: semver::Op::Tilde,
                        major: version.major,
                        minor: Some(version.minor),
                        patch: Some(version.patch),
                        pre: version.pre.clone(),
                    }
                ]
            }
        }
    }
}

impl TryFrom<&Package> for Dependency {
    type Error = anyhow::Error;

    fn try_from(package: &Package) -> Result<Self> {
        let manifest = package.manifest()?;
        let metadata = manifest.metadata();
        Ok(metadata.into())
    }
}

impl Manifest {
    pub fn new(name: Name) -> Result<Self> {
        Ok(Self {
            metadata: Metadata {
                id: Id::new_v4(),
                name,
                version: "0.1.0".parse()?,
                authors: vec![get_author()?],
                description: "An amazing mod".to_owned(),
                license: "CC-BY-SA-4.0".to_owned(),
                keywords: vec![],
            },
            dependencies: vec![], // note: no Dependency::Decomp (init will add this)
        })
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    pub fn read_from_path(path: &Path) -> Result<Self> {
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

    pub fn declare_direct_dependency(&mut self, dependency: Dependency) -> Result<()> {
        match dependency {
            Dependency::Package { id, .. } => {
                if self.dependencies
                    .iter()
                    .any(|dep| matches!(dep, Dependency::Package { id: dep_id, .. } if id == *dep_id))
                {
                    bail!("dependency on package ID {} already declared", id);
                }
            }
            Dependency::Decomp { .. } => {
                if self.has_direct_decomp_dependency() {
                    // could also just update existing dependency?
                    bail!("dependency on decomp already declared");
                }
            }
        }
        self.dependencies.push(dependency);
        Ok(())
    }

    pub fn iter_direct_dependencies(&self) -> impl Iterator<Item = &Dependency> {
        self.dependencies.iter()
    }

    pub fn has_direct_decomp_dependency(&self) -> bool {
        self.dependencies.iter().any(|dep| matches!(dep, Dependency::Decomp { .. }))
    }
}

/// Get base commit of git submodule
fn get_base_commit(mod_path: &Path) -> Result<String> {
    let submodule_path = mod_path.join("papermario");
    let base_commit = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(&submodule_path)
        .output()?
        .stdout;
    let base_commit = String::from_utf8(base_commit)?;
    let base_commit = base_commit.trim().to_owned();
    Ok(base_commit)
}

/// Get author from git config as `name <email>`
fn get_author() -> Result<String> {
    let git_user_name = std::process::Command::new("git")
        .arg("config")
        .arg("user.name")
        .output()?
        .stdout;
    let git_user_name = String::from_utf8(git_user_name)?;
    let git_user_name = git_user_name.trim().to_owned();
    let git_user_email = std::process::Command::new("git")
        .arg("config")
        .arg("user.email")
        .output()?
        .stdout;
    let git_user_email = String::from_utf8(git_user_email)?;
    let git_user_email = git_user_email.trim().to_owned();

    Ok(format!("{} <{}>", git_user_name, git_user_email))
}
