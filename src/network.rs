use crate::models::OnixManifest;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
//use std::io::Write;

/// Resolves a shorthand identifier (e.g., user@repo) or a direct URL
/// into a full manifest URL.
pub async fn resolve_url(input: &str) -> String {
    if input.starts_with("http://") || input.starts_with("https://") {
        return input.to_string();
    }

    if let Some((user, repo)) = input.split_once('@') {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .user_agent("onix-cli/1.0.0")
            .build()
            .unwrap_or_default();

        let main_url = format!("https://raw.githubusercontent.com/{}/{}/main/.onix/install.onix", user, repo);
        
        // Check if the 'main' branch contains the manifest
        if let Ok(resp) = client.head(&main_url).send().await {
            if resp.status().is_success() {
                return main_url;
            }
        }

        // Default/Fallback to the 'master' branch
        return format!("https://raw.githubusercontent.com/{}/{}/master/.onix/install.onix", user, repo);
    }

    input.to_string()
}

/// Fetches an Onix manifest from a remote URL and parses it.
pub async fn fetch_manifest(url: &str) -> Result<OnixManifest> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("onix-cli/1.0.0")
        .build()?;

    let response = client
        .get(url)
        .send()
        .await
        .with_context(|| format!("Failed to connect to {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Server returned error: {} for URL: {}",
            response.status(),
            url
        ));
    }

    let yaml_text = response
        .text()
        .await
        .context("Failed to read manifest response body")?;

    let manifest: OnixManifest = serde_yaml::from_str(&yaml_text)
        .context("Failed to parse the fetched manifest as YAML")?;

    Ok(manifest)
}

/// Downloads a binary artifact and verifies its SHA256 checksum.
/// Returns the bytes of the artifact if verification succeeds.
pub async fn download_artifact(url: &str, expected_sha256: &str) -> Result<Vec<u8>> {
    let response = reqwest::get(url)
        .await
        .with_context(|| format!("Failed to download artifact from {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download artifact: HTTP {}",
            response.status()
        ));
    }

    // Get bytes from response
    let bytes = response
        .bytes()
        .await
        .context("Failed to read artifact bytes")?;

    // Calculate SHA256
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let actual_sha256 = hex::encode(hasher.finalize());

    if actual_sha256 != expected_sha256 {
        return Err(anyhow::anyhow!(
            "Checksum verification failed!\nExpected: {}\nActual:   {}",
            expected_sha256,
            actual_sha256
        ));
    }

    Ok(bytes.to_vec())
}