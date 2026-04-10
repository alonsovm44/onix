use anyhow::{Context, Result};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Adds a directory to the system PATH environment variable across different platforms.
pub fn add_to_path(path: &Path) -> Result<()> {
    let path_str = path.to_str().context("Invalid path provided for environment update")?;

    match env::consts::OS {
        "windows" => add_to_path_windows(path_str),
        "linux" | "macos" => add_to_path_unix(path_str),
        _ => Err(anyhow::anyhow!("Unsupported operating system for PATH management: {}", env::consts::OS)),
    }
}

fn add_to_path_windows(new_path: &str) -> Result<()> {
    // Get the current User PATH to check for existence using PowerShell.
    // PowerShell is used to avoid the 1024-character limitation of the 'setx' command.
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "[Environment]::GetEnvironmentVariable('Path', 'User')",
        ])
        .output()
        .context("Failed to fetch current Windows User PATH")?;

    let current_path = String::from_utf8_lossy(&output.stdout);
    
    // Check if the path is already present to avoid redundant entries.
    if current_path.split(';').any(|p| p == new_path) {
        return Ok(());
    }

    // Append to User PATH using PowerShell.
    let script = format!(
        "$oldPath = [Environment]::GetEnvironmentVariable('Path', 'User'); \
         $newPath = if ($oldPath -eq $null -or $oldPath -eq '') {{ '{}' }} else {{ $oldPath + ';' + '{}' }}; \
         [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')",
        new_path, new_path
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .status()
        .context("Failed to update Windows PATH environment variable via PowerShell")?;

    if !status.success() {
        return Err(anyhow::anyhow!("PowerShell exited with a non-zero status while updating PATH"));
    }

    Ok(())
}

fn add_to_path_unix(new_path: &str) -> Result<()> {
    let home = env::var("HOME").context("HOME environment variable is not set")?;
    let home_path = PathBuf::from(home);
    
    // Determine the user's shell to select the correct configuration file.
    let shell_path = env::var("SHELL").unwrap_or_default();
    
    let (config_file, export_cmd) = if shell_path.contains("zsh") {
        (home_path.join(".zshrc"), format!("\nexport PATH=\"$PATH:{}\"", new_path))
    } else if shell_path.contains("fish") {
        (home_path.join(".config/fish/config.fish"), format!("\nfish_add_path {}", new_path))
    } else if shell_path.contains("bash") {
        // Use .bashrc for typical interactive shell usage.
        (home_path.join(".bashrc"), format!("\nexport PATH=\"$PATH:{}\"", new_path))
    } else {
        // Fallback to .profile for generic POSIX shells.
        (home_path.join(".profile"), format!("\nexport PATH=\"$PATH:{}\"", new_path))
    };

    // Ensure parent directories exist (crucial for fish config).
    if let Some(parent) = config_file.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).context("Failed to create configuration directory")?;
        }
    }

    // Check if the path is already mentioned in the file to avoid duplicates.
    if config_file.exists() {
        let content = fs::read_to_string(&config_file)
            .with_context(|| format!("Failed to read shell config at {:?}", config_file))?;
        if content.contains(new_path) {
            return Ok(());
        }
    }

    // Append the export command to the configuration file.
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_file)
        .with_context(|| format!("Failed to open shell config for writing: {:?}", config_file))?;

    file.write_all(export_cmd.as_bytes())
        .context("Failed to write to shell configuration file")?;

    Ok(())
}