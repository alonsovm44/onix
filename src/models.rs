use serde::{Deserialize, Serialize};
//use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnixManifest {
    pub schema: String,
    pub app: String,
    pub version: String,
    #[serde(rename = "install-on")]
    pub install_on: Vec<PlatformSource>,
    pub installation: Installation,
    pub permissions: Vec<Permission>,
    pub message: Option<String>,
}

impl OnixManifest {
    /// Finds the correct download source based on the current system's OS and architecture.
    pub fn find_source(&self) -> Option<&PlatformSource> {
        let current_os = std::env::consts::OS;
        let current_arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            arch => arch,
        };

        self.install_on
            .iter()
            .find(|source| source.os == current_os && source.arch == current_arch)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlatformSource {
    pub os: String,
    pub arch: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Permission {
    #[serde(rename = "type")]
    pub permission_type: String,
    pub action: String,
    pub path: Option<String>, // For filesystem permissions
    pub variable: Option<String>, // For environment permissions
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Installation {
    #[serde(rename = "file-type")]
    pub file_type: String, // e.g., "binary"
    #[serde(rename = "target-dir")]
    pub target_dir: String,
    #[serde(rename = "bin-name")]
    pub bin_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectConfig {
    pub app: AppConfig,
    pub build: BuildConfig,
    pub targets: Vec<BuildTarget>,
    pub install: Installation,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
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
pub struct BuildTarget {
    pub os: String,
    pub arch: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runner: Option<String>,
}