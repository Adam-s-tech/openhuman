//! Tests for the generic driver vocabulary.
//!
//! Note: this is the **only** file under `crates/openhuman-core/src/core/subsystem/` that may name
//! `tinycortex_api`, and it does so purely as a drift witness — see
//! `driver_health_shape_matches_memory_health_one_for_one` and
//! `every_memory_contract_capability_string_maps_into_driver_capabilities`.
//! The production modules mention it only in prose; they must never *depend*
//! on it, which is checkable with:
//!
//! ```text
//! grep -rn '^ *use .*tinycortex_api' crates/openhuman-core/src/core/subsystem/driver.rs \
//!     crates/openhuman-core/src/core/subsystem/registry.rs crates/openhuman-core/src/core/subsystem/mod.rs   # no output
//! ```

use std::str::FromStr;

use serde_json::json;

use super::*;

#[test]
fn driver_class_as_str_matches_serde_representation() {
    for class in DriverClass::ALL {
        let encoded = serde_json::to_value(class).expect("class serializes");
        assert_eq!(encoded, json!(class.as_str()), "mismatch for {class:?}");
    }
}

#[test]
fn driver_class_parse_round_trips_every_variant() {
    for class in DriverClass::ALL {
        assert_eq!(DriverClass::parse(class.as_str()), Ok(class));
        assert_eq!(DriverClass::from_str(class.as_str()), Ok(class));
        assert_eq!(class.to_string(), class.as_str());
    }
}

#[test]
fn driver_class_parse_rejects_unknown_with_the_input_in_the_message() {
    let err = DriverClass::parse("sidecar").expect_err("unknown class rejected");
    assert!(
        err.contains("sidecar"),
        "message should name the input: {err}"
    );
}

#[test]
fn driver_health_is_usable_is_false_only_when_down() {
    assert!(DriverHealth::Ready.is_usable());
    assert!(DriverHealth::degraded("index rebuilding").is_usable());
    assert!(!DriverHealth::down("connection refused").is_usable());
}

#[test]
fn driver_health_serializes_with_a_stable_status_discriminant() {
    assert_eq!(
        serde_json::to_value(DriverHealth::Ready).expect("ready serializes"),
        json!({ "status": "ready" })
    );
    assert_eq!(
        serde_json::to_value(DriverHealth::degraded("slow")).expect("degraded serializes"),
        json!({ "status": "degraded", "reason": "slow" })
    );
    assert_eq!(
        serde_json::to_value(DriverHealth::down("refused")).expect("down serializes"),
        json!({ "status": "down", "reason": "refused" })
    );

    let decoded: DriverHealth =
        serde_json::from_value(json!({ "status": "degraded", "reason": "slow" }))
            .expect("degraded deserializes");
    assert_eq!(decoded, DriverHealth::degraded("slow"));
}

#[test]
fn driver_health_display_includes_the_reason() {
    assert_eq!(DriverHealth::Ready.to_string(), "ready");
    assert_eq!(DriverHealth::Ready.reason(), None);
    assert_eq!(
        DriverHealth::down("connection refused").to_string(),
        "down: connection refused"
    );
    assert_eq!(
        DriverHealth::degraded("slow").reason(),
        Some("slow"),
        "reason is readable for status output"
    );
}

#[test]
fn driver_capabilities_round_trips_through_json() {
    let caps = DriverCapabilities::empty()
        .with("core")
        .with("recall")
        .with("portability");

    let encoded = serde_json::to_value(&caps).expect("capabilities serialize");
    let decoded: DriverCapabilities =
        serde_json::from_value(encoded.clone()).expect("capabilities deserialize");

    assert_eq!(decoded, caps);
    assert_eq!(encoded, json!(["core", "portability", "recall"]));
}

#[test]
fn driver_capabilities_collapses_duplicates() {
    let caps: DriverCapabilities = ["core", "recall", "core"].into_iter().collect();
    assert_eq!(caps.len(), 2);
    assert!(caps.contains("core"));
    assert!(caps.contains("recall"));
    assert!(!caps.contains("tree"));

    let mut caps = caps;
    caps.remove("recall");
    assert_eq!(caps.len(), 1);
    caps.remove("recall");
    assert_eq!(caps.len(), 1, "remove is idempotent");
}

#[test]
fn driver_capabilities_serializes_as_an_array_of_strings() {
    assert_eq!(
        serde_json::to_value(DriverCapabilities::empty()).expect("empty serializes"),
        json!([])
    );
    assert!(DriverCapabilities::empty().is_empty());

    let mut caps = DriverCapabilities::empty();
    caps.extend(["tree", "core"]);
    assert_eq!(
        serde_json::to_value(&caps).expect("serializes"),
        json!(["core", "tree"]),
        "a set has no order; the backing BTreeSet emits lexicographic order"
    );
}

#[test]
fn driver_capabilities_contains_all_is_subset_semantics() {
    let advertised: DriverCapabilities = ["core", "recall", "portability", "tree"]
        .into_iter()
        .collect();
    let mandatory: DriverCapabilities = ["core", "recall", "portability"].into_iter().collect();

    assert!(advertised.contains_all(&mandatory));
    assert!(!mandatory.contains_all(&advertised));
    assert!(advertised.contains_all(&DriverCapabilities::empty()));
}

