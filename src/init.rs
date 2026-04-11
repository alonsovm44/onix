use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use crate::manifest_generator::{
    AppConfig, AppInfo, BuildConfig, InstallConfig, PermissionConfig, TargetConfig,
};

pub fn execute() -> Result<()> {
    let onix_dir = Path::new(".onix");
    let config_path = onix_dir.join("config.yaml");

    if config_path.exists() {
        println!("✨ Onix is already initialized (found .onix/config.yaml)");
        return Ok(());
    }

    println!("Initializing Onix project...");

    if !onix_dir.exists() {
        fs::create_dir(onix_dir).context("Failed to create .onix directory")?;
    }

    // Use the current directory name as the default app name
    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("my-app")
        .to_string();

    let default_config = AppConfig {
        app: AppInfo {
            name: project_name.clone(),
            version: "0.1.0".to_string(),
        },
        build: BuildConfig {
            entry: "src/main.rs".to_string(),
            command: "cargo build --release".to_string(),
            output_name: project_name.clone(),
        },
        targets: vec![
            TargetConfig { os: "linux".to_string(), arch: "amd64".to_string() },
            TargetConfig { os: "linux".to_string(), arch: "arm64".to_string() },
            TargetConfig { os: "macos".to_string(), arch: "arm64".to_string() },
            TargetConfig { os: "windows".to_string(), arch: "amd64".to_string() },
        ],
        install: InstallConfig {
            file_type: "binary".to_string(),
            target_dir: "~/.local/bin".to_string(),
            bin_name: project_name,
        },
        permissions: vec![
            PermissionConfig {
                r#type: "filesystem".to_string(),
                action: "write".to_string(),
                path: Some("~/.local/bin".to_string()),
                variable: None,
            },
            PermissionConfig {
                r#type: "environment".to_string(),
                action: "modify".to_string(),
                path: None,
                variable: Some("PATH".to_string()),
            },
        ],
    };

    let yaml = serde_yaml::to_string(&default_config).context("Failed to serialize default configuration")?;
    fs::write(config_path, yaml).context("Failed to write .onix/config.yaml")?;

    println!("🚀 Successfully initialized! Edit .onix/config.yaml to customize your distribution.");
    Ok(())
}