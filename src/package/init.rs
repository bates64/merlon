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
use anyhow::{Result, Error, bail, anyhow, Context};
use clap::Parser;
use scopeguard::defer;
use semver::VersionReq;

use super::manifest::Dependency;
use super::{Package, Id, Registry, PATCHES_DIR_NAME, Distributable};
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

    /// Git revision of decomp to use.
    /// 
    /// If not provided, the latest commit on `main` is used.
    #[arg(long)]
    pub rev: Option<String>,   
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

#[derive(Parser, Debug)]
pub struct AddDependencyOptions {
    /// Path to the package to add as a dependency.
    #[arg(long)]
    pub path: PathBuf,
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

    pub fn package_id(&self) -> Id {
        self.package_id
    }

    pub fn baserom_path(&self) -> PathBuf {
        self.subrepo_path().join("ver/us/baserom.z64")
    }

    pub fn subrepo_path(&self) -> PathBuf {
        self.package().path().join(SUBREPO_DIR_NAME)
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    pub fn set_registry(&mut self, registry: Registry) {
        self.registry = registry;
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

            if let Some(rev) = options.rev {
                // Reset to revision
                let status = Command::new("git")
                    .arg("reset")
                    .arg("--hard")
                    .arg(&rev)
                    .current_dir(package.path().join(SUBREPO_DIR_NAME))
                    .status()?;
                if !status.success() {
                    bail!("failed to checkout revision");
                }
            }

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

            let initialised = Self::from_initialised(package)?;

            // Add decomp as dependency
            let main_head = initialised.git_head_commit()?;
            initialised.package().edit_manifest(|manifest| {
                manifest.upsert_decomp_dependency(main_head)
            })?;

            // In case there are patches in the package already, apply them
            // i.e. sync patches ---> repo
            let branch_name = initialised.package_id().to_string();
            initialised.git_create_branch(&branch_name)?;
            initialised.git_checkout_branch(&branch_name)?;
            initialised.package().apply_patches_to_decomp_repo(&initialised.subrepo_path())?;

            // Load dependency patches
            initialised.setup_git_branches()?;

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
    /// This will recreate the dependency tree in the subrepo, where each dependency has a branch.
    /// Additionally, a branch will be created for this package if it doesn't exist, and the branch will be off of
    /// the branches that this package directly depends on.
    /// Each branch will have the respective package patches applied to it.
    /// 
    /// splat.yaml will be updated so that the asset_stack is correct.
    pub fn setup_git_branches(&self) -> Result<()> {
        // Make sure commits are saved to patches/
        let package_id_string = self.package_id.to_string();
        if self.git_branch_exists(&package_id_string)? {
            log::info!("updating patches directory");
            self.update_patches_dir()
                .context("failed to update patches dir for backup")?;
        }

        log::info!("starting repo sync");

        // Switch to main so we can delete branches
        self.git_checkout_branch("main")?;

        // Delete all branches, if they exist
        let dependencies_including_self = {
            let mut deps = self.registry.get_dependencies(self.package_id)?;
            let manifest = self.package().manifest()?;
            let version = manifest.metadata().version();
            deps.insert(Dependency::Package {
                id: self.package_id,
                version: VersionReq {
                    comparators: vec![semver::Comparator {
                        op: semver::Op::Exact,
                        major: version.major,
                        minor: Some(version.minor),
                        patch: Some(version.patch),
                        pre: version.pre.clone(),
                    }],
                }
            });
            deps
        };
        for dependency in dependencies_including_self {
            if let Dependency::Package { id, .. } = dependency {
                let id_string = id.to_string();
                if self.git_branch_exists(&id_string)? {
                    self.git_delete_branch(&id_string)?;
                }
            }
        }

        // Create dependency tree as branches
        let patch_order = self.registry.calc_dependency_patch_order(self.package_id)?;
        let repo = self.subrepo_path();
        for id in patch_order {
            let package = self.registry.get_or_error(id)?;
            log::info!("applying patches of package: {}", &package);
            let id_string = id.to_string();
            self.git_create_branch(&id_string)?;
            self.git_checkout_branch(&id_string)?;
            let package = self.registry.get_or_error(id)?;
            package.apply_patches_to_decomp_repo(&repo)?;
        }
        if self.git_current_branch()? != self.package_id.to_string() {
            bail!("patch order was incorrect");
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

    /// Writes the patches required to take the repo from the nearest dependency to this package's branch into the patches dir.
    pub fn update_patches_dir(&self) -> Result<()> {
        let package_id_str = self.package_id.to_string();
        if self.git_current_branch()? != package_id_str {
            bail!("repo is not on package branch {}", package_id_str);
        }
        if self.git_is_dirty()? {
            bail!("repo is dirty, commit changes and try again");
        }

        let dir = self.package().path().join(PATCHES_DIR_NAME);
        remove_dir_all(&dir)
            .with_context(|| format!("failed to remove patches dir {}", dir.display()))?;
        create_dir(&dir)
            .with_context(|| format!("failed to create patches dir {}", dir.display()))?;

        // Figure out which branch to diff against.
        // We want to diff against the nearest dependency, but if that doesn't exist, we want to diff against main.
        let branch_order =
            std::iter::once("main".to_string())
                .chain(
                    self.registry()
                        .calc_dependency_patch_order(self.package_id)?
                        .into_iter()
                        .map(|id| id.to_string())
                );
        let mut diff_against = None;
        for branch in branch_order.rev() {
            if branch != package_id_str && self.git_branch_exists(&branch)? {
                diff_against = Some(branch);
                break;
            }
        };
        let diff_against = diff_against.ok_or_else(|| anyhow!("no branch to diff against"))?;
        let diff_against_package_name = match diff_against.as_str() {
            "main" => "Paper Mario (N64) decompilation".to_string(),
            _ => {
                let package = self.registry.get_or_error(diff_against.parse()?)?;
                format!("{}", package)
            },
        };
        log::info!("saving patches since dependency: {}", &diff_against_package_name);

        // Create patches
        let status = Command::new("git")
            .arg("format-patch")
            .arg(format!("{}..HEAD", diff_against))
            .arg("-o").arg(&dir.canonicalize()?)
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
            .arg("assets") //.arg(format!("assets/{}", package_name))
            .arg("ver/us")
            .arg("--no-track") // Don't track the branch on origin, since origin is the original decomp repo
            .current_dir(self.subrepo_path())
            .status()?;
        if !status.success() {
            bail!("failed git format-patch");
        }

        // List patches
        let patches = std::fs::read_dir(&dir)?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.extension()? == "patch" {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        log::info!("saved {} patches", patches.len());

        Ok(())
    }

    /// Adds a dependency by copying it into the dependencies directory and registering it.
    /// If the dependency already exists, it will be updated.
    /// Specifically, it will be copied into `.merlon/dependencies/<package_id>`.
    pub fn add_dependency(&mut self, options: AddDependencyOptions) -> Result<Id> {
        let path = options.path;
        let dependencies_dir = self.package().path().join(DEPENDENCIES_DIR_NAME);
        let package = if super::is_unexported_package(&path) {
            let package = Package::try_from(path)?;

            // Could also do symbolic link?
            let path = dependencies_dir.join(package.id()?.to_string());
            if path.is_dir() {
                log::info!("dependency directory already exists, updating it");
                remove_dir_all(&path)?;
            }
            let package = package.clone_to_dir(path)
                .context("failed to clone package to dependencies dir")?;

            // If package has any dependencies in its directory we don't have, add them too
            if let Ok(initialised) = InitialisedPackage::try_from(package.clone()) {
                log::info!("copying dependencies of new dependency to this package");
                for id in initialised.registry().package_ids() {
                    if !self.registry.has(id) {
                        self.add_dependency(AddDependencyOptions {
                            path: self.registry.get_or_error(id)?.path().to_owned(),
                        })?;
                    }
                }
            }

            package
        } else if super::distribute::is_distributable_package(&path) {
            let distributable = Distributable::try_from(path)?;
            let manifest = distributable.manifest(self.baserom_path())?;
            let package_id = manifest.metadata().id().to_string();
            let path = dependencies_dir.join(package_id);
            if path.is_dir() {
                log::info!("dependency directory already exists, updating it");
                remove_dir_all(&path)?;
            }
            distributable.open_to_dir(super::distribute::OpenOptions {
                output: Some(path),
                baserom: self.baserom_path(),
            })?
        } else {
            bail!("not a package directory or distributable file: {}", path.display());
        };
        let id = package.id()?;
        let id = match self.registry.has(id) {
            true => id,
            false => self.registry.register(package)?,
        };
        let dependency: Dependency = self.registry.get_or_error(id)
            .context("dependency not added to registry correctly")?
            .try_into()?;
        self.package().edit_manifest(move |manifest| {
            manifest.declare_direct_dependency(dependency)
        })?;
        Ok(id)
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

    fn git_branch_exists(&self, branch_name: &str) -> Result<bool> {
        let output = Command::new("git")
            .arg("branch")
            .arg("--list")
            .arg(&branch_name)
            .current_dir(self.subrepo_path())
            .output()?;
        if !output.status.success() {
            bail!("failed to run git branch --list {}", branch_name);
        }
        Ok(!output.stdout.is_empty())
    }

    fn git_delete_branch(&self, branch_name: &str) -> Result<()> {
        let status = Command::new("git")
            .arg("branch")
            .arg("-D")
            .arg(&branch_name)
            .current_dir(self.subrepo_path())
            .status()?;
        if !status.success() {
            bail!("failed to run git branch -D {}", branch_name);
        }
        Ok(())
    }

    /// Stashes if needed, switches to the main branch, pulls, then switches back, merges, and pops stash.
    /// Also updates the decomp dependency in the package manifest to the main's HEAD commit.
    pub fn update_decomp(&self) -> Result<()> {
        let main_branch = "main";
        let prev_branch = self.git_current_branch()?;

        // Stash if needed
        if self.git_is_dirty()? {
            self.git_stash()?;
            defer!(warn_if_err(self.git_stash_pop()));
        }

        // Switch to main branch
        if prev_branch != main_branch {
            self.git_checkout_branch(main_branch)?;
        }

        // Pull
        let status = Command::new("git")
            .arg("pull")
            .current_dir(self.subrepo_path())
            .status()?;
        if !status.success() {
            bail!("failed to run git pull");
        }

        // Switch back to package branch
        if prev_branch != main_branch {
            self.git_checkout_branch(&prev_branch)?;

            // Merge main into package branch
            // TODO: and/or sync_to_repo?
            let status = Command::new("git")
                .arg("merge")
                .arg(main_branch)
                .current_dir(self.subrepo_path())
                .status()?;
            if !status.success() {
                bail!("failed to run git merge");
            }
        }

        // Update decomp dependency in manifest
        let main_head = self.git_head_commit()?;
        self.package().edit_manifest(|manifest| {
            manifest.upsert_decomp_dependency(main_head)
        })
    }

    fn git_head_commit(&self) -> Result<String> {
        let output = Command::new("git")
            .arg("rev-parse")
            .arg("HEAD")
            .current_dir(self.subrepo_path())
            .output()?;
        if !output.status.success() {
            bail!("failed to run git rev-parse");
        }
        String::from_utf8(output.stdout)
            .map(|s| s.trim().to_string())
            .map_err(Into::into)
    }
}

impl TryFrom<Package> for InitialisedPackage {
    type Error = Error;

    fn try_from(package: Package) -> Result<Self> {
        Self::from_initialised(package)
    }
}

fn warn_if_err<T, E: std::fmt::Debug>(result: Result<T, E>) {
    if let Err(err) = result {
        log::warn!("{:?}", err);
    }
}
