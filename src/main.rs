use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod manifest_generator;

#[derive(Parser)]
#[command(name = "onix")]
#[command(about = "The modern binary distributor", version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Onix project
    Init,
    /// Publish a new release (builds, tags, and generates manifest)
    Publish {
        /// Optional version to publish (overrides config)
        version: Option<String>,
    },
    /// Install a package from a GitHub repository
    Install {
        /// The repository to install from (e.g., user/repo)
        repo: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init => commands::init::execute()?,
        Commands::Publish { version } => commands::publish::execute(version.clone()).await?,
        Commands::Install { repo } => commands::install::execute(repo.clone()).await?,
    }

    Ok(())
}