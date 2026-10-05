use runtipi_fleet_dashboard::model::Inventory;

#[test]
fn example_contract_is_strict_and_valid() {
    let value: Inventory =
        serde_json::from_str(include_str!("../docs/contracts/inventory-v1.example.json")).unwrap();
    value.validate().unwrap();
}

#[test]
fn duplicate_urns_are_rejected() {
    let source = include_str!("../docs/contracts/inventory-v1.example.json");
    let mut value: serde_json::Value = serde_json::from_str(source).unwrap();
    let app = value["apps"][0].clone();
    value["apps"].as_array_mut().unwrap().push(app);
    let inventory: Inventory = serde_json::from_value(value).unwrap();
    assert!(inventory
        .validate()
        .unwrap_err()
        .contains("duplicate app URN"));
}
