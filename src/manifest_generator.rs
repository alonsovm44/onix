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

/// Generates the content for the `install.onix` manifest.
/// Uses models::OnixManifest so the output is directly parseable by `onix install`.
pub fn generate_install_manifest(
    config: &AppConfig,
    owner: &str,
    repo: &str,
    tag: &str,
    binary_checksums: &HashMap<(String, String), String>, // (os, arch) -> sha256
) -> Result<String, Box<dyn std::error::Error>> {
    let mut install_on_entries = Vec::new();
    for target in &config.targets {
        let extension = if target.os == "windows" { ".exe" } else { "" };
        let filename = format!("{}-{}-{}{}", config.build.output_name, target.os, target.arch, extension);
        let url = format!("https://github.com/{}/{}/releases/download/{}/{}", owner, repo, tag, filename);
        match binary_checksums.get(&(target.os.clone(), target.arch.clone())) {
            Some(sha256) => {
                install_on_entries.push(crate::models::PlatformSource { os: target.os.clone(), arch: target.arch.clone(), url, sha256: sha256.clone() });
            }
            None => {
                eprintln!("⚠️  Skipping {}/{} in manifest — no checksum available", target.os, target.arch);
            }
        }
    }

    if install_on_entries.is_empty() {
        return Err("No targets with checksums available for manifest".into());
    }

    // Convert from config types to manifest types (kebab-case field names)
    let installation = crate::models::Installation {
        file_type: config.install.file_type.clone(),
        target_dir: config.install.target_dir.clone(),
        bin_name: config.install.bin_name.clone(),
    };

    let permissions = config.permissions.iter().map(|p| {
        crate::models::Permission {
            permission_type: p.r#type.clone(),
            action: p.action.clone(),
            path: p.path.clone(),
            variable: p.variable.clone(),
        }
    }).collect();

    let manifest = crate::models::OnixManifest {
        schema: "1.0.0".to_string(),
        app: config.app.name.clone(),
        version: config.app.version.clone(),
        install_on: install_on_entries,
        installation,
        permissions,
        message: Some(format!("Run `{}` to get started", config.install.bin_name)),
    };
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