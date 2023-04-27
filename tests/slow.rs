use temp_dir::TempDir;
use anyhow::Result;
use merlon::package::{*, init::*};

const DECOMP_REV: &str = "7a9df943ad079e7b19df0f8690bdc92e2beed964";

#[path = "rom.rs"]
mod rom;

/// Create a new package, initialise it, and build it.
#[test]
#[ignore]
fn new_init_build() -> Result<()> {
    let tempdir = TempDir::new()?;
    let pkg_path = tempdir.path().join("test");
    let package = Package::new("Test", pkg_path)?;
    let initialised = package.to_initialised(InitialiseOptions {
        baserom: rom::baserom(),
        rev: Some(DECOMP_REV.to_string()),
    })?;
    let rom = initialised.build_rom(BuildRomOptions {
        output: None,
        skip_configure: false,
    })?;
    assert_eq!(rom.sha1_string()?, "e1f9c77fa35549897ace8b8627e821a27309d538");
    Ok(())
}
