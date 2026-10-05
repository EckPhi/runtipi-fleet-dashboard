use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};
use url::Url;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FleetConfig {
    #[serde(default = "default_poll_seconds")]
    pub poll_interval_seconds: u64,
    #[serde(default = "default_timeout_seconds")]
    pub request_timeout_seconds: u64,
    pub servers: Vec<ServerConfig>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub id: String,
    pub name: String,
    pub url: Url,
    pub token_file: PathBuf,
    #[serde(default)]
    pub url_overrides: BTreeMap<String, String>,
}

fn default_poll_seconds() -> u64 {
    60
}
fn default_timeout_seconds() -> u64 {
    10
}

impl FleetConfig {
    pub async fn load(path: &Path) -> Result<Self> {
        let bytes = tokio::fs::read(path)
            .await
            .with_context(|| format!("read fleet config {}", path.display()))?;
        let config: Self = serde_yaml::from_slice(&bytes)
            .with_context(|| format!("parse fleet config {}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        anyhow::ensure!(!self.servers.is_empty(), "at least one server is required");
        anyhow::ensure!(
            self.poll_interval_seconds > 0,
            "poll interval must be positive"
        );
        anyhow::ensure!(
            self.request_timeout_seconds > 0,
            "request timeout must be positive"
        );
        let mut names = std::collections::HashSet::new();
        for server in &self.servers {
            anyhow::ensure!(!server.id.trim().is_empty(), "server id must not be empty");
            uuid::Uuid::parse_str(&server.id)
                .with_context(|| format!("server id must be a UUID: {}", server.id))?;
            anyhow::ensure!(
                !server.name.trim().is_empty(),
                "server name must not be empty"
            );
            anyhow::ensure!(
                names.insert(&server.id),
                "duplicate configured server id: {}",
                server.id
            );
            anyhow::ensure!(
                matches!(server.url.scheme(), "http" | "https"),
                "server URL must be http(s)"
            );
            for url in server.url_overrides.values() {
                Url::parse(url).with_context(|| format!("invalid browser URL override {url:?}"))?;
            }
        }
        Ok(())
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_interval_seconds)
    }
    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_seconds)
    }
}
