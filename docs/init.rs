use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub app: AppDetails,
    pub build: BuildDetails,
    pub install: InstallDetails,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppDetails {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildDetails {
    pub entry: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallDetails {
    pub bin_name: String,
    pub target_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    pub r#type: String,
    pub action: String,
    pub path: Option<String>,
    pub variable: Option<String>,
}

pub fn run_init() -> Result<()> {
    println!("❄️ Onix Project Initializer");

    let project_type = detect_project();
    println!("Detected project: {}", project_type);

    let current_dir = std::env::current_dir()?;
    let default_name = current_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("mycli");

    let app_name = prompt("CLI name", default_name)?;
    let version = prompt("Version", "1.0.0")?;
    let entry = prompt("Entry file", "main.rs")?;

    let config = ProjectConfig {
        app: AppDetails {
            name: app_name.clone(),
            version,
        },
        build: BuildDetails {
            entry,
            command: match project_type.as_str() {
                "Rust" => "cargo build --release".to_string(),
                "Go" => format!("go build -o {}", app_name),
                _ => "echo 'Add your build command here'".to_string(),
            },
        },
        install: InstallDetails {
            bin_name: app_name,
            target_dir: "~/.local/bin".to_string(),
        },
        permissions: vec![
            Permission {
                r#type: "filesystem".to_string(),
                action: "write".to_string(),
                path: Some("~/.local/bin".to_string()),
                variable: None,
            },
            Permission {
                r#type: "environment".to_string(),
                action: "modify".to_string(),
                path: None,
                variable: Some("PATH".to_string()),
            },
        ],
    };

    let onix_dir = Path::new(".onix");
    if !onix_dir.exists() {
        fs::create_dir(onix_dir)?;
    }

    let yaml = serde_yaml::to_string(&config)?;
    fs::write(onix_dir.join("config.yaml"), yaml)?;

    println!("\n✅ Generated .onix/config.yaml");

    if prompt("Generate workflow + install.onix?", "Y")?.to_uppercase() == "Y" {
        generate_github_workflow(&config)?;
        println!("✅ Generated .github/workflows/onix.yml");
    }

    Ok(())
}

fn generate_github_workflow(config: &ProjectConfig) -> Result<()> {
    let workflow_dir = Path::new(".github/workflows");
    if !workflow_dir.exists() {
        fs::create_dir_all(workflow_dir).context("Failed to create .github/workflows directory")?;
    }

    let app_name = &config.app.name;
    let build_cmd = &config.build.command;

    let content = format!(
        r#"name: Onix Build & Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    runs-on: ${{{{ matrix.os }}}}

    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            name: linux-amd64
            build_cmd: {build_cmd}
            out: {app_name}

          - os: ubuntu-latest
            name: linux-arm64
            build_cmd: {build_cmd}
            out: {app_name}

          - os: macos-latest
            name: macos-arm64
            build_cmd: {build_cmd}
            out: {app_name}

          - os: windows-latest
            name: windows-amd64
            build_cmd: {build_cmd}
            out: {app_name}

    steps:
      - name: Checkout repo
        uses: actions/checkout@v4

      - name: Build binary
        run: ${{{{ matrix.build_cmd }}}}

      - name: Rename artifact
        run: |
          mkdir -p dist
          mv ${{{{ matrix.out }}}} dist/${{{{ matrix.name }}}} || mv ${{{{ matrix.out }}}}.exe dist/${{{{ matrix.name }}}}

      - name: Generate SHA256
        run: |
          cd dist
          sha256sum * > checksums.txt || certutil -hashfile * SHA256 > checksums.txt

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{{{ matrix.name }}}}
          path: dist/

  release:
    needs: build
    runs-on: ubuntu-latest

    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: dist

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: dist/**/*
        env:
          GITHUB_TOKEN: ${{{{ secrets.GITHUB_TOKEN }}}}
"#,
        build_cmd = build_cmd,
        app_name = app_name
    );

    fs::write(workflow_dir.join("onix.yml"), content).context("Failed to write onix.yml workflow")?;
    Ok(())
}

fn detect_project() -> String {
    if Path::new("Cargo.toml").exists() { "Rust".to_string() }
    else if Path::new("go.mod").exists() { "Go".to_string() }
    else if Path::new("package.json").exists() { "Node.js".to_string() }
    else { "Generic".to_string() }
}

fn prompt(label: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", label, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() { Ok(default.to_string()) } else { Ok(input.to_string()) }
}