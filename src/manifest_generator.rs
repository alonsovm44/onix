use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize};

// Conceptual structure for .onix/config.yaml
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub app: AppInfo,
    pub build: BuildConfig,
    pub targets: Vec<TargetConfig>,
    pub install: InstallConfig,
    pub permissions: Vec<PermissionConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BuildConfig {
    pub entry: String,
    pub command: String,
    pub output_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetConfig {
    pub os: String,
    pub arch: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallConfig {
    pub file_type: String,
    pub target_dir: String,
    pub bin_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PermissionConfig {
    #[serde(rename = "type")]
    pub r#type: String,
    pub action: String,
    pub path: Option<String>,
    pub variable: Option<String>,
}

// Conceptual structure for install.onix
#[derive(Debug, Serialize, Deserialize)]
pub struct InstallManifest {
    pub schema: String,
    pub app: String,
    pub version: String,
    #[serde(rename = "install-on")]
    pub install_on: Vec<InstallOnEntry>,
    pub installation: InstallConfig,
    pub permissions: Vec<PermissionConfig>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallOnEntry {
    pub os: String,
    pub arch: String,
    pub url: String,
    pub sha256: String,
}

/// Generates the content for the `install.onix` manifest.
pub fn generate_install_manifest(
    config: &AppConfig,
    owner: &str,
    repo: &str,
    tag: &str,
    binary_checksums: &HashMap<(String, String), String>, // (os, arch) -> sha256
) -> Result<String, Box<dyn std::error::Error>> {
    let mut install_on_entries = Vec::new();
    for target in &config.targets {
        let filename = format!("{}-{}-{}", config.build.output_name, target.os, target.arch);
        let url = format!("https://github.com/{}/{}/releases/download/{}/{}", owner, repo, tag, filename);
        let sha256 = binary_checksums.get(&(target.os.clone(), target.arch.clone()))
            .ok_or_else(|| format!("SHA256 not found for target: {}/{}", target.os, target.arch))?.clone();
        install_on_entries.push(InstallOnEntry { os: target.os.clone(), arch: target.arch.clone(), url, sha256 });
    }
    let manifest = InstallManifest { schema: "1.0.0".to_string(), app: config.app.name.clone(), version: config.app.version.clone(), install_on: install_on_entries, installation: config.install.clone(), permissions: config.permissions.clone(), message: format!("Run `{}` to get started", config.install.bin_name) };
    serde_yaml::to_string(&manifest).map_err(Into::into)
}

/// Calculates the SHA256 checksum of a given file.
pub fn calculate_sha256(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    use sha2::{Sha256, Digest};
    let mut file = std::fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}