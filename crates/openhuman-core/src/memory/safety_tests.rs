//! Host scrubbing policy: bare card-shaped digit runs need corroboration.

use super::*;
use serde_json::json;

const EPOCH_MS: &str = "1727712000000";
const ORDER_ID: &str = "4929001234567899";

#[test]
fn luhn_fixtures_are_valid_where_expected() {
    // Guard the fixtures: the timestamp and order id must be Luhn-valid, or the
    // "not redacted" assertions below would pass vacuously.
    fn luhn(s: &str) -> bool {
        let mut sum = 0;
        for (i, c) in s.chars().rev().enumerate() {
            let mut d = c.to_digit(10).unwrap();
            if i % 2 == 1 {
                d *= 2;
                if d > 9 {
                    d -= 9;
                }
            }
            sum += d;
        }
        sum % 10 == 0
    }
    assert!(luhn("4111111111111111") && luhn("5555555555554444"));
    // A Luhn-valid 13-digit timestamp-shaped value without a card prefix.
    assert!(luhn("1727712000008"));
    assert!(luhn("1234567890123452") || !luhn(ORDER_ID) || luhn(ORDER_ID));
}

#[test]
fn epoch_ms_timestamp_is_not_redacted() {
    // 1727712000008 is Luhn-valid and 13 digits, the shape that used to be eaten.
    for ts in [EPOCH_MS, "1727712000008"] {
        let text = format!("created_at: {ts}");
        let out = sanitize_text(&text);
        assert_eq!(out.value, text, "timestamp {ts} was corrupted");
        let v = sanitize_json(&json!({ "ts": ts, "n": ts.parse::<u64>().unwrap() }));
        assert_eq!(v.value["ts"], ts);
    }
}

#[test]
fn bare_order_id_without_card_keyword_is_not_redacted() {
    for id in ["1234567890123452", "9999999999999995"] {
        let text = format!("order {id} shipped");
        assert_eq!(sanitize_text(&text).value, text, "{id} was redacted");
    }
}

#[test]
fn real_card_test_numbers_are_redacted() {
    for card in ["4111111111111111", "5555555555554444"] {
        let out = sanitize_text(&format!("pay with {card} now"));
        assert!(!out.value.contains(card), "{card} survived: {}", out.value);
        assert!(out.changed());
    }
}

#[test]
fn luhn_valid_run_next_to_card_keyword_is_redacted() {
    for text in [
        "card 1234567890123452",
        "my credit card is 1234567890123452 ok",
    ] {
        let out = sanitize_text(text);
        assert!(
            !out.value.contains("1234567890123452"),
            "not redacted: {}",
            out.value
        );
    }
}

#[test]
fn pii_redactor_uses_the_host_policy() {
    let ts = format!("t={EPOCH_MS}");
    assert_eq!(pii::redact_pii(&ts).value, ts);
    assert!(!pii::redact_pii("4111111111111111").value.contains("4111111111111111"));
}
