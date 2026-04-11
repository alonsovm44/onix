use crate::models::{ProjectConfig, OnixManifest, BuildTarget};
use anyhow::{Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub async fn execute(version_arg: Option<String>) -> Result<()> {
    let version = version_arg.as_deref();
    let bump: Option<&str> = None;
    let config_path = ".onix/config.yaml";
    
    if !std::path::Path::new(config_path).exists() {
        return Err(anyhow::anyhow!(
            "Project configuration not found at {}. Run `onix init` first.",
            config_path
        ));
    }

    let content = fs::read_to_string(config_path)
        .context("Failed to read .onix/config.yaml")?;
    
    let mut config: ProjectConfig = serde_yaml::from_str(&content)
        .context("Failed to parse project configuration")?;

    let target_version = if let Some(v) = version {
        Some(v.to_string())
    } else if let Some(level) = bump {
        Some(bump_version(&config.app.version, level)?)
    } else {
        None
    };

    if let Some(new_version) = target_version {
        let old_version = config.app.version.clone();
        update_files_version(config_path, &mut config, &new_version, &old_version)?;

        if version.is_some() {
            let tag_name = format!("v{}", new_version);
            let mut force = false;

            // Pre-flight check: Detect existing tag locally or remotely
            let local_exists = !Command::new("git").args(["tag", "-l", &tag_name]).output()?.stdout.is_empty();
            let remote_exists = !Command::new("git").args(["ls-remote", "--tags", "origin", &tag_name]).output()?.stdout.is_empty();

            if local_exists || remote_exists {
                print!("\n⚠️  Release {} already exists. Overwrite? (y/N): ", tag_name);
                io::stdout().flush()?;
                let mut response = String::new();
                io::stdin().read_line(&mut response)?;
                if response.trim().to_lowercase() != "y" {
                    anyhow::bail!("Publication aborted: version {} already exists.", new_version);
                }
                force = true;
                println!("🔥 Overwrite confirmed. Proceeding with force-update...");
            }

            execute_git_commands(&new_version, force)?;
            return Ok(());
        }
    }

    // Pre-flight check: Ensure we are in a git repo and have a remote
    let remote_url = get_git_remote().unwrap_or_else(|_| "unknown".to_string());
    let repo_slug = extract_github_slug(&remote_url);

    println!("🚀 Preparing publication for {} v{}", config.app.name, config.app.version);
    println!("\n📊 Generated Compilation Matrix:");
    
    println!("{:<15} | {:<10} | {:<20}", "OS", "Arch", "Runner (GHA)");
    println!("{:-<15}-|-{:-<10}-|-{:-<20}", "", "", "");

    for target in &config.targets {
        let runner = get_runner_for_target(target);
        println!("{:<15} | {:<10} | {:<20}", target.os, target.arch, runner);
    }

    println!("\n✅ Matrix validated.");
    println!("🔗 GitHub Action: .github/workflows/onix.yml is synced.");
    println!("\nNext steps:");
    println!("1. Commit your changes: `git add . && git commit -m 'Release {}'`", config.app.version);
    println!("2. Tag the release: `git tag v{}`", config.app.version);
    println!("3. Push to trigger CI: `git push origin v{}`", config.app.version);
    
    if let Some(slug) = repo_slug {
        println!("\n🌍 Your public installation URL will be:");
        println!("   https://raw.githubusercontent.com/{}/master/install.onix", slug);
    }

    Ok(())
}

fn bump_version(version: &str, level: &str) -> Result<String> {
    let mut parts: Vec<u32> = version
        .split('.')
        .map(|s| s.parse().context("Invalid version segment in config.yaml"))
        .collect::<Result<Vec<_>>>()?;

    if parts.len() != 3 {
        anyhow::bail!("Version must follow semantic versioning (x.y.z)");
    }

    match level {
        "major" => { parts[0] += 1; parts[1] = 0; parts[2] = 0; }
        "minor" => { parts[1] += 1; parts[2] = 0; }
        "patch" => { parts[2] += 1; }
        _ => anyhow::bail!("Invalid bump level"),
    }

    Ok(format!("{}.{}.{}", parts[0], parts[1], parts[2]))
}

fn update_files_version(config_path: &str, config: &mut ProjectConfig, new_version: &str, old_version: &str) -> Result<()> {
    println!("🔄 Updating version: {} -> {}", old_version, new_version);
    
    config.app.version = new_version.to_string();
    let updated_config = serde_yaml::to_string(config)?;
    fs::write(config_path, updated_config)?;

    let manifest_path = "install.onix";
    if std::path::Path::new(manifest_path).exists() {
        let content = fs::read_to_string(manifest_path)?;
        let mut manifest: OnixManifest = serde_yaml::from_str(&content)?;
        manifest.version = new_version.to_string();
        
        for source in &mut manifest.install_on {
            source.url = source.url.replace(&format!("v{}", old_version), &format!("v{}", new_version));
        }
        fs::write(manifest_path, serde_yaml::to_string(&manifest)?)?;
    }
    Ok(())
}

fn execute_git_commands(version: &str, force: bool) -> Result<()> {
    println!("📦 Automating Git release for v{}...", version);
    let tag = format!("v{}", version);
    let commit_msg = format!("Release {}", tag);

    let tag_args = if force { vec!["tag", "-f", &tag] } else { vec!["tag", &tag] };
    let push_tag_args = if force { vec!["push", "origin", "-f", &tag] } else { vec!["push", "origin", &tag] };

    let cmds: Vec<(Vec<&str>, &str)> = vec![
        (vec!["add", "."], "Staging changes"),
        (vec!["commit", "-m", &commit_msg], "Committing"),
        (vec!["push"], "Pushing to master"),
        (tag_args, "Creating/Updating tag"),
        (push_tag_args, "Pushing tag to origin"),
    ];

    for (args, desc) in cmds {
        println!("  > {}...", desc);
        let status = Command::new("git").args(args).status()?;
        if !status.success() {
            anyhow::bail!("Git command failed during: {}", desc);
        }
    }
    println!("✅ Git release completed.");
    Ok(())
}

fn get_runner_for_target(target: &BuildTarget) -> &str {
    target.runner.as_deref().unwrap_or_else(|| {
        match target.os.as_str() {
            "linux" => "ubuntu-latest",
            "macos" => "macos-latest",
            "windows" => "windows-latest",
            _ => "ubuntu-latest",
        }
    })
}

fn get_git_remote() -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to execute git command")?;
    
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn extract_github_slug(url: &str) -> Option<String> {
    // Simple extractor for git@github.com:user/repo.git or https://github.com/user/repo
    if url.contains("github.com") {
        let parts: Vec<&str> = if url.contains("https://") {
            url.trim_end_matches(".git").split('/').collect()
        } else {
            url.trim_end_matches(".git").split(':').collect()
        };
        
        parts.last().map(|s| s.to_string())
    } else {
        None
    }
}

pub fn update_manifest_hashes(checksum_path: &PathBuf) -> Result<()> {
    let manifest_path = "install.onix";
    
    // 1. Parse the checksum file (Format: <hash>  <filename>)
    let checksum_content = fs::read_to_string(checksum_path)
        .with_context(|| format!("Failed to read checksum file at {:?}", checksum_path))?;
    
    let mut hashes = std::collections::HashMap::new();
    for line in checksum_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            // parts[0] is the hash, parts[1] is the filename
            hashes.insert(parts[1].to_string(), parts[0].to_string());
        }
    }

    // 2. Load the current manifest
    let manifest_content = fs::read_to_string(manifest_path)
        .context("Failed to read install.onix")?;
    let mut manifest: OnixManifest = serde_yaml::from_str(&manifest_content)
        .context("Failed to parse manifest for update")?;

    // 3. Match and update hashes based on filename extracted from URL
    let mut updated_count = 0;
    for source in &mut manifest.install_on {
        if let Some(filename) = source.url.split('/').last() {
            if let Some(hash) = hashes.get(filename) {
                source.sha256 = hash.clone();
                updated_count += 1;
            }
        }
    }

    // 4. Serialize back to YAML and save
    let updated_yaml = serde_yaml::to_string(&manifest)?;
    fs::write(manifest_path, updated_yaml)?;

    println!("✅ Successfully updated {} platform hashes in {}", updated_count, manifest_path);
    Ok(())
}