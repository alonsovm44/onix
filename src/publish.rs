use crate::models::{ProjectConfig, OnixManifest, BuildTarget};
use anyhow::{Context, Result};
use serde::Serialize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[derive(Serialize, Default)]
struct DebugReport {
    config: Option<ProjectConfig>,
    git_ops: Vec<GitOpLog>,
    repo_slug: Option<String>,
    workflow_run: Option<serde_json::Value>,
    jobs: Vec<serde_json::Value>,
    assets: Vec<String>,
}

#[derive(Serialize)]
struct GitOpLog {
    description: String,
    args: Vec<String>,
    success: bool,
}

pub async fn execute(version_arg: Option<String>, debug: bool, dry_run: bool) -> Result<()> {
    let mut report = DebugReport::default();
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

    if debug {
        report.config = Some(config.clone());
    }

    let target_version = if let Some(v) = version {
        Some(v.to_string())
    } else if let Some(level) = bump {
        Some(bump_version(&config.app.version, level)?)
    } else {
        None
    };

    if let Some(new_version) = target_version {
        let old_version = config.app.version.clone();
        update_files_version(config_path, &mut config, &new_version, &old_version, dry_run)?;
    }

    let final_version = config.app.version.clone();
    let tag_name = format!("v{}", final_version);
    let mut force = false;

    // Pre-flight check: Detect existing tag locally or remotely
    let local_exists = !Command::new("git").args(["tag", "-l", &tag_name]).output()?.stdout.is_empty();
    let remote_exists = !Command::new("git").args(["ls-remote", "--tags", "origin", &tag_name]).output()?.stdout.is_empty();

    if (local_exists || remote_exists) && !dry_run {
        print!("\n⚠️  Release {} already exists. Overwrite? (y/N): ", tag_name);
        io::stdout().flush()?;
        let mut response = String::new();
        io::stdin().read_line(&mut response)?;
        if response.trim().to_lowercase() != "y" {
            anyhow::bail!("Publication aborted: version {} already exists.", final_version);
        }
        force = true;
        println!("🔥 Overwrite confirmed. Proceeding with force-update...");
    }

    execute_git_commands(&final_version, force, if debug { Some(&mut report.git_ops) } else { None }, dry_run)?;

    // Pre-flight check: Ensure we are in a git repo and have a remote
    let remote_url = get_git_remote().unwrap_or_else(|_| "unknown".to_string());
    let repo_slug = extract_github_slug(&remote_url);
    
    report.repo_slug = repo_slug.clone();

    println!("🚀 Preparing publication for {} v{}", config.app.name, config.app.version);
    println!("\n📊 Generated Compilation Matrix:");
    
    println!("{:<12} | {:<8} | {:<15} | {:<10}", "OS", "Arch", "Runner", "Status");
    println!("{:-<12}-|-{:-<8}-|-{:-<15}-|-{:-<10}", "", "", "", "");

    for target in &config.targets {
        println!("{:<12} | {:<8} | {:<15} | {:<10}", 
            target.os, target.arch, get_runner_for_target(target), "PENDING");
    }

    if repo_slug.is_none() {
        anyhow::bail!("Could not determine GitHub repository slug from remote URL: {}", remote_url);
    }
    let slug = repo_slug.unwrap();

    if dry_run {
        println!("\n✨ Dry run complete. No actions were performed on the repository or files.");
        return Ok(());
    }

    // Start Polling Logic
    let res = poll_github_actions(&slug, &tag_name, &config, if debug { Some(&mut report) } else { None }).await;

    if debug {
        let json_report = serde_json::to_string_pretty(&report)?;
        println!("\n🛠️  DEBUG REPORT (JSON):\n{}", json_report);

        fs::write("onix-debug-report.json", &json_report).context("Failed to write debug report to file")?;
        println!("\n💾 Debug report saved to: onix-debug-report.json");
    }

    res
}

async fn poll_github_actions(slug: &str, tag: &str, config: &ProjectConfig, mut report: Option<&mut DebugReport>) -> Result<()> {
    let client = reqwest::Client::builder()
        .user_agent("onix-cli/1.0.0")
        .build()?;
    
    let start_time = Instant::now();
    let timeout = Duration::from_secs(600); // 10 minutes
    let mut last_check = Instant::now();

    println!("\n⏳ Waiting for GitHub Actions to complete (Tag: {})...", tag);
    
    loop {
        if start_time.elapsed() > timeout {
            anyhow::bail!("Timeout reached while waiting for GitHub Actions.");
        }

        // Rate limit local polling to every 10 seconds
        if last_check.elapsed() < Duration::from_secs(10) {
            tokio::time::sleep(Duration::from_millis(500)).await;
            continue;
        }
        last_check = Instant::now();

        // 1. Find the Workflow Run for this tag
        let url = format!("https://api.github.com/repos/{}/actions/runs?event=push&branch={}", slug, tag);
        let resp = client.get(&url).send().await?;
        let data: serde_json::Value = resp.json().await?;

        let runs = data["workflow_runs"].as_array().context("Failed to parse workflow runs")?;
        let run = runs.iter().find(|r| r["head_branch"] == tag || r["name"].as_str().unwrap_or("").contains(tag));

        if let Some(r) = run {
            let status = r["status"].as_str().unwrap_or("unknown");
            let conclusion = r["conclusion"].as_str().unwrap_or("pending");
            let run_id = r["id"].as_u64().unwrap_or(0);

            if let Some(ref mut rep) = report {
                rep.workflow_run = Some(r.clone());
            }

            // 2. Fetch specific jobs to see which target is failing
            let jobs_url = format!("https://api.github.com/repos/{}/actions/runs/{}/jobs", slug, run_id);
            let jobs_resp = client.get(&jobs_url).send().await?;
            let jobs_data: serde_json::Value = jobs_resp.json().await?;
            let jobs = jobs_data["jobs"].as_array().context("Failed to parse jobs")?;

            if let Some(ref mut rep) = report {
                rep.jobs = jobs.clone();
            }

            let mut completed_count = 0;
            let total_targets = config.targets.len();

            for job in jobs {
                let name = job["name"].as_str().unwrap_or("unknown");
                let job_status = job["status"].as_str().unwrap_or("unknown");
                let job_conclusion = job["conclusion"].as_str().unwrap_or("none");

                if job_status == "completed" {
                    completed_count += 1;
                    if job_conclusion == "failure" {
                        println!("  ❌ Job '{}' failed! Check GHA logs for details.", name);
                    }
                }
            }

            print!("\r🔍 Progress: [{}/{}] | Run Status: {} ({})          ", 
                completed_count, total_targets, status, conclusion);
            io::stdout().flush()?;

            if status == "completed" {
                println!("\n✅ All GitHub Actions jobs finished.");
                let mut asset_list = Vec::new();
                let update_res = update_manifest_hashes_from_release(slug, tag, if report.is_some() { Some(&mut asset_list) } else { None }).await;
                if let Some(ref mut rep) = report {
                    rep.assets = asset_list;
                }
                return update_res;
            }
        }
    }
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

fn update_files_version(config_path: &str, config: &mut ProjectConfig, new_version: &str, old_version: &str, dry_run: bool) -> Result<()> {
    println!("🔄 Updating version: {} -> {}", old_version, new_version);
    
    config.app.version = new_version.to_string();
    if dry_run {
        println!("  [DRY RUN] Would update version in {}", config_path);
        return Ok(());
    }

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

fn execute_git_commands(version: &str, force: bool, mut log: Option<&mut Vec<GitOpLog>>, dry_run: bool) -> Result<()> {
    println!("📦 Automating Git release for v{}...", version);
    let tag = format!("v{}", version);
    let commit_msg = format!("Release {}", tag);

    // Check if there are changes to commit
    let status = Command::new("git").args(["status", "--porcelain"]).output()?;
    let is_dirty = !status.stdout.is_empty();

    let tag_args = if force { vec!["tag", "-f", &tag] } else { vec!["tag", &tag] };
    let push_tag_args = if force { vec!["push", "origin", "-f", &tag] } else { vec!["push", "origin", &tag] };

    let mut cmds: Vec<(Vec<&str>, &str)> = Vec::new();
    
    if is_dirty {
        cmds.push((vec!["add", "."], "Staging changes"));
        cmds.push((vec!["commit", "-m", &commit_msg], "Committing"));
        cmds.push((vec!["push"], "Pushing to master"));
    } else {
        println!("  > Working tree clean, skipping commit.");
    }

    cmds.push((tag_args, "Creating/Updating tag"));
    cmds.push((push_tag_args, "Pushing tag to origin"));

    for (args, desc) in cmds {
        let cmd_args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        
        let success = if dry_run {
            println!("  [DRY RUN] git {}", args.join(" "));
            true
        } else {
            println!("  > {}...", desc);
            Command::new("git").args(&args).status()?.success()
        };
        
        if let Some(ref mut l) = log {
            l.push(GitOpLog {
                description: desc.to_string(),
                args: cmd_args,
                success,
            });
        }
        if !success {
            anyhow::bail!("Git command failed during: {}", desc);
        }
    }
    println!("✅ Git release completed.");
    Ok(())
}

async fn update_manifest_hashes_from_release(slug: &str, tag: &str, mut asset_log: Option<&mut Vec<String>>) -> Result<()> {
    println!("📥 Fetching remote artifacts to verify checksums...");
    let client = reqwest::Client::builder().user_agent("onix-cli/1.0.0").build()?;
    let url = format!("https://api.github.com/repos/{}/releases/tags/{}", slug, tag);
    
    let resp = client.get(&url).send().await?;
    let release: serde_json::Value = resp.json().await?;
    
    let assets = release["assets"].as_array().context("No assets found in release")?;
    
    // Log found assets to debug the Mac x86_64 issue
    println!("🔎 Found {} assets in release:", assets.len());
    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("unknown").to_string();
        println!("  - {}", name);
        if let Some(ref mut log) = asset_log {
            log.push(name);
        }
    }

    // Here you would find the checksums.txt if you generate it in GHA, 
    // or download each binary and hash them manually.
    if let Some(checksum_asset) = assets.iter().find(|a| a["name"] == "checksums.txt") {
        let download_url = checksum_asset["browser_download_url"].as_str().unwrap();
        // Proceed to call your existing update_manifest_hashes logic
        println!("✨ Found checksums.txt, syncing manifest...");
    }

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
    if url.contains("github.com") {
        let clean_url = url.trim_end_matches(".git");
        if clean_url.contains("https://") {
            let parts: Vec<&str> = clean_url.split('/').collect();
            if parts.len() >= 2 {
                let repo = parts.last()?;
                let owner = parts.get(parts.len() - 2)?;
                return Some(format!("{}/{}", owner, repo));
            }
        } else {
            // SSH format: git@github.com:owner/repo
            let parts: Vec<&str> = clean_url.split(':').collect();
            return parts.last().map(|s| s.to_string());
        }
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