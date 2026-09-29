use serde_json::Value;

const CASES: &[&str] = &[
    r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"Claude Desktop","version":"0"}}}"#,
    r#"{"jsonrpc":"2.0","id":"init","method":"initialize","params":{"protocolVersion":"1999-01-01"}}"#,
    r#"{"jsonrpc":"2.0","id":7,"method":"initialize"}"#,
    r#"{"jsonrpc":"2.0","id":"abc","method":"ping"}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}"#,
    r#"{"jsonrpc":"2.0","method":"notifications/other"}"#,
    r#"[]"#,
    r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"},42,{"jsonrpc":"2.0","id":3,"method":"nope"}]"#,
    r#"[{"jsonrpc":"2.0","method":"notifications/initialized"}]"#,
    r#"42"#,
    r#"{"jsonrpc":"2.0","id":1.5,"method":"ping"}"#,
    r#"{"jsonrpc":"2.0","id":true,"method":"ping"}"#,
    r#"{"jsonrpc":"2.0","id":null,"method":"ping"}"#,
    r#"{"id":1,"method":"ping"}"#,
    r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#,
    r#"{"jsonrpc":"2.0","id":1}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":5}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list"}"#,
    r#"{not-json"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call"}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":[]}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"   "}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search","arguments":[1,2]}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search","arguments":"not json"}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search","arguments":7}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search","arguments":{}}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search","arguments":"{}"}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search"}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory.search","arguments":{"query":"x","bogus":1}}}"#,
    r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"no.such_tool","arguments":{}}}"#,
    r#"{"jsonrpc":"2.0","id":12,"method":"resources/read","params":{"uri":"openhuman://prompts/agents/does_not_exist"}}"#,
    r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{}}"#,
    r#"{"jsonrpc":"2.0","id":13,"method":"resources/read","params":{"uri":"  "}}"#,
    r#"{"jsonrpc":"2.0","id":13,"method":"resources/read"}"#,
    r#"{"jsonrpc":"2.0","id":14,"method":"resources/templates/list"}"#,
    r#"{"jsonrpc":"2.0","id":15,"method":"resources/templates/list","params":{"cursor":"x"}}"#,
];

#[tokio::test]
async fn capture() {
    let mut out = String::new();
    for case in CASES {
        let got = super::protocol::handle_json_line(case).await;
        let got = got.map(|s| s.replace(env!("CARGO_PKG_VERSION"), "{{CARGO_PKG_VERSION}}"));
        out.push_str(&format!("    (\n        r#\"{case}\"#,\n        {},\n    ),\n", match got { Some(s) => format!("Some(r#\"{s}\"#)"), None => "None".into() }));
    }
    let v: Value = serde_json::from_str(&super::protocol::handle_json_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#).await.unwrap()).unwrap();
    out.push_str(&format!("TOOLS_LIST {}\n", v));
    let v = super::protocol::handle_json_line(r#"{"jsonrpc":"2.0","id":10,"method":"resources/list"}"#).await.unwrap();
    out.push_str(&format!("RES_LIST {}\n", v));
    std::fs::write(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../wire_capture.txt"), out).unwrap();
}
