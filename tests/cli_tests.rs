use std::path::PathBuf;
use std::fs::copy;
use anyhow::Result;

#[test]
fn cli_tests() -> Result<()> {
    trycmd::TestCases::new()
        .case("tests/cmd/*.toml")
        .run();
    Ok(())
}
