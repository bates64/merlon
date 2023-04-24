use assert_cmd::prelude::*;
use std::process::Command;

#[test]
fn skip_intro_logos() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new mod
    let mut cmd = Command::cargo_bin("merlon")?;
    cmd.arg("new").arg("mymod").assert().success();

    // Enter the directory
    std::env::set_current_dir("mymod")?;

    // Apply the package
    let mut cmd = Command::cargo_bin("merlon")?;
    cmd.arg("apply").arg("../../examples/skip-intro-logos.merlon").assert().success();

    // Build the mod
    let mut cmd = Command::cargo_bin("merlon")?;
    cmd.arg("build").assert().success();

    Ok(())
}
