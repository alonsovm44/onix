use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::{Context, Result};

mod models;
mod network;
mod init;
mod env;
mod tui;

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
            
            let source = manifest.find_source()
                .context("No compatible binary found for your system's OS/Architecture")?;

            let confirmed = if *yes {
                true
            } else {
                tui::confirm_install(&manifest)?
            };

            if !confirmed {
                println!("Installation cancelled by user.");
                return Ok(());
            }

            println!("Downloading binary artifact...");
            let bin_bytes = network::download_artifact(&source.url, &source.sha256).await?;

            // Resolve target path (Home expansion logic for production would go here)
            let target_dir = std::path::PathBuf::from(manifest.installation.target_dir.replace("~", &std::env::var("HOME").unwrap_or_default()));
            let target_path = target_dir.join(&manifest.installation.bin_name);

            if let Some(parent) = target_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {:?}", parent))?;
            }

            std::fs::write(&target_path, bin_bytes)
                .with_context(|| format!("Failed to write binary to {:?}", target_path))?;
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&target_path)?.permissions();
                perms.set_mode(0o755); // rwxr-xr-x
                std::fs::set_permissions(&target_path, perms)?;
            }

            println!("✅ Installed binary to {:?}", target_path);

            if manifest.permissions.iter().any(|p| p == "env:PATH") {
                env::add_to_path(&target_dir)
                    .context("Failed to update system PATH")?;
                println!("✅ Updated system PATH environment variable");
            }

            if let Some(msg) = manifest.message {
                println!("\n{}", msg);
            }
        }
        Commands::Inspect { url } => {
            println!("Inspecting manifest at: {}", url);
            let manifest = network::fetch_manifest(url).await?;
            print_install_plan(&manifest)?;
        }
        Commands::Validate { path } => {
            let content = std::fs::read_to_string(&path)
                .with_context(|| format!("Failed to read manifest file at {:?}", path))?;

            let _manifest: models::OnixManifest = serde_yaml::from_str(&content)
                .with_context(|| format!("Manifest at {:?} is invalid", path))?;

            println!("✅ Manifest at {:?} is valid!", path);
        }
        Commands::Init => {
            init::run_init()?;
        }
        Commands::Publish => {
            println!("Preparing project for publication...");
            // TODO: Implement build matrix generation
        }
        Commands::SelfInstall => {
            println!("Bootstrapping Onix onto the system...");
            // TODO: Implement self-installation logic
        }
    }

    Ok(())
}

/// Prints a clean, non-interactive summary of the installation plan for the 'inspect' command.
fn print_install_plan(manifest: &models::OnixManifest) -> Result<()> {
    println!("\n❄️  Onix Install Plan: {} v{}", manifest.app, manifest.version);
    println!("--------------------------------------------------");
    
    let source = manifest.find_source()
        .context("No compatible binary found for your system's OS/Architecture")?;

    println!("Source URL:   {}", source.url);
    println!("Target Dir:   {}", manifest.installation.target_dir);
    println!("Binary Name:  {}", manifest.installation.bin_name);
    println!("Checksum:     {}", source.sha256);
    
    println!("\nRequested Permissions:");
    for perm in &manifest.permissions {
        println!("  • {}", perm);
    }
    
    if let Some(msg) = &manifest.message {
        println!("\nPost-install message:\n  {}", msg);
    }
    println!("--------------------------------------------------");
    println!("Dry run complete. No changes were made to your system.");
    Ok(())
}