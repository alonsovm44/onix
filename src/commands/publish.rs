use std::{fs, env, io, fs::OpenOptions, io::Write};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use anyhow::{Context, Result, anyhow};
use git2::Repository;
use regex::Regex;
use tokio::time::sleep;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute as cross_execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use crate::manifest_generator::{AppConfig};

pub async fn execute() -> Result<()> {
    // 1. Load configuration to get the current version
    let config_path = Path::new(".onix/config.yaml");
    if !config_path.exists() {
        return Err(anyhow!("Not an Onix project. Please run 'onix init' first."));
    }

    let config_content = fs::read_to_string(config_path)
        .context("Failed to read .onix/config.yaml")?;
    let config: AppConfig = serde_yaml::from_str(&config_content)
        .context("Failed to parse .onix/config.yaml")?;

    let tag_name = format!("v{}", config.app.version);
    println!("🚀 Preparing to publish {} version {}...", config.app.name, tag_name);

    // 2. Open the Git repository
    let repo = Repository::open(".")
        .context("Failed to open git repository. 'onix publish' must be run inside a git repo.")?;

    // 3. Automated Git Workflow: Stage, Commit, Push, Tag, and Push Tag
    println!("📦 Staging and committing changes..."); 
    run_git(&["add", "."])?;
    // Ignore commit error if there's nothing new to commit
    let _ = run_git(&["commit", "-m", &format!("release: {}", tag_name)]);
    run_git(&["push"])?;

    println!("🏷️ Creating and pushing tag {}...", tag_name);
    run_git(&["tag", "-f", &tag_name])?;
    run_git(&["push", "origin", &tag_name, "-f"])?;

    // 5. Extract GitHub Info
    let (owner, repo_name) = get_repo_remote_info(&repo)?;
    println!("📦 Repository identified: {}/{}", owner, repo_name);

    let token = get_github_token()?;
    let octo = octocrab::Octocrab::builder()
        .personal_token(token)
        .build()
        .context("Failed to initialize GitHub client")?;

    // 6. Poll GitHub Actions
    poll_ci_status(&octo, &owner, &repo_name, &tag_name).await?;

    // 7. Fetch release and compute hashes
    println!("🔍 Fetching release artifacts for verification...");
    let release = octo.repos(&owner, &repo_name)
        .releases()
        .get_by_tag(&tag_name)
        .await
        .context("Could not find release for the pushed tag")?;

    let checksums = fetch_and_hash_assets(&config, &release).await?;

    // 8. Generate and upload install.onix
    println!("📝 Generating automated install manifest...");
    let manifest_content = crate::manifest_generator::generate_install_manifest(
        &config, &owner, &repo_name, &tag_name, &checksums
    ).map_err(|e| anyhow!(e.to_string()))?;

println!("📤 Uploading install.onix to GitHub Release...");
    octo.repos(&owner, &repo_name)
        .releases()
        .upload_asset(release.id.0, "install.onix", manifest_content.into())
        .send() 
        .await
        .context("Failed to upload manifest to release")?;
        
    println!("🚀 Version {} successfully published!", config.app.version);
    Ok(())
}

/// Extracts GitHub owner and repo name from the 'origin' remote URL.
fn get_repo_remote_info(repo: &Repository) -> Result<(String, String)> {
    let remote = repo.find_remote("origin")
        .context("Could not find git remote 'origin'.")?;
    
    let url = remote.url()
        .context("Remote 'origin' has no URL defined.")?;

    // Regex covers:
    // https://github.com/owner/repo.git
    // git@github.com:owner/repo.git
    let re = Regex::new(r"github\.com[:/](?P<owner>[^/]+)/(?P<repo>[^/\.]+)(\.git)?$")?;
    
    let caps = re.captures(url)
        .ok_or_else(|| anyhow!("Could not parse GitHub owner/repo from URL: {}", url))?;

    Ok((
        caps["owner"].to_string(),
        caps["repo"].to_string()
    ))
}

/// Polls GitHub Actions until the workflow run for the specific tag completes.
async fn poll_ci_status(octo: &octocrab::Octocrab, owner: &str, repo: &str, tag: &str) -> Result<()> {
    // Setup TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    cross_execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut progress = 0;
    let mut status_msg = String::from("Initializing poll...");
    let mut last_api_poll = Instant::now() - Duration::from_secs(5); // Trigger first poll immediately

    // Use an async block to ensure terminal cleanup happens even on error
    let poll_result: Result<()> = async {
        loop {
            // 1. Handle user input (runs every iteration)
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        // Detect 'q' OR 'Ctrl+C'
                        let is_ctrl_c = key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
                        if key.code == KeyCode::Char('q') || is_ctrl_c {
                            return Err(anyhow!("Publishing aborted by user."));
                        }
                    }
                }
            }

            // 2. Draw TUI
            terminal.draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(2)
                    .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                    .split(size);

                let gauge = Gauge::default()
                    .block(Block::default().title("CI Build Progress").borders(Borders::ALL))
                    .gauge_style(Style::default().fg(Color::Cyan))
                    .percent(progress);
                f.render_widget(gauge, chunks[0]);

                // Add a simple animated "polling" indicator so the UI doesn't look frozen
                let dots = ".".repeat((Instant::now().elapsed().as_secs() % 4) as usize);
                let text = format!("Status: {}{}\n(Press 'q' or Ctrl+C to abort)", status_msg, dots);
                let paragraph = Paragraph::new(text)
                    .block(Block::default().title("Activity").borders(Borders::ALL));
                f.render_widget(paragraph, chunks[1]);
            })?;
            
            // 3. Poll GitHub API
            if last_api_poll.elapsed() >= Duration::from_secs(5) {
                let runs = octo.workflows(owner, repo)
                    .list_all_runs()
                    .send()
                    .await
                    .context("Failed to fetch workflow runs from GitHub")?;

                let target_run = runs.items.iter().find(|r| {
                    r.head_branch == tag || r.head_sha == tag
                });

                match target_run {
                    Some(run) => {
                        match run.status.as_str() {
                            "completed" => {
                                if run.conclusion.as_deref() == Some("success") {
                                    progress = 100;
                                    status_msg = "✅ CI Build Completed Successfully!".into();
                                    sleep(Duration::from_secs(1)).await;
                                    return Ok(());
                                } else {
                                    return Err(anyhow!("CI failed: {:?}", run.conclusion));
                                }
                            }
                            _ => {
                                progress = 50;
                                status_msg = format!("🔨 CI Status: {}", run.status);
                            }
                        }
                    }
                    None => status_msg = "⏳ Waiting for GitHub to register the tag...".into(),
                }
                last_api_poll = Instant::now();
            }
        }
    }.await;

    // Teardown TUI: This code is now guaranteed to run
    let _ = disable_raw_mode();
    let _ = cross_execute!(io::stdout(), LeaveAlternateScreen);
    
    poll_result
}

async fn fetch_and_hash_assets(
    config: &AppConfig,
    release: &octocrab::models::repos::Release,
) -> Result<HashMap<(String, String), String>> {
    let mut checksums = HashMap::new();
    let client = reqwest::Client::new();
    let temp_dir = env::temp_dir().join("onix_publish");
    fs::create_dir_all(&temp_dir)?;

    for target in &config.targets {
        let asset_name = format!("{}-{}-{}", config.build.output_name, target.os, target.arch);
        let asset = release.assets.iter()
            .find(|a| a.name == asset_name || a.name == format!("{}.exe", asset_name))
            .ok_or_else(|| anyhow!("Asset {} not found in release", asset_name))?;

        let file_path = temp_dir.join(&asset.name);
        let resp = client.get(asset.browser_download_url.clone()).send().await?;
        let bytes = resp.bytes().await?;
        fs::write(&file_path, bytes)?;

        let hash = crate::manifest_generator::calculate_sha256(&file_path)
            .map_err(|e| anyhow!(e.to_string()))?;
        
        checksums.insert((target.os.clone(), target.arch.clone()), hash);
        fs::remove_file(file_path)?;
    }
    Ok(checksums)
}

/// Helper to run git commands and handle errors.
fn run_git(args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .status()
        .context(format!("Failed to execute 'git {}'", args.join(" ")))?;

    if !status.success() {
        return Err(anyhow!("Git command failed: git {}", args.join(" ")));
    }
    Ok(())
}

/// Retrieves the GitHub token from the environment or prompts the user with masked input.
fn get_github_token() -> Result<String> {
    // 1. Check Environment Variable
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        return Ok(token);
    }

    // 2. Check Local File
    let token_path = Path::new(".onix/token.key");
    if token_path.exists() {
        let token = fs::read_to_string(token_path)?.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    println!("🔑 GITHUB_TOKEN not found in environment.");
    println!("Please enter a GitHub Personal Access Token (input will be hidden):");
    io::stdout().flush()?;

    enable_raw_mode()?;
    
    // Drain any leftover events (like an Enter release from a previous command)
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read();
    }

    let mut token = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Enter => break,
                KeyCode::Char(c) => token.push(c),
                KeyCode::Backspace => { token.pop(); }
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    return Err(anyhow!("Operation cancelled by user."));
                }
                _ => {}
            }
        }
    }
    disable_raw_mode()?;
    println!();

    let token = token.trim().to_string();
    if token.is_empty() {
        return Err(anyhow!("GitHub token cannot be empty."));
    }

    // 3. Save for future use
    save_token(&token)?;
    Ok(token)
}

/// Saves the token to .onix/token.key and ensures it is ignored by git.
fn save_token(token: &str) -> Result<()> {
    let dot_onix = Path::new(".onix");
    if !dot_onix.exists() {
        fs::create_dir_all(dot_onix)?;
    }

    let token_file = dot_onix.join("token.key");
    fs::write(&token_file, token)?;
    
    // Ensure the token is in .gitignore
    let gitignore_path = Path::new(".gitignore");
    let pattern = ".onix/token.key";
    
    let mut content = if gitignore_path.exists() {
        fs::read_to_string(gitignore_path)?
    } else {
        String::new()
    };

    if !content.lines().any(|l| l.trim() == pattern) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(gitignore_path)?;
        
        if !content.is_empty() && !content.ends_with('\n') {
            writeln!(file)?;
        }
        writeln!(file, "{}", pattern)?;
        println!("🛡️  Added {} to .gitignore", pattern);
    }

    println!("💾 Token saved locally to {}", token_file.display());
    Ok(())
}