//! Package manifests are stored in `merlon.toml` files.
//! 
//! The manifest format is loosely inspired by Cargo's `Cargo.toml` format.

use std::io::prelude::*;
use std::{fs::File, path::Path, io::{BufReader, BufWriter}};
use anyhow::{Result, bail};
use pyo3::exceptions::PyValueError;
use pyo3::types::PyDict;
use serde::{Deserialize, Serialize};
pub use semver::{Version, VersionReq};
use pyo3::prelude::*;

/// Validated package name utilities
pub mod name;
use name::Name;

mod id;
pub use id::Id;

use super::Package;

// TODO: use taplo instead of toml to preserve comments etc

/// Package manifest data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass(module = "merlon.package.manifest")]
pub struct Manifest {
    /// Package metadata
    #[serde(rename = "package")]
    metadata: Metadata,

    /// Direct dependencies (not transitive)
    dependencies: Vec<Dependency>,
}

/// Metadata about a package. Corresponds to the `[package]` section in `merlon.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[pyclass(module = "merlon.package.manifest")]
pub struct Metadata {
    id: Id,
    name: Name,
    version: Version,
    authors: Vec<String>,
    description: String,
    license: String,
    keywords: Vec<String>,
}

#[pymethods]
impl Metadata {
    /// The package ID.
    #[getter]
    pub fn id(&self) -> Id {
        self.id
    }

    /// The package name.
    #[getter]
    fn get_name(&self) -> Name {
        self.name.clone()
    }

    /// The package version.
    #[getter]
    fn get_version(&self) -> String {
        self.version.to_string()
    }

    /// Set the package version. Must be a valid semver version (e.g. `1.0.0-rc1`).
    #[setter(version)]
    fn py_set_version(&mut self, version: String) -> Result<()> {
        self.version = version.parse()?;
        Ok(())
    }

    /// The package one-line description.
    #[getter]
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

    /// Returns whether the package metadata is valid.
    pub fn is_valid(&self) -> bool {
        self.validate().is_empty()
    }

    /// The package authors.
    #[getter]
    fn get_authors(&self) -> Vec<String> {
        self.authors.clone()
    }
}

impl Metadata {
    /// Returns the package name.
    pub fn name(&self) -> &Name {
        &self.name
    }

    /// Returns the package version.
    pub fn version(&self) -> &Version {
        &self.version
    }

    /// Updates the package version.
    pub fn set_version(&mut self, version: Version) {
        self.version = version;
    }

    /// Returns the package authors.
    pub fn authors(&self) -> &Vec<String> {
        &self.authors
    }

    /// Prints validation warnings to stderr.
    #[deprecated(since = "1.1.0", note = "iterate over validate() instead")]
    pub fn print_validation_warnings(&self) {
        for error in self.validate() {
            eprintln!("warning: {}", error);
        }
    }
}

/// A dependency description. Corresponds to values of the `[[dependencies]]` list in `merlon.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Dependency {
    /// Dependency on another Merlon package.
    Package {
        /// The ID of the dependency package.
        id: Id,

        /// The semantic version requirement for the dependency.
        /// See https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html
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

impl ToPyObject for Dependency {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        match self {
            Self::Package { id, version } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "package").unwrap();
                dict.set_item("id", id.to_string()).unwrap();
                dict.set_item("version", version.to_string()).unwrap();
                dict.into()
            }
            Self::Decomp { rev } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "decomp").unwrap();
                dict.set_item("rev", rev).unwrap();
                dict.into()
            }
        }
    }
}

impl FromPyObject<'_> for Dependency {
    fn extract(ob: &PyAny) -> PyResult<Self> {
        let dict = ob.downcast::<PyDict>()?;
        let type_: &str = dict.get_item("type")
            .ok_or(PyValueError::new_err("missing dependency type"))?
            .extract()?;
        match type_ {
            "package" => {
                let id: Id = dict.get_item("id")
                    .ok_or(PyValueError::new_err("missing dependency id"))?
                    .extract()?;
                let version: String = dict.get_item("version")
                    .ok_or(PyValueError::new_err("missing dependency version"))?
                    .extract()?;
                let version: VersionReq = version.parse()
                    .map_err(|e| PyValueError::new_err(format!("invalid dependency version: {}", e)))?;
                Ok(Self::Package { id, version })
            }
            "decomp" => {
                let rev: String = dict.get_item("rev")
                    .ok_or(PyValueError::new_err("missing dependency rev"))?
                    .extract()?;
                Ok(Self::Decomp { rev })
            }
            _ => Err(PyValueError::new_err(format!("invalid dependency type: {}", type_))),
        }
    }
}

impl IntoPy<PyObject> for Dependency {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}

#[pymethods]
impl Manifest {
    /// Creates a new manifest for a package with the given name.
    #[new]
    pub fn new(name: Name) -> Result<Self> {
        Ok(Self {
            metadata: Metadata {
                id: Id::new(),
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

    /// Package metadata.
    #[getter]
    fn get_metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    /// Updates the package metadata.
    #[setter]
    pub fn set_metadata(&mut self, metadata: Metadata) {
        self.metadata = metadata;
    }
}

impl Manifest {
    /// Borrows the package metadata.
    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    /// Mutably borrows the package metadata.
    pub fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }

    /// Reads a manifest from a file. Typically, manifest files are named `merlon.toml`.
    pub fn read_from_path(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut toml_string = String::new();
        reader.read_to_string(&mut toml_string)?;
        let config = toml::from_str(&toml_string)?;
        Ok(config)
    }

    /// Writes a manifest to a file.
    pub fn write_to_file(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let toml_string = toml::to_string_pretty(self)?;
        writer.write_all(toml_string.as_bytes())?;
        Ok(())
    }

    /// Adds a dependency to the manifest.
    /// If the dependency is declared already but with a different version/revision, errors.
    pub fn declare_direct_dependency(&mut self, dependency: Dependency) -> Result<()> {
        match &dependency {
            Dependency::Package { id, version } => {
                if let Some(Dependency::Package { version: existing_version, .. }) = self.dependencies
                    .iter_mut()
                    .find(|dep| matches!(dep, Dependency::Package { id: dep_id, .. } if *id == *dep_id))
                {
                    // TODO: if existing version <= version, update it
                    if *existing_version != *version {
                        bail!("dependency on package ID {} already declared with incompatible version", id);
                    }
                    return Ok(());
                }
            }
            Dependency::Decomp { rev } => {
                if let Some(Dependency::Decomp { rev: existing_rev, .. }) = self.dependencies
                    .iter_mut()
                    .find(|dep| matches!(dep, Dependency::Decomp { .. }))
                {
                    // TODO: if existing rev <= rev, update it
                    if *existing_rev != *rev {
                        bail!("dependency on decomp already declared with incompatible revision");
                    }
                    return Ok(());
                }
            }
        }
        self.dependencies.push(dependency);
        Ok(())
    }

    /// Iterates over the dependencies that are declared in the manifest.
    pub fn iter_direct_dependencies(&self) -> impl Iterator<Item = &Dependency> {
        self.dependencies.iter()
    }

    /// Returns true if the manifest has a decomp-type dependency.
    pub fn has_direct_decomp_dependency(&self) -> bool {
        self.dependencies.iter().any(|dep| matches!(dep, Dependency::Decomp { .. }))
    }

    /// Adds a Dependency::Decomp dependency if one does not already exist.
    /// If it does exist, updates it.
    pub fn upsert_decomp_dependency(&mut self, rev: String) -> Result<()> {
        if let Some(dep) = self.dependencies.iter_mut().find(|dep| matches!(dep, Dependency::Decomp { .. })) {
            if let Dependency::Decomp { rev: existing_rev } = dep {
                *existing_rev = rev;
                return Ok(());
            }
        }
        self.declare_direct_dependency(Dependency::Decomp { rev })
    }

    /// Returns the Git revision (commit hash) of the decomp dependency, if one exists.
    pub fn get_direct_decomp_dependency_rev(&self) -> Option<&str> {
        if let Some(dep) = self.dependencies.iter().find(|dep| matches!(dep, Dependency::Decomp { .. })) {
            if let Dependency::Decomp { rev } = dep {
                return Some(rev);
            }
        }
        None
    }
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
