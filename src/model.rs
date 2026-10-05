use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const PROTOCOL_VERSION: &str = "1";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub protocol_version: String,
    pub server: ServerIdentity,
    pub collected_at: DateTime<Utc>,
    #[serde(default)]
    pub apps: Vec<AppRecord>,
}

impl Inventory {
    pub fn validate(&self) -> Result<(), String> {
        if self.protocol_version != PROTOCOL_VERSION {
            return Err(format!(
                "unsupported protocol version {:?}",
                self.protocol_version
            ));
        }
        if self.server.id.trim().is_empty() || self.server.name.trim().is_empty() {
            uuid::Uuid::parse_str(&self.server.id)
                .map_err(|error| format!("server id must be a UUID: {error}"))?;
            return Err("server id and name must not be empty".into());
        }
        let mut seen = std::collections::HashSet::new();
        for app in &self.apps {
            if app.urn.trim().is_empty() || app.name.trim().is_empty() {
                return Err("app urn and name must not be empty".into());
            }
            if !seen.insert(&app.urn) {
                return Err(format!("duplicate app URN {:?}", app.urn));
            }
            url::Url::parse(&app.browser_url)
                .map_err(|error| format!("invalid browser_url for {}: {error}", app.urn))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ServerIdentity {
    pub id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AppRecord {
    pub urn: String,
    pub name: String,
    pub status: AppStatus,
    pub browser_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppStatus {
    Running,
    Stopped,
    Installing,
    Updating,
    Error,
    Unknown,
}

impl std::fmt::Display for AppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Installing => "installing",
            Self::Updating => "updating",
            Self::Error => "error",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CachedInventory {
    pub refreshed_at: DateTime<Utc>,
    pub inventory: Inventory,
}

#[derive(Clone, Debug, Default)]
pub struct FleetSnapshot {
    pub servers: BTreeMap<String, ServerSnapshot>,
}

#[derive(Clone, Debug)]
pub struct ServerSnapshot {
    pub configured_name: String,
    pub cache: Option<CachedInventory>,
    pub error: Option<String>,
    pub url_overrides: BTreeMap<String, String>,
}
