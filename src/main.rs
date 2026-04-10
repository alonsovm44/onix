use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct OnixManifest {
    pub name: String,
    pub version: String,
    pub sources: Vec<Source>,
    pub install: InstallInstructions,
    pub permissions: Vec<Permission>,
    pub env: Option<HashMap<String, EnvOp>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub os: String,
    pub arch: String,
    pub url: String,
    pub sha256: String,
    pub signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallInstructions {
    #[serde(rename = "type")]
    pub install_type: String, // e.g., "archive", "binary"
    pub bin: Vec<String>,
    pub targets: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Permission {
    #[serde(rename = "type")]
    pub perm_type: String, // e.g., "filesystem", "env"
    pub action: String,    // e.g., "write", "modify"
    pub path: Option<String>,
    pub variable: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnvOp {
    pub add: Vec<String>,
    pub method: String, // e.g., "append", "prepend"
}
