use crate::{
    cache::{write_if_changed, CacheStore},
    client::CompanionClient,
    config::FleetConfig,
    model::{CachedInventory, FleetSnapshot, ServerSnapshot},
    render::homepage_services,
};
use anyhow::Result;
use chrono::Utc;
use std::{
    collections::{BTreeMap, HashMap},
    path::Path,
};
use tracing::{info, warn};

pub async fn run_once(config: &FleetConfig, cache: &CacheStore, output: &Path) -> Result<()> {
    let existing: HashMap<String, CachedInventory> = cache
        .load_all()
        .await?
        .into_iter()
        .map(|item| (item.inventory.server.id.clone(), item))
        .collect();
    let client = CompanionClient::new(config.request_timeout())?;
    let results = futures::future::join_all(config.servers.iter().map(|server| {
        let client = client.clone();
        async move { (server, client.inventory(server).await) }
    }))
    .await;
    let mut snapshot = FleetSnapshot {
        servers: BTreeMap::new(),
    };

    for (server, result) in results {
        match result {
            Ok(inventory) => {
                let id = inventory.server.id.clone();
                let saved = CachedInventory {
                    refreshed_at: Utc::now(),
                    inventory,
                };
                cache.save(&saved).await?;
                snapshot.servers.insert(
                    id,
                    ServerSnapshot {
                        configured_name: server.name.clone(),
                        cache: Some(saved),
                        error: None,
                        url_overrides: server.url_overrides.clone(),
                    },
                );
            }
            Err(error) => {
                warn!(server = %server.name, error = %error, "inventory request failed");
                let prior = existing.get(&server.id).cloned();
                snapshot.servers.insert(
                    server.id.clone(),
                    ServerSnapshot {
                        configured_name: server.name.clone(),
                        cache: prior,
                        error: Some(error.to_string()),
                        url_overrides: server.url_overrides.clone(),
                    },
                );
            }
        }
    }
    let rendered = homepage_services(&snapshot)?;
    if write_if_changed(output, &rendered).await? {
        info!(path = %output.display(), "updated Homepage services");
    }
    Ok(())
}
