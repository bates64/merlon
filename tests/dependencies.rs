use std::process::Command;
use std::io::prelude::*;
use std::fs::File;
use temp_dir::TempDir;
use anyhow::Result;
use merlon::package::{*, init::*, manifest::*, distribute::ExportOptions};

/// Pinned decomp commit hash so that tests don't break when decomp updates
const DECOMP_REV: &str = "7a9df943ad079e7b19df0f8690bdc92e2beed964";

#[path = "rom.rs"]
mod rom;

#[test]
fn initialising_package_gives_decomp_dependency() -> Result<()> {
    let tempdir = TempDir::new()?;
    let pkg_path = tempdir.path().join("test");
    let package = Package::new("Test", pkg_path)?;
    let mut registry = Registry::new();
    let id = registry.register(package)?;
    assert_eq!(registry.all_dependencies()?.len(), 0);
    let package = registry.get_or_error(id)?;
    let _initialised = package.clone().to_initialised(InitialiseOptions {
        baserom: rom::baserom(),
        rev: Some(DECOMP_REV.to_string()),
    })?;
    let all_dependencies = registry.all_dependencies()?;
    assert_eq!(all_dependencies.len(), 1);
    assert!(matches!(all_dependencies.iter().next(), Some(Dependency::Decomp { .. })));
    Ok(())
}

#[test]
fn sync_complex_dependency_graph_to_repo() -> Result<()> {
    let tempdir = TempDir::new()?;
    let dir_path = tempdir.path();
    let mut registry = Registry::new();

    // Helper function to create a package with one patch and register it with the registry
    let mut create_and_register_package = |name: &str| -> Result<Id> {
        let pkg_path = dir_path.join(name);
        let package = Package::new(name, pkg_path)?;

        // Add a single commit adding a test file
        let mut file = File::create(package.path().join("patches/0001-test.patch")).unwrap();
        write!(&mut file, "{}", touch_file_patch(&format!("src/merlon_test_{name}"))).unwrap(); // TODO

        let id = registry.register(package)?;
        Ok(id)
    };

    // Create this dependency graph:
    //        Root      <-- We want to build this package 
    //      /     \
    //    DepA   DepB
    //      \     /
    //     SharedDep
    let root = create_and_register_package("Root")?;
    let dep_a = create_and_register_package("DepA")?;
    let dep_b = create_and_register_package("DepB")?;
    let shared_dep = create_and_register_package("SharedDep")?;
    dbg!(&root, &dep_a, &dep_b, &shared_dep);
    registry.add_direct_dependency(root, dep_a)?;
    registry.add_direct_dependency(root, dep_b)?;
    registry.add_direct_dependency(dep_a, shared_dep)?;

    // Initialise the root package and sync
    let root_package = registry.get_or_error(root)?.clone();
    let mut initialised = root_package.clone().to_initialised(InitialiseOptions {
        baserom: rom::baserom(),
        rev: Some(DECOMP_REV.to_string()),
    })?;
    initialised.set_registry(registry); // XXX
    initialised.sync_repo()?;
    initialised.update_patches_dir()?;

    // There should be 1 patch in the root package now
    let root_patches = initialised.package().path().join("patches");
    dbg!(root_patches
        .read_dir()?
        .map(|e| Ok(e?.file_name()))
        .collect::<Result<Vec<_>>>()?
    );
    assert_eq!(root_patches.read_dir()?.count(), 1);

    // If the patches applied correctly, all the test files should have been made
    assert!(initialised.subrepo_path().join("src/merlon_test_Root.c").is_file());
    assert!(initialised.subrepo_path().join("src/merlon_test_DepA.c").is_file());
    assert!(initialised.subrepo_path().join("src/merlon_test_DepB.c").is_file());
    assert!(initialised.subrepo_path().join("src/merlon_test_SharedDep.c").is_file());

    Ok(())
}

// Generate a random git-like commit hash
fn gen_random_commit_hash_for_patch() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut hash = String::with_capacity(40);
    for _ in 0..40 {
        hash.push(rng.gen_range(b'0'..b'9') as char);
    }
    hash
}

fn touch_file_patch(filename: &str) -> String {
    let hash = gen_random_commit_hash_for_patch();
    format!(r#"From {hash} Mon Sep 17 00:00:00 2001
From: Merlon test <merlontest@nanaian.town>
Date: Wed, 26 Apr 2023 22:40:19 +0100
Subject: test

---
    {filename}.c | 0
    1 file changed, 0 insertions(+), 0 deletions(-)
    create mode 100644 {filename}

diff --git a/{filename} b/{filename}
new file mode 100644
index 0000000..e69de29
-- 
2.39.0"#)
}

#[test]
fn single_dependency() -> Result<()> {
    let tempdir = TempDir::new()?;

    // Root package with no commits
    let root = Package::new("Root", tempdir.path().join("root"))?;
    let mut root = root.to_initialised(InitialiseOptions {
        baserom: rom::baserom(),
        rev: Some(DECOMP_REV.to_string()),
    })?;

    // Dependency package with single commit
    let dependency = Package::new("Dependency", tempdir.path().join("dependency"))?;
    let mut file = File::create(dependency.path().join("patches/0001-skip-intro-patch.patch"))?;
    write!(&mut file, "{}", skip_intro_patch())?;

    // Add dependency, sync repo, check skip intro commit was added
    root.add_dependency(AddDependencyOptions {
        path: dependency.path().to_path_buf(),
    })?;
    root.sync_repo()?;
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%s")
        .current_dir(root.subrepo_path())
        .output()?;
    let head_commit = String::from_utf8(output.stdout)?.trim().to_string();
    assert_eq!(&head_commit, "set bSkipIntro to true");

    // Export root and make some assertions
    let distributable = root.package().export_distributable(ExportOptions {
        baserom: Some(rom::baserom()),
        output: Some(tempdir.path().join("output.merlon")),
    })?;
    distributable.open_scoped(rom::baserom(), |package| {
        let manifest = package.manifest()?;

        // Should have 2 dependencies: decomp & dependency
        assert_eq!(manifest.iter_direct_dependencies().count(), 2);

        // Should have no patches (only dependency has patches)
        let patches_count = package.path().join("patches")
            .read_dir()?
            .count();
        assert_eq!(patches_count, 0);

        Ok(())
    })?;

    Ok(())
}

fn skip_intro_patch() -> &'static str {
    include_str!(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/dependencies/skip_intro_patch.patch"
        )
    )
}
