use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Context, Result};

mod models;
mod network;
mod init;
mod env;
mod tui;
mod publish;

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
    Publish {
        /// Update install.onix with hashes from a checksum file (e.g. dist/checksums.txt)
        #[arg(long)]
        update_hashes: Option<PathBuf>,

        /// Increment version (patch, minor, or major)
        #[arg(long, value_parser = ["patch", "minor", "major"])]
        bump: Option<String>,
    },

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
            let resolved_url = network::resolve_url(url).await;
            let manifest = network::fetch_manifest(&resolved_url).await?;
            
            let confirmed = if *yes {
                true
            } else {
                tui::display_manifest_tui(manifest.clone(), true)
                    .map_err(|e| anyhow::anyhow!("TUI error: {}", e))?
            };

            if !confirmed {
                println!("❌ Installation aborted by user.");
                return Ok(());
            }

            println!("🚀 Installing: {} v{}", manifest.app, manifest.version);
            // TODO: Call Phase 3 logic (download_artifact and write to disk)
        }
        Commands::Inspect { url } => {
            let resolved_url = network::resolve_url(url).await;
            let manifest = network::fetch_manifest(&resolved_url).await?;
            tui::display_manifest_tui(manifest, false)
                .map_err(|e| anyhow::anyhow!("TUI error: {}", e))?;
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
        Commands::Publish { update_hashes, bump } => {
            if let Some(path) = update_hashes {
                publish::update_manifest_hashes(path)?;
            } else {
                publish::run_publish(bump.as_deref())?;
            }
        }
        Commands::SelfInstall => {
            println!("Bootstrapping Onix onto the system...");
            // TODO: Implement self-installation logic
        }
    }

    Ok(())
}