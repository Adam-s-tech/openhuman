use super::*;

#[test]
fn provider_family_exposes_status_and_the_engine_selector() {
    assert_eq!(
        FUNCTIONS,
        &[
            "provider_status",
            "engines_list",
            "engine_get",
            "engine_set",
            "engine_migrate",
            "engine_migrate_status",
            "engine_migrate_cancel"
        ]
    );
    assert_eq!(controllers().len(), FUNCTIONS.len());
    for function in FUNCTIONS {
        assert!(schema(function).is_some(), "{function} has no schema");
    }
}

#[test]
fn provider_status_schema_has_no_inputs_and_names_the_status_fields() {
    let schema = schema("provider_status").unwrap();
    assert_eq!(schema.namespace, "memory");
    assert!(schema.inputs.is_empty());
    let names: Vec<&str> = schema.outputs.iter().map(|f| f.name).collect();
    for expected in [
        "slot",
        "driver",
        "class",
        "health",
        "contract_version",
        "capabilities",
    ] {
        assert!(names.contains(&expected), "missing output {expected}");
    }
}

#[tokio::test]
async fn handler_returns_driver_and_capability_fields() {
    let value = handle_provider_status(Map::new())
        .await
        .expect("handler succeeds");
    assert!(value["driver"].is_string());
    assert!(value["capabilities"].is_array());
    assert!(value["contract_version"].is_string());
}
