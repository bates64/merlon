use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use merlon::mod_dir::ModDir;

#[derive(Parser, Debug)]
pub struct Args {
    /// Unlocked package to apply.
    input: PathBuf,
}

pub fn run(mod_dir: &mut ModDir, args: Args) -> Result<()> {
    let submodule_dir = mod_dir.submodule_dir();
    let pkg = merlon::package::Package::try_from(args.input)?;
    pkg.apply_patches_to_repo(&submodule_dir)
}
