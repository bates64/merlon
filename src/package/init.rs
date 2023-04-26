//! A package is considered "initialised" when it has the following additional directory structure.
//! 
//! package/
//! ├── ...
//! ├── .merlon/
//! │   ├── dependencies/
//! │   │   └── <other_package_id>/
//! ├── papermario/                 - git clone of decomp (not submodule)
//! │   ├── assets/
//! │   │   └── <package_id>/
//! │   ├── src/
//! │   └── ver/
//! │       └── us/
//! │           ├── splat.yaml
//! │           └── baserom.z64
//! ├── .vscode/                    - optional, but created by default
//! │   ├── c_cpp_properties.json
//! │   ├── extensions.json
//! │   ├── settings.json
//! │   └── tasks.json
//! └── .gitignore                  - optional, but created by default
//!
//! Being initialised means that the package is ready to be built.

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::fs::{create_dir, create_dir_all, remove_dir_all, write, copy, remove_file};
use anyhow::{Result, Error, bail, Context};
use clap::Parser;
use scopeguard::defer;

use super::{Package, Id, Registry};
use crate::rom::Rom;

const MERLON_DIR_NAME: &str = ".merlon";
const DEPENDENCIES_DIR_NAME: &str = ".merlon/dependencies";
const SUBREPO_DIR_NAME: &str = "papermario";
const VSCODE_DIR_NAME: &str = ".vscode";
const GITIGNORE_FILE_NAME: &str = ".gitignore";

#[derive(Debug)]
pub struct InitialisedPackage {
    registry: Registry,
    package_id: Id,
}

#[derive(Parser, Debug)]
pub struct InitialiseOptions {
    /// Path to an unmodified US-release Paper Mario (N64) ROM.
    #[arg(long)]
    pub baserom: PathBuf,
}

#[derive(Parser, Debug)]
pub struct BuildRomOptions {
    /// Whether to skip configuring (useful if you've already configured).
    #[arg(long)]
    pub skip_configure: bool,

    /// Path to output ROM to.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

impl Package {
    /// Initialises this package if needed, and returns an InitialisedPackage.
    pub fn to_initialised(self, initialise_options: InitialiseOptions) -> Result<InitialisedPackage> {
        if InitialisedPackage::is_initialised(&self)? {
            InitialisedPackage::from_initialised(self.clone())
        } else {
            InitialisedPackage::initialise(self.clone(), initialise_options)
        }
    }
}

impl InitialisedPackage {
    pub fn from_initialised(package: Package) -> Result<Self> {
        if !Self::is_initialised(&package)? {
            bail!("package is not initialised");
        }

        let dependencies_dir_path = package.path().join(DEPENDENCIES_DIR_NAME);

        // Create registry of this package and .merlon/dependencies/*
        let mut registry = Registry::new();
        let package_id = registry.register(package)?;
        if dependencies_dir_path.is_dir() {
            for path in dependencies_dir_path.read_dir()? {
                let path = path?;
                if path.file_type()?.is_dir() {
                    let package = Package::try_from(path.path())?;
                    registry.register(package)?;
                }
            }
        }

        Ok(Self {
            registry,
            package_id,
        })
    }

    pub fn package(&self) -> &Package {
        self.registry.get(self.package_id).expect("package somehow removed from registry")
    }

    pub fn baserom_path(&self) -> PathBuf {
        self.subrepo_path().join("ver/us/baserom.z64")
    }

    pub fn subrepo_path(&self) -> PathBuf {
        self.package().path().join(SUBREPO_DIR_NAME)
    }

    pub fn initialise(package: Package, options: InitialiseOptions) -> Result<Self> {
        if Self::is_initialised(&package)? {
            bail!("package is already initialised");
        }
        let path_clone = package.path().to_owned();
        let error_context = format!("failed to initialise package {}", &package);
        let do_it = || {
            let package_id_string = package.id()?.to_string();

            // Clone decomp subrepo
            let mut command = Command::new("git");
            command
                .arg("clone")
                .arg("--depth=1");
            // TODO: if existing clone, reference that
            /*if let Some(repo) = local_decomp_repo.as_ref() {
                command.arg("--reference").arg(repo.path());
            }*/
            let status = command
                .arg("https://github.com/pmret/papermario.git")
                .arg(SUBREPO_DIR_NAME)
                .current_dir(package.path())
                .status()?;
            if !status.success() {
                bail!("failed to clone decomp repository");
            }

            // TODO: declare dependency on papermario

            // Create assets dir for this mod
            create_dir_all(package.path().join(SUBREPO_DIR_NAME).join("assets").join(&package_id_string))
                .context("failed to create assets subdirectory")?;

            // Copy baserom
            if !options.baserom.is_file() {
                bail!("baserom {:?} is not a file", options.baserom);
            }
            // TODO: check baserom sha1 is valid
            let baserom_path = package.path().join(SUBREPO_DIR_NAME).join("ver/us/baserom.z64");
            copy(options.baserom, &baserom_path)
                .with_context(|| format!("failed to copy baserom to {:?}", baserom_path))?;

            // Create merlon dir
            create_dir(package.path().join(MERLON_DIR_NAME))
                .with_context(|| format!("failed to create {MERLON_DIR_NAME} directory"))?;

            // Create vscode dir and copy files
            let vscode_dir = package.path().join(VSCODE_DIR_NAME);
            create_dir(&vscode_dir)
                .with_context(|| format!("failed to create {VSCODE_DIR_NAME} directory"))?;
            write(vscode_dir.join("c_cpp_properties.json"), include_str!("../../templates/.vscode/c_cpp_properties.json"))
                .with_context(|| format!("failed to create {VSCODE_DIR_NAME}/c_cpp_properties.json"))?;
            write(vscode_dir.join("extensions.json"), include_str!("../../templates/.vscode/extensions.json"))
                .with_context(|| format!("failed to create {VSCODE_DIR_NAME}/extensions.json"))?;
            write(vscode_dir.join("settings.json"), include_str!("../../templates/.vscode/settings.json"))
                .with_context(|| format!("failed to create {VSCODE_DIR_NAME}/settings.json"))?;
            write(vscode_dir.join("tasks.json"), include_str!("../../templates/.vscode/tasks.json"))
                .with_context(|| format!("failed to create {VSCODE_DIR_NAME}/tasks.json"))?;

            // Create gitignore file if it doesn't exist
            let gitignore_path = package.path().join(GITIGNORE_FILE_NAME);
            if !gitignore_path.exists() {
                write(gitignore_path, include_str!("../../templates/gitignore"))
                    .with_context(|| format!("failed to create {GITIGNORE_FILE_NAME}"))?;
            }

            // Run decomp install.sh
            let status = Command::new("bash")
                .arg("install.sh")
                .current_dir(package.path().join(SUBREPO_DIR_NAME))
                .status()?;
            if !status.success() {
                bail!("failed to run decomp install.sh");
            }

            let package_id_string = package.id()?.to_string();
            let initialised = Self::from_initialised(package)?;
            initialised.git_create_branch(&package_id_string)?;
            initialised.git_checkout_branch(&package_id_string)?;
            initialised.sync_with_repo()?;
            Ok(initialised)
        };
        match do_it() {
            Err(e) => {
                // Cleanup
                let _ = remove_dir_all(path_clone.join(SUBREPO_DIR_NAME));
                let _ = remove_dir_all(path_clone.join(MERLON_DIR_NAME));
                let _ = remove_dir_all(path_clone.join(VSCODE_DIR_NAME));
                let _ = remove_file(path_clone.join(GITIGNORE_FILE_NAME));
                Err(e).context(error_context)
            },
            result => result,
        }
    }

    // TODO: move to impl Package
    pub fn is_initialised(package: &Package) -> Result<bool> {
        let path = package.path();

        // Check subrepo exists and is a git repo
        if !path.join(SUBREPO_DIR_NAME).is_dir() {
            return Ok(false);
        }
        let status = Command::new("git")
            .arg("status")
            .current_dir(path.join(SUBREPO_DIR_NAME))
            .stdout(Stdio::null())
            .status()?;
        if !status.success() {
            return Ok(false);
        }

        // Check merlon dir exists
        if !path.join(MERLON_DIR_NAME).is_dir() {
            return Ok(false);
        }

        // VSCODE_DIR_NAME and GITIGNORE_FILE_NAME are optional

        Ok(true)
    }

    /// Perform a repo sync.
    /// 
    /// This will recreate the dependency tree in the subrepo, where each dependency is a branch.
    /// Additionally, a branch will be created for this package if it doesn't exist, and the branch will be off of
    /// the branches that this package directly depends on.
    /// 
    /// splat.yaml will be updated so that the asset_stack is correct.
    /// 
    /// Any commits since the last branch will be stashed, and then unstashed after the sync.
    pub fn sync_with_repo(&self) -> Result<()> {
        let package_id_str = self.package_id.to_string();

        if self.git_current_branch()? != package_id_str {
            bail!("not on correct branch! run `git checkout {}` in {}/ to fix", package_id_str, SUBREPO_DIR_NAME);
        }

        // Stash any changes
        if self.git_is_dirty()? {
            self.git_stash()?;
            defer! { warn_if_err(self.git_stash_pop()); }
        }

        // Create dependency tree as branches, if they don't exist already
        // TODO: recreate dependency tree as branches
        if self.git_current_branch()? != package_id_str {
            self.git_checkout_branch(&package_id_str)?;
        }

        // Update splat.yaml with dependencies
        // TODO
        // TODO: also need to figure out whether to store splat.yaml in patches/ or not - probably not, but need merge strategy

        Ok(())
    }

    /// Builds the ROM and returns the path to the output ROM.
    pub fn build_rom(&self, options: BuildRomOptions) -> Result<Rom> {
        let dir = self.subrepo_path();

        // Configure
        // TODO: only do this if we have to (i.e. file tree changed) - maybe ask git?
        if !options.skip_configure {
            let status = Command::new("./configure")
                //.arg("--non-matching")
                //.arg("--debug")
                .arg("--shift")
                .arg("us")
                .current_dir(&dir)
                .status()?;
            if !status.success() {
                bail!("failed to configure");
            }
        }

        // Build
        let status = Command::new("ninja")
            .current_dir(&dir)
            .status()?;
        if !status.success() {
            bail!("failed to build");
        }

        // Copy output file if needed
        let rom = dir.join("ver/us/build/papermario.z64");
        if let Some(output) = options.output {
            std::fs::copy(rom, &output)?;
            Ok(output.into())
        } else {
            Ok(rom.into())
        }
    }

    fn git_create_branch(&self, branch_name: &str) -> Result<()> {
        let status = Command::new("git")
            .arg("branch")
            .arg(&branch_name)
            .arg("--no-track") // Don't track the branch on origin, since origin is the original decomp repo
            .current_dir(self.subrepo_path())
            .status()?;
        if !status.success() {
            bail!("failed to create git branch {}", branch_name);
        }
        Ok(())
    }

    fn git_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .current_dir(self.subrepo_path())
            .output()?;
        if !output.status.success() {
            panic!("failed to run git rev-parse");
        }
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(Into::into)
    }

    fn git_checkout_branch(&self, branch_name: &str) -> Result<()> {
        let status = Command::new("git")
            .arg("checkout")
            .arg(&branch_name)
            .current_dir(self.subrepo_path())
            .status()?;
        if !status.success() {
            bail!("failed to checkout git branch {}", branch_name);
        }
        Ok(())
    }

    fn git_is_dirty(&self) -> Result<bool> {
        let output = Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(self.subrepo_path())
            .output()?;
        if !output.status.success() {
            bail!("failed to run git status");
        }
        Ok(!output.stdout.is_empty())
    }

    fn git_stash(&self) -> Result<()> {
        let status = Command::new("git")
            .arg("stash")
            .current_dir(self.subrepo_path())
            .status()?;
        if !status.success() {
            bail!("failed to run git stash");
        }
        Ok(())
    }

    fn git_stash_pop(&self) -> Result<()> {
        let status = Command::new("git")
            .arg("stash")
            .arg("pop")
            .current_dir(self.subrepo_path())
            .status()
            .expect("failed to run git stash pop");
        if !status.success() {
            bail!("failed to run git stash pop");
        }
        Ok(())
    }
}

impl TryFrom<Package> for InitialisedPackage {
    type Error = Error;

    fn try_from(package: Package) -> Result<Self> {
        Self::from_initialised(package)
    }
}

fn warn_if_err<T>(result: Result<T>) {
    if let Err(err) = result {
        log::warn!("{}", err);
    }
}
