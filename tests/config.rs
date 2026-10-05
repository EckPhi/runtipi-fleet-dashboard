use runtipi_fleet_dashboard::config::FleetConfig;

#[tokio::test]
async fn duplicate_server_names_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("fleet.yaml");
    tokio::fs::write(
        &path,
        r#"
servers:
  - id: "11111111-1111-4111-8111-111111111111"
    name: "Same"
    url: "http://north.example.test/"
    token_file: "/run/secrets/north.token"
  - id: "22222222-2222-4222-8222-222222222222"
    name: "Same"
    url: "http://south.example.test/"
    token_file: "/run/secrets/south.token"
"#,
    )
    .await
    .unwrap();

    let error = FleetConfig::load(&path).await.unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate configured server name"));
}
