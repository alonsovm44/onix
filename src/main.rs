use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod manifest_generator;
mod models;
mod network;
mod env;
mod utils;

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
        #[arg(short = 'v', long = "version-override", long = "v")]
        version: Option<String>,
        /// Output a JSON debug report to the console and a file
        #[arg(long)]
        debug: bool,
        /// Show what would happen without executing Git commands or modifying files
        #[arg(long)]
        dry_run: bool,
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

    let result = match &cli.command {
        Commands::Init => commands::init::execute(),
        Commands::Publish { version, debug, dry_run } => commands::publish::execute(version.clone(), *debug, *dry_run).await,
        Commands::Install { repo } => commands::install::execute(repo.clone()).await,
    };

    if let Err(e) = result {
        eprintln!("\n❌ Error: {:?}", e);
        std::process::exit(1);
    }

    Ok(())
}