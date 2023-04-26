use std::path::PathBuf;

pub fn baserom() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/baserom.z64"))
}

/// Check that the baserom exists. It isn't checked into git, so testers should copy theirs to tests/baserom.z64.
#[test]
fn baserom_exists() {
    assert!(baserom().exists());
}
