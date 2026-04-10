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