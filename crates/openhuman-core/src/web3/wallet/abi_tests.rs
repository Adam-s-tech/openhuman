use super::*;

#[test]
fn an_invalid_recipient_still_names_the_address() {
    let error = encode_erc20_transfer("not-an-address", "5").unwrap_err();
    assert!(error.contains("not-an-address"), "{error}");
}

#[test]
fn an_invalid_amount_still_reads_the_way_the_tool_schema_says() {
    let error =
        encode_erc20_transfer("0x1111111111111111111111111111111111111111", "-1").unwrap_err();
    assert!(
        error.contains("not a valid non-negative integer"),
        "{error}"
    );
}
