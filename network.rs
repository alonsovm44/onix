use crate::models::OnixManifest;
use anyhow::{Context, Result};

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