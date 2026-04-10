use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use sha2::{Digest, Sha256};
use crate::init::ProjectConfig;
use crate::models::OnixManifest;

/// Executes the project build command, hashes the result, and updates install.onix.
pub fn run_publish() -> Result<()> {
    let config_path = Path::new(".onix/config.yaml");
    if !config_path.exists() {
        return Err(anyhow::anyhow!(".onix/config.yaml not found. Run 'onix init' first."));
    }

    let config_str = fs::read_to_string(config_path)
        .context("Failed to read .onix/config.yaml")?;
    let config: ProjectConfig = serde_yaml::from_str(&config_str)
        .context("Failed to parse .onix/config.yaml")?;

    println!("🚀 Running build command: {}", config.build.command);
    
    let (shell, arg) = if cfg!(windows) { ("cmd", "/C") } else { ("sh", "-c") };
    
    let status = Command::new(shell)
        .arg(arg)
        .arg(&config.build.command)
        .status()
        .context("Failed to execute build command")?;

    if !status.success() {
        return Err(anyhow::anyhow!("Build command failed with status: {}", status));
    }

    let bin_path = find_binary(&config)?;
    println!("📦 Found binary: {:?}", bin_path);

    let bytes = fs::read(&bin_path)
        .with_context(|| format!("Failed to read binary at {:?}", bin_path))?;
    
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hex::encode(hasher.finalize());
    println!("🛡️  Generated SHA256: {}", hash);

    let manifest_path = Path::new("install.onix");
    if !manifest_path.exists() {
        return Err(anyhow::anyhow!("install.onix not found. Run 'onix init' first."));
    }

    let manifest_str = fs::read_to_string(manifest_path)
        .context("Failed to read install.onix")?;
    let mut manifest: OnixManifest = serde_yaml::from_str(&manifest_str)
        .context("Failed to parse install.onix")?;

    let current_os = std::env::consts::OS;
    let current_arch = match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        arch => arch,
    };

    let mut found = false;
    for source in &mut manifest.install_on {
        if source.os == current_os && source.arch == current_arch {
            source.sha256 = hash.clone();
            found = true;
            break;
        }
    }

    if found {
        let updated_manifest = serde_yaml::to_string(&manifest)
            .context("Failed to serialize updated manifest")?;
        fs::write(manifest_path, updated_manifest)
            .context("Failed to write updated install.onix")?;
        println!("✅ Updated install.onix for {}/{}", current_os, current_arch);
    } else {
        println!("⚠️  Current platform ({}/{}) not found in install.onix. No hash was updated.", current_os, current_arch);
    }

    Ok(())
}

fn find_binary(config: &ProjectConfig) -> Result<PathBuf> {
    let bin_name = &config.install.bin_name;
    
    let paths = [
        PathBuf::from(bin_name),
        PathBuf::from(format!("{}.exe", bin_name)),
        PathBuf::from("target/release").join(bin_name),
        PathBuf::from("target/release").join(format!("{}.exe", bin_name)),
    ];

    for path in paths {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow::anyhow!("Could not locate binary: {}. Ensure your build command produces this file.", bin_name))
}