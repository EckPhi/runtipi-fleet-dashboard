use chrono::{TimeZone, Utc};
use runtipi_fleet_dashboard::{
    model::{
        AppRecord, AppStatus, CachedInventory, FleetSnapshot, Inventory, ServerIdentity,
        ServerSnapshot,
    },
    render::homepage_services,
};
use std::collections::BTreeMap;

fn inventory(id: &str, app_name: &str) -> CachedInventory {
    CachedInventory {
        refreshed_at: Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap(),
        inventory: Inventory {
            protocol_version: "1".into(),
            server: ServerIdentity {
                id: id.into(),
                name: id.into(),
            },
            collected_at: Utc.with_ymd_and_hms(2026, 1, 1, 11, 59, 0).unwrap(),
            apps: vec![AppRecord {
                urn: "app:store".into(),
                name: app_name.into(),
                status: AppStatus::Stopped,
                browser_url: "https://app.example.test".into(),
                icon: None,
            }],
        },
    }
}

#[test]
fn duplicate_apps_on_different_servers_remain_separate() {
    let mut snapshot = FleetSnapshot::default();
    for (id, name) in [("server-a", "North"), ("server-b", "South")] {
        snapshot.servers.insert(
            id.into(),
            ServerSnapshot {
                configured_name: name.into(),
                cache: Some(inventory(id, "Same App")),
                error: None,
                url_overrides: BTreeMap::new(),
            },
        );
    }
    let output = String::from_utf8(homepage_services(&snapshot).unwrap()).unwrap();
    assert_eq!(output.matches("Same App").count(), 2);
    assert!(output.contains("North"));
    assert!(output.contains("South"));
    assert!(output.contains("stopped"));
}

#[test]
fn offline_server_retains_cached_tile_and_override() {
    let mut overrides = BTreeMap::new();
    overrides.insert("app:store".into(), "https://override.example.test".into());
    let mut snapshot = FleetSnapshot::default();
    snapshot.servers.insert(
        "server-a".into(),
        ServerSnapshot {
            configured_name: "North".into(),
            cache: Some(inventory("server-a", "App")),
            error: Some("timeout".into()),
            url_overrides: overrides,
        },
    );
    let output = String::from_utf8(homepage_services(&snapshot).unwrap()).unwrap();
    assert!(output.contains("North (offline)"));
    assert!(output.contains("cached 2026-01-01T12:00:00+00:00"));
    assert!(output.contains("https://override.example.test"));
}
