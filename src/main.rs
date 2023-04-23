use clap::Parser;
use anyhow::Result;

mod new;

/// General-purpose Paper Mario (N64) modding tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
    /// Creates a new mod.
    New(new::Args),
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.subcmd {
        SubCommand::New(new_args) => new::run(new_args),
    }
}
