use std::fs;
use std::path::Path;
use anyhow::{Context, Result};
use crate::manifest_generator::{
    AppConfig, AppInfo, BuildConfig, InstallConfig, PermissionConfig, TargetConfig,
};
use crate::utils::get_toolset_root;

const RELEASE_WORKFLOW_TEMPLATE: &str = r#"name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build-and-release:
    name: Build and Release
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: windows-x86_64
          - os: ubuntu-latest
            target: linux-x86_64
          - os: macos-latest
            target: macos-arm64
          - os: macos-13
            target: macos-x86_64

    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Build
        run: cargo build --release
      - name: Rename Artifact
        shell: bash
        run: |
          mkdir -p dist
          if [ "${{ matrix.os }}" = "windows-latest" ]; then
            cp target/release/{{BIN_NAME}}.exe dist/{{BIN_NAME}}-${{ matrix.target }}.exe
          else
            cp target/release/{{BIN_NAME}} dist/{{BIN_NAME}}-${{ matrix.target }}
          fi
      - name: Upload to Release
        uses: softprops/action-gh-release@v2
        with:
          files: dist/*
          generate_release_notes: true
"#;

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

    let toolset_root = get_toolset_root();
    let toolset_root_str = toolset_root.to_string_lossy().into_owned();

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
            TargetConfig { os: "linux".to_string(), arch: "x86_64".to_string() },
            TargetConfig { os: "macos".to_string(), arch: "arm64".to_string() },
            TargetConfig { os: "macos".to_string(), arch: "x86_64".to_string() },
            TargetConfig { os: "windows".to_string(), arch: "x86_64".to_string() },
        ],
        install: InstallConfig {
            file_type: "binary".to_string(),
            target_dir: toolset_root_str.clone(),
            bin_name: project_name.clone(),
        },
        permissions: vec![
            PermissionConfig {
                r#type: "filesystem".to_string(),
                action: "write".to_string(),
                path: Some(toolset_root_str.clone()),
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

    // Create the deprecated subfolder in the toolset root
    let deprecated_dir = toolset_root.join("deprecated");
    if !deprecated_dir.exists() {
        fs::create_dir_all(&deprecated_dir).context("Failed to create deprecated directory")?;
    }

    // Create GitHub Actions workflow directory and file
    let workflow_dir = Path::new(".github/workflows");
    if !workflow_dir.exists() {
        fs::create_dir_all(workflow_dir).context("Failed to create .github/workflows directory")?;
    }

    let workflow_path = workflow_dir.join("release.yml");
    let rendered_workflow = RELEASE_WORKFLOW_TEMPLATE.replace("{{BIN_NAME}}", &project_name);
    fs::write(workflow_path, rendered_workflow).context("Failed to write GitHub Action workflow file")?;

    println!("🚀 Successfully initialized at {:?}!", toolset_root);
    println!("💡 Active binaries will be kept in the root, with older versions moved to the 'deprecated' subfolder.");
    Ok(())
}