use temp_dir::TempDir;
use anyhow::Result;
use merlon::package::{*, init::*, manifest::*};

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
