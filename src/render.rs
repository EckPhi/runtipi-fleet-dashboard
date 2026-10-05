use crate::model::{AppStatus, FleetSnapshot};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct ServiceEntry(BTreeMap<String, Service>);

#[derive(Serialize)]
struct Service {
    href: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>,
    #[serde(skip_serializing_if = "is_false")]
    ping: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub fn homepage_services(snapshot: &FleetSnapshot) -> anyhow::Result<Vec<u8>> {
    let mut groups: Vec<BTreeMap<String, Vec<ServiceEntry>>> = Vec::new();
    for server in snapshot.servers.values() {
        let mut entries = Vec::new();
        let group_name = if let Some(cache) = &server.cache {
            let stale = server.error.is_some();
            for app in &cache.inventory.apps {
                let href = server
                    .url_overrides
                    .get(&app.urn)
                    .cloned()
                    .unwrap_or_else(|| app.browser_url.clone());
                let mut description = format!("{} · {}", app.status, app.urn);
                if stale {
                    description.push_str(&format!(" · cached {}", cache.refreshed_at.to_rfc3339()));
                }
                let mut named = BTreeMap::new();
                named.insert(
                    app.name.clone(),
                    Service {
                        href,
                        description,
                        icon: app.icon.clone(),
                        ping: app.status == AppStatus::Running,
                    },
                );
                entries.push(ServiceEntry(named));
            }
            if stale {
                format!("{} (offline)", server.configured_name)
            } else {
                server.configured_name.clone()
            }
        } else {
            format!("{} (offline)", server.configured_name)
        };
        if entries.is_empty() {
            let mut named = BTreeMap::new();
            named.insert(
                "Inventory unavailable".into(),
                Service {
                    href: "#".into(),
                    description: "No successful inventory or no installed apps".into(),
                    icon: None,
                    ping: false,
                },
            );
            entries.push(ServiceEntry(named));
        }
        let mut group = BTreeMap::new();
        group.insert(group_name, entries);
        groups.push(group);
    }
    Ok(serde_yaml::to_string(&groups)?.into_bytes())
}
