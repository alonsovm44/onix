use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct OnixManifest {
    pub schema: String,
    pub app: String,
    pub version: String,
    #[serde(rename = "install-on")]
    pub install_on: Vec<Source>,
    pub installation: Installation,
    pub permissions: Vec<String>,
    pub message: Option<String>,
}

impl OnixManifest {
    /// Finds the correct download source based on the current system's OS and architecture.
    pub fn find_source(&self) -> Option<&Source> {
        let current_os = std::env::consts::OS;
        let current_arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "aarch64" => "arm64",
            arch => arch,
        };

        self.install_on
            .iter()
            .find(|s| s.os == current_os && s.arch == current_arch)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub os: String,
    pub arch: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Installation {
    #[serde(rename = "file-type")]
    pub file_type: String, // e.g., "binary"
    #[serde(rename = "target-dir")]
    pub target_dir: String,
    #[serde(rename = "bin-name")]
    pub bin_name: String,
}