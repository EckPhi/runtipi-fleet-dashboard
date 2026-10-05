use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use runtipi_fleet_dashboard::{
    cache::CacheStore,
    config::{FleetConfig, ServerConfig},
    model::{AppRecord, AppStatus, Inventory, ServerIdentity},
    runner::run_once,
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

const SERVER_ID: &str = "11111111-1111-4111-8111-111111111111";

async fn inventory(
    State(mode): State<Arc<AtomicU8>>,
    headers: HeaderMap,
) -> Result<Json<Inventory>, StatusCode> {
    if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some("Bearer test-token") {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if mode.load(Ordering::SeqCst) == 1 {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let apps = if mode.load(Ordering::SeqCst) == 2 {
        Vec::new()
    } else {
        vec![AppRecord {
            urn: "notes:official".into(),
            name: "Notes".into(),
            status: AppStatus::Running,
            browser_url: "https://notes.example.test".into(),
            icon: None,
        }]
    };
    Ok(Json(Inventory {
        protocol_version: "1".into(),
        server: ServerIdentity {
            id: SERVER_ID.into(),
            name: "Synthetic".into(),
        },
        collected_at: chrono::Utc::now(),
        apps,
    }))
}

#[tokio::test]
async fn retains_offline_cache_then_accepts_confirmed_removal() {
    let mode = Arc::new(AtomicU8::new(0));
    let app = Router::new()
        .route("/v1/inventory", get(inventory))
        .with_state(mode.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let temp = tempfile::tempdir().unwrap();
    let token = temp.path().join("token");
    tokio::fs::write(&token, "test-token\n").await.unwrap();
    let output = temp.path().join("homepage/services.yaml");
    let config = FleetConfig {
        poll_interval_seconds: 60,
        request_timeout_seconds: 2,
        servers: vec![ServerConfig {
            id: SERVER_ID.into(),
            name: "Synthetic".into(),
            url: format!("http://{address}/").parse().unwrap(),
            token_file: token,
            url_overrides: BTreeMap::new(),
        }],
    };
    let cache = CacheStore::new(temp.path().join("state"));

    run_once(&config, &cache, &output).await.unwrap();
    assert!(tokio::fs::read_to_string(&output)
        .await
        .unwrap()
        .contains("Notes"));

    mode.store(1, Ordering::SeqCst);
    run_once(&config, &cache, &output).await.unwrap();
    let stale = tokio::fs::read_to_string(&output).await.unwrap();
    assert!(stale.contains("Notes"));
    assert!(stale.contains("Synthetic (offline)"));

    mode.store(2, Ordering::SeqCst);
    run_once(&config, &cache, &output).await.unwrap();
    let recovered = tokio::fs::read_to_string(&output).await.unwrap();
    assert!(!recovered.contains("Notes"));
    assert!(!recovered.contains("Synthetic (offline)"));
}
