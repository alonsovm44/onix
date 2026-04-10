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

            let source = manifest.find_source()
                .context("No compatible binary found for your system (OS/Arch mismatch).")?;

            println!("📥 Downloading {}...", manifest.app);
            let binary_bytes = network::download_artifact(&source.url, &source.sha256).await?;

            // 1. Resolve target directory
            let mut target_dir_str = manifest.installation.target_dir.clone();
            
            // Feature: Use a centralized path on Windows if the manifest uses the default Unix style
            #[cfg(windows)]
            if target_dir_str == "~/.local/bin" {
                target_dir_str = "C:\\onix".to_string();
            }

            // 2. Expand home directory (~)
            if target_dir_str.starts_with('~') {
                let home = if cfg!(windows) { std::env::var("USERPROFILE") } else { std::env::var("HOME") }
                    .context("Failed to resolve home directory for path expansion")?;
                target_dir_str = target_dir_str.replacen('~', &home, 1);
            }

            let target_dir = PathBuf::from(target_dir_str);
            let target_path = target_dir.join(&manifest.installation.bin_name);

            // 3. Create directory and write binary
            std::fs::create_dir_all(&target_dir)
                .with_context(|| format!("Failed to create directory {:?}", target_dir))?;
                
            std::fs::write(&target_path, binary_bytes)
                .with_context(|| format!("Failed to write binary to {:?}", target_path))?;

            // 4. Set executable permissions on Unix systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&target_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&target_path, perms)?;
            }

            println!("✅ Successfully installed {} to {:?}", manifest.app, target_path);
            if let Some(msg) = &manifest.message {
                println!("\n{}", msg);
            }
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