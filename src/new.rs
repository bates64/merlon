use std::path::PathBuf;
use clap::Parser;
use anyhow::Result;
use heck::AsKebabCase;
use merlon::package::Package;

#[derive(Parser, Debug)]
pub struct Args {
    /// The name of the mod. This will be used as the mod's directory name.
    /// It is recommended that mods be named in the snake-case format.
    name: String,
}

pub fn run(dir: Option<PathBuf>, args: Args) -> Result<()> {
    // Create the package
    let current_dir = std::env::current_dir()?;
    let dir = dir.unwrap_or_else(|| current_dir.join(format!("{}", AsKebabCase(&args.name))));
    let package = Package::new(args.name, dir)?;

    // Try and make path relative to current directory, but if that fails, just use the absolute path
    let path_relative_to_current = package.path()
        .strip_prefix(current_dir)
        .unwrap_or_else(|_| package.path());

    // Done!
    println!("");
    println!("Created package: {}", &package);
    println!("To build and run this package, run the following commands:");
    println!("");
    println!("    cd {:?}", path_relative_to_current);
    println!("    merlon init");
    println!("    merlon run");
    println!("");

    Ok(())
}
