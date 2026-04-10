use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Context, Result};

mod models;
mod network;
mod init;
mod env;
mod tui;

///
/// Onix ❄️: A universal, trust-first installer for standalone binaries.
#[derive(Parser)]
#[command(name = "onix")]
#[command(about = "Safe tool installation without the curl | sh risk", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetches and installs a tool from a .onix manifest URL
    Install {
        /// URL to the .onix manifest
        url: String,

        /// Skip the TUI confirmation prompt (use for CI/CD)
        #[arg(short, long)]
        yes: bool,
    },

    /// Dry-run: Parses and validates the manifest, then shows the install plan
    Inspect {
        /// URL to the .onix manifest
        url: String,
    },

    /// Validates a local .onix manifest file for schema errors
    Validate {
        /// Path to the local .onix file
        path: PathBuf,
    },

    /// Interactively prepares a repository for Onix distribution
    Init,

    /// Generates the compilation matrix and publish-ready hashes
    Publish,

    /// Installs the Onix binary to the system path
    SelfInstall,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        println!("[DEBUG] Verbose mode enabled");
    }

    match &cli.command {
        Commands::Install { url, yes } => {
            println!("Installing from: {}", url);
            if *yes {
                println!("Confirmation skipped (--yes)");
            }
            
            let manifest = network::fetch_manifest(url).await?;
            println!("Successfully fetched manifest for: {} v{}", manifest.app, manifest.version);
            
            // TODO: Implement TUI logic for permission confirmation
        }
        Commands::Inspect { url } => {
            println!("Inspecting manifest at: {}", url);
            let manifest = network::fetch_manifest(url).await?;
            println!("{:#?}", manifest);
        }
        Commands::Validate { path } => {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read manifest file at {:?}", path))?;

            let _manifest: models::OnixManifest = serde_yaml::from_str(&content)
                .with_context(|| format!("Manifest at {:?} is invalid", path))?;

            println!("✅ Manifest at {:?} is valid!", path);
        }
        Commands::Init => {
            init::run_init()?;
        }
        Commands::Publish => {
            // This would normally call the publish logic
            println!("Running publish logic...");
        }
        Commands::SelfInstall => {
            println!("Bootstrapping Onix onto the system...");
            // TODO: Implement self-installation logic
        }
    }

    Ok(())
}