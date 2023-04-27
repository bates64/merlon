//! Merlon package management.

/// Versioned root directory name. Can bump this if we ever need to change the directory structure.
const ROOT_DIR_NAME: &str = "merlon_v1";

const MANIFEST_FILE_NAME: &str = "merlon.toml";
const README_FILE_NAME: &str = "README.md";
const LICENSE_FILE_NAME: &str = "LICENSE";
const PATCHES_DIR_NAME: &str = "patches";

use std::{
    fs,
    process::Command,
    path::{Path, PathBuf},
    fmt::{self, Display, Formatter},
    hash::Hash,
    cmp::{Eq, PartialEq},
    io::prelude::*,
};
use anyhow::{Result, bail};
use pyo3::prelude::*;

pub mod manifest;
pub use manifest::{
    Manifest,
    Id,
    name::Name,
};

mod registry;
pub use registry::Registry;

pub mod init;
pub use init::InitialisedPackage;

pub mod distribute;
pub use distribute::Distributable;

/// Returns true if the given directory is probably a Merlon package.
pub fn is_unexported_package(path: &Path) -> bool {
    path.is_dir() && path.join(MANIFEST_FILE_NAME).is_file()
}

/// A package in the form of a directory.
#[derive(Debug, Clone)]
#[pyclass(module = "merlon.package")]
pub struct Package {
    path: PathBuf,
}

impl TryFrom<PathBuf> for Package {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self> {
        if is_unexported_package(&path) {
            Ok(Self { path })
        } else {
            bail!("{} is not an unexported Merlon package", path.display());
        }
    }
}

impl Package {
    /// Create a new package at the given path. The path must not exist.
    pub fn new<N>(name: N, path: PathBuf) -> Result<Self>
    where
        N: manifest::name::TryIntoName,
    {
        let name: Result<Name> = name.try_into_name().map_err(Into::into);
        let name = name?;

        if path.exists() {
            bail!("{} already exists", path.display());
        }

        let path_clone = path.clone();

        let create_package = || {
            fs::create_dir(&path)?;
            fs::create_dir(&path.join(PATCHES_DIR_NAME))?;
            fs::write(&path.join(README_FILE_NAME), generate_readme(&name))?;
            let manifest = manifest::Manifest::new(name)?;
            manifest.write_to_file(&path.join(MANIFEST_FILE_NAME))?;
            fs::write(&path.join(LICENSE_FILE_NAME), generate_license(&manifest))?;

            debug_assert!(Package::try_from(path.clone()).is_ok());
            Ok(Self { path })
        };

        // If it errors, delete the directory
        match create_package() {
            Err(e) => {
                let _ = fs::remove_dir_all(path_clone);
                Err(e)
            }
            result => result,
        }
    }

    /// Edit the package manifest. The given function will be called with a mutable reference to the manifest,
    /// and after the function returns the manifest will be written back to disk.
    pub fn edit_manifest<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut Manifest) -> Result<()>,
    {
        let path = self.path.join(MANIFEST_FILE_NAME);
        let mut manifest = self.manifest()?;
        f(&mut manifest)?;
        manifest.write_to_file(&path)
    }
}

#[pymethods]
impl Package {
    /// Creates a new package at the given path. The path must not exist.
    #[new]
    fn py_new(name: Name, path: PathBuf) -> Result<Self> {
        Self::new(name, path)
    }

    /// Gets the current package, if any, by looking for `merlon.toml` in the current directory and its parents.
    #[staticmethod]
    pub fn current() -> Result<Option<Self>> {
        let mut dir = std::env::current_dir()?;
        while !dir.join(MANIFEST_FILE_NAME).is_file() {
            if !dir.pop() {
                return Ok(None);
            }
        }
        Self::try_from(dir).map(|pkg| Some(pkg))
    }

    /// The package ID.
    #[getter]
    pub fn id(&self) -> Result<Id> {
        Ok(self.manifest()?.metadata().id().clone())
    }

    /// The package path.
    #[getter]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the text content of the README.md file in the package.
    pub fn readme(&self) -> Result<String> {
        fs::read_to_string(self.path.join(README_FILE_NAME)).map_err(Into::into)
    }

    /// Returns the manifest of the package by parsing the `merlon.toml` file.
    pub fn manifest(&self) -> Result<Manifest> {
        let path = self.path.join(MANIFEST_FILE_NAME);
        Manifest::read_from_path(&path)
            .map_err(|err| err.context(format!(
                "Error reading package manifest {}",
                &path.display(),
            )))
    }

    /// Compares two packages by ID.
    pub fn uuid_equals(&self, other: &Package) -> Result<bool> {
        Ok(self.manifest()?.metadata().id() == other.manifest()?.metadata().id())
    }

    /// Returns a copyright notice for this package by reading the package's `LICENSE` file.
    pub fn copyright_notice(&self) -> Result<String> {
        let mut notice = String::new();
        let mut file = fs::File::open(self.path.join(LICENSE_FILE_NAME))?;
        file.read_to_string(&mut notice)?;
        Ok(notice)
    }
}

impl Package {
    pub(crate) fn apply_patches_to_decomp_repo(&self, repo: &Path) -> Result<()> {
        let mut patch_files = fs::read_dir(&self.path.join(PATCHES_DIR_NAME))?
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().map(|ext| ext == "patch").unwrap_or(false))
            .map(|path| path.canonicalize())
            .collect::<Result<Vec<_>, _>>()?;
        patch_files.sort_unstable();
        if patch_files.is_empty() {
            return Ok(())
        }
        let status = Command::new("git")
            .arg("am")
            .arg("--3way")
            .args(patch_files.iter().map(|path| path.to_string_lossy().to_string()))
            .current_dir(&repo)
            .status()?;
        if !status.success() {
            // git am --abort?
            bail!("failed to cleanly apply patches - run `cd papermario && git am --abort` to abort the merge");
        }
        Ok(())
    }

    /// Copies the package to the given path and updates. The path must not exist.
    /// Effectively a set_path method.
    pub fn clone_to_dir(&self, path: PathBuf) -> Result<Self> {
        if path.exists() {
            bail!("{} already exists", path.display());
        }
        // Copy entire directory structure
        fs::create_dir_all(&path)?;
        let mut copy_opts = fs_extra::dir::CopyOptions::new();
        copy_opts.content_only = true;
        fs_extra::dir::copy(&self.path, &path, &copy_opts)?;
        Self::try_from(path)
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.manifest() {
            Ok(manifest) => {
                write!(f, "{}", manifest.metadata().name())?;
                let authors = manifest.metadata().authors();
                if authors.is_empty() {
                    write!(f, " (unknown authors)")?;
                } else {
                    write!(f, " by {}", authors[0])?;
                    for author in authors.iter().skip(1) {
                        write!(f, ", {}", author)?;
                    }
                }
                Ok(())
            }
            Err(error) => {
                log::warn!("{:?}", error.context("Error displaying package name"));
                write!(f, "{} (manifest error)", self.path.display())
            }
        }
    }
}

impl PartialEq for Package {
    // TODO: compare directory contents
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Package {}

impl Hash for Package {
    // TODO: hash directories with merkle_hash
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

/// Finds the nearest git root, starting from the current directory.
/// If within a submodule, returns the root of the parent repo.
/// Otherwise, returns the root of the current repo.
/// If not in a repo at all, returns an error.
#[allow(dead_code)]
fn nearest_git_root() -> Result<PathBuf> {
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

fn generate_readme(package_name: &Name) -> String {
    include_str!("../templates/README.md")
        .replace("{{package_name}}", &format!("{}", package_name))
}

fn generate_license(manifest: &Manifest) -> String {
    let authors = manifest.metadata().authors();
    let author_names;
    if authors.is_empty() {
        author_names = "Unknown Author(s)".to_string();
    } else {
        author_names = authors.join(", ");
    }

    include_str!("../templates/CC-BY-SA-4.0.txt")
        .replace("{{year}}", &chrono::Utc::now().format("%Y").to_string())
        .replace("{{author_names}}", &author_names)
}
