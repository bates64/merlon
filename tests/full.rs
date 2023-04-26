use std::path::PathBuf;
use temp_dir::TempDir;
use anyhow::Result;
use merlon::package::{*, init::*};

fn baserom() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/baserom.z64"))
}

#[test]
fn baserom_exists() {
    assert!(baserom().exists());
}

/// Create a new package, initialise it, and build it.
#[test]
fn new_init_build() -> Result<()> {
    let tempdir = TempDir::new()?;
    let pkg_path = tempdir.path().join("test");
    let package = Package::new("Test", pkg_path)?;
    let initialised = package.to_initialised(InitialiseOptions {
        baserom: baserom(),
    })?;
    let rom = initialised.build_rom(BuildRomOptions {
        output: None,
        skip_configure: false,
    })?;
    assert_eq!(rom.sha1_string()?, "e1f9c77fa35549897ace8b8627e821a27309d538");
    Ok(())
}
