use anyhow::Result;

#[test]
fn cli_tests() -> Result<()> {
    trycmd::TestCases::new()
        .case("tests/cmd/*.toml")
        .run();
    Ok(())
}
