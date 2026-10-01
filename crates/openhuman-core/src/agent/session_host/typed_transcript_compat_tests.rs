//! Typed transcript rows against the #6872 compatibility corpus.
//!
//! The corpus in `tests/fixtures/session_compat/` was captured from the release
//! that wrote string-envelope rows. Two gates, both reusing its goldens:
//!
//! - **Same rows, new on-disk form.** `<name>.typed.jsonl` is the legacy
//!   capture re-written by the current writer (tool calls, tool results and
//!   images stored as fields). Resuming it must give byte-identical committed
//!   history, frozen prefix, recorded tool list, journal projection and next
//!   provider request as the legacy file's golden: the typed form is purely an
//!   on-disk change.
//! - **Old session, new binary.** A legacy file continued with a native tool
//!   round keeps its bytes, gains typed rows, and a fresh process resumes the
//!   mixed file to exactly what the continuing process held.
//!
//! Regenerate the `.typed` files only when deliberately re-deriving them:
//! `OH_REGEN_SESSION_COMPAT=1 cargo test -p openhuman --lib regenerate_typed_session_compat -- --ignored`.

use serde_json::{json, Value};
use tinyagents_session::transcript::{append_tools_record, read_transcript, write_transcript};
use tinyinference_llm::message::ContentBlock;
use tinyinference_llm::model::ModelResponse;

use super::transcript_compat_tests::{
    build_host, call, fixture, golden, model, response, run_async, snapshot_with, stem_path,
    thread_id, Scenario, SCENARIOS,
};

/// Head fixtures that have a typed re-write (the chain and legacy layouts are
/// multi-file and the compaction record is not a plain row list).
const TYPED_SCENARIOS: &[&str] = &["plain", "native_tools", "image_user", "xml_tools"];

fn scenario(name: &str) -> &'static Scenario {
    SCENARIOS
        .iter()
        .find(|scenario| scenario.name == name)
        .expect("scenario")
}

fn message_lines(raw: &str) -> Vec<Value> {
    raw.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("role").is_some())
        .collect()
}

#[test]
fn typed_rewrites_of_the_corpus_resume_to_the_legacy_goldens() {
    for name in TYPED_SCENARIOS {
        run_async(async move {
            let got = snapshot_with(scenario(name), ".typed").await;
            assert_eq!(
                got,
                golden(name),
                "{name}: typed rows must resume to the legacy golden"
            );
        });
    }
}

#[test]
fn typed_fixtures_really_use_the_typed_form() {
    let shapes = |name: &str| -> Vec<String> {
        let raw = std::fs::read_to_string(fixture(&format!("{name}.typed.jsonl"))).unwrap();
        message_lines(&raw)
            .iter()
            .filter_map(|line| line.get("shape").and_then(Value::as_str).map(String::from))
            .collect()
    };
    let native = shapes("native_tools");
    assert!(native.iter().any(|shape| shape == "assistant_calls"));
    assert!(native.iter().any(|shape| shape == "tool_result"));
    assert!(shapes("image_user")
        .iter()
        .any(|shape| shape == "user_parts"));
    assert!(shapes("plain").is_empty());
    // The envelope string no longer appears in a typed row's content.
    let raw = std::fs::read_to_string(fixture("native_tools.typed.jsonl")).unwrap();
    for line in message_lines(&raw) {
        if line.get("v").is_some() {
            assert!(
                !line["content"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with('{'),
                "{line}"
            );
        }
    }
}

#[test]
fn an_old_session_continued_by_this_binary_reloads_identically() {
    run_async(async {
        let scenario = scenario("native_tools");
        let root = tempfile::tempdir().expect("tempdir");
        let text = |s: &str| response(vec![ContentBlock::Text(s.into())], Vec::new());
        let thread = thread_id(scenario.name);

        let mut first = build_host(
            root.path(),
            model(
                vec![
                    response(
                        vec![ContentBlock::Text("one more call".into())],
                        vec![call("call_new", "echo", json!({"q": "later"}))],
                    ),
                    text("continued"),
                ],
                true,
            ),
            true,
            &thread,
        );
        let stem = first.session_id().expect("session id");
        let path = stem_path(root.path(), &stem);
        std::fs::copy(fixture("native_tools.jsonl"), &path).expect("place legacy file");
        let legacy_bytes = std::fs::read(&path).expect("legacy bytes");

        assert!(first.resume_bound_session().await.expect("resume"));
        first.turn("keep going").await.expect("continued turn");
        let continued_history = serde_json::to_value(
            first
                .runtime_session
                .as_ref()
                .expect("runtime session")
                .history(),
        )
        .expect("history");
        drop(first);

        // The old rows are byte-for-byte what was captured; new rows follow.
        let after = std::fs::read(&path).expect("file after");
        assert!(
            after.starts_with(&legacy_bytes),
            "existing bytes were rewritten"
        );
        let appended = String::from_utf8(after[legacy_bytes.len()..].to_vec()).unwrap();
        let new_rows = message_lines(&appended);
        let shape = |index: usize| new_rows[index].get("shape").and_then(Value::as_str);
        assert!(
            new_rows.iter().any(|row| row["shape"] == "assistant_calls")
                && new_rows.iter().any(|row| row["shape"] == "tool_result"),
            "continued tool round must be typed: {shape:?}",
            shape = (0..new_rows.len()).map(shape).collect::<Vec<_>>()
        );

        // A fresh process resumes the mixed file to what the first one held.
        let second = {
            let mut host = build_host(
                root.path(),
                model(vec![ModelResponse::assistant("unused")], true),
                true,
                &thread,
            );
            assert!(host.resume_bound_session().await.expect("second resume"));
            host
        };
        let runtime = second.runtime_session.as_ref().expect("runtime session");
        assert_eq!(
            serde_json::to_value(runtime.history()).expect("history"),
            continued_history
        );
        let transcript = read_transcript(&path).expect("read mixed");
        assert!(transcript
            .messages
            .iter()
            .any(|row| row.content.contains("call_new")));
    });
}

#[test]
#[ignore = "re-derives the committed .typed fixtures; run deliberately (OH_REGEN_SESSION_COMPAT=1)"]
fn regenerate_typed_session_compat() {
    if std::env::var("OH_REGEN_SESSION_COMPAT").as_deref() != Ok("1") {
        return;
    }
    for name in TYPED_SCENARIOS {
        let transcript = read_transcript(&fixture(&format!("{name}.jsonl"))).expect("read legacy");
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("typed.jsonl");
        write_transcript(&path, &transcript.messages, &transcript.meta, None).expect("write typed");
        // The rows' sibling record: the tool list the session was sent with.
        let legacy = std::fs::read_to_string(fixture(&format!("{name}.jsonl"))).expect("raw");
        let tools = legacy
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|value| value.get("kind").and_then(Value::as_str) == Some("tools"))
            .next_back()
            .map(|value| value["tools"].clone())
            .expect("legacy fixture records its tools");
        append_tools_record(&path, &tools).expect("tools record");
        std::fs::copy(&path, fixture(&format!("{name}.typed.jsonl"))).expect("store typed");
    }
}
