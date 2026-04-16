use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::network;
use crate::env;

pub async fn execute(repo: String) -> Result<()> {
    println!("🔍 Resolving package: {}...", repo);

    // 1. Resolve the manifest URL
    let manifest_url = network::resolve_url(&repo).await;
    println!("📄 Fetching manifest from {}...", manifest_url);

    // 2. Fetch and parse the manifest
    let manifest = network::fetch_manifest(&manifest_url).await
        .context("Failed to fetch install manifest. Make sure the repo has .onix/install.onix")?;

    println!("📦 Found: {} v{}", manifest.app, manifest.version);

    // 3. Find the correct platform source
    let source = manifest.find_source()
        .ok_or_else(|| anyhow::anyhow!(
            "No binary available for your platform ({} {}). Available: {}",
            std::env::consts::OS,
            std::env::consts::ARCH,
            manifest.install_on.iter()
                .map(|s| format!("{}/{}", s.os, s.arch))
                .collect::<Vec<_>>()
                .join(", ")
        ))?;

    println!("🎯 Selected binary: {}/{}", source.os, source.arch);

    // 4. Show install plan
    let target_dir = Path::new(&manifest.installation.target_dir);
    let bin_name = &manifest.installation.bin_name;
    let bin_filename = if cfg!(windows) { format!("{}.exe", bin_name) } else { bin_name.clone() };
    let bin_path = target_dir.join(&bin_filename);

    // Detect placeholder manifest (from onix init, never updated by publish)
    if source.sha256 == "PLACEHOLDER" || source.url.contains("OWNER/REPO") || source.url.contains("v0.0.0") {
        return Err(anyhow::anyhow!(
            "This package's manifest has not been updated yet. \
             The owner needs to run `onix publish` to generate real download URLs and checksums. \
             (Detected placeholder values in manifest for {} v{})",
            manifest.app, manifest.version
        ));
    }

    println!("\n📋 Install Plan:");
    println!("   App:        {} v{}", manifest.app, manifest.version);
    println!("   Binary:     {}", bin_filename);
    println!("   Target:     {}", target_dir.display());
    println!("   URL:        {}", source.url);
    println!("   SHA256:     {}", &source.sha256[..source.sha256.len().min(16)]);

    if let Some(msg) = &manifest.message {
        println!("   Message:    {}", msg);
    }

    // 5. Ensure target directory exists
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create target directory {}", target_dir.display()))?;
        println!("\n📁 Created target directory: {}", target_dir.display());
    }

    // 6. Archive existing version if present
    let deprecated_dir = target_dir.join("deprecated");
    if bin_path.exists() {
        println!("🔄 Existing version detected. Archiving...");

        if !deprecated_dir.exists() {
            fs::create_dir_all(&deprecated_dir)?;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();

        let old_bin_filename = bin_path.file_name().unwrap().to_string_lossy();
        let backup_name = format!("{}.{}", old_bin_filename, timestamp);
        let backup_path = deprecated_dir.join(&backup_name);

        fs::rename(&bin_path, &backup_path)
            .context("Failed to move current binary to deprecated folder")?;

        println!("✅ Archived old version to {}", backup_path.display());
    }

    // 7. Download and verify
    println!("⬇️  Downloading binary...");
    let bytes = network::download_artifact(&source.url, &source.sha256).await
        .context("Download or checksum verification failed")?;

    println!("✅ Checksum verified (SHA256 match)");

    // 8. Write binary to target
    fs::write(&bin_path, &bytes)
        .with_context(|| format!("Failed to write binary to {}", bin_path.display()))?;

    // 9. Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))
            .context("Failed to set executable permissions")?;
    }

    println!("✅ Installed {} v{} to {}", manifest.app, manifest.version, bin_path.display());

    // 10. Add target dir to PATH if not already present
    let path_str = target_dir.to_str().context("Invalid target directory path")?;
    env::add_to_path(Path::new(path_str))?;
    println!("✅ Ensured {} is in PATH", target_dir.display());

    if let Some(msg) = &manifest.message {
        println!("\n💡 {}", msg);
    }

    Ok(())
}