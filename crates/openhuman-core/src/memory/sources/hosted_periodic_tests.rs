//! The hosted-memory schedule for local sources: its cadence, and which
//! sources a tick finds due.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, TimeZone, Utc};

use super::*;
use crate::memory::sources::run_history::HostRun;

const DAY: u64 = 86_400;

fn now() -> DateTime<Utc> {
    Utc.timestamp_opt(1_789_000_000, 0)
        .single()
        .expect("a valid instant")
}

fn source(id: &str, kind: &str, enabled: bool) -> MemorySourceEntry {
    serde_json::from_value(serde_json::json!({
        "id": id,
        "kind": kind,
        "label": id,
        "enabled": enabled,
        "path": ".",
        "url": "https://example.test/feed.xml",
        "toolkit": "gmail",
        "connection_id": "conn-1",
    }))
    .expect("a valid source entry")
}

fn row(source_id: &str, kind: &str, hours_ago: i64, failed: bool) -> SyncAuditEntry {
    HostRun {
        source_id: source_id.to_string(),
        source_kind: kind.to_string(),
        scope: source_id.to_string(),
        error: failed.then(|| "offline".to_string()),
        ..HostRun::default()
    }
    .into_entry(now() - chrono::Duration::hours(hours_ago))
}

fn ids(due: &[&MemorySourceEntry]) -> Vec<String> {
    due.iter().map(|source| source.id.clone()).collect()
}

#[test]
fn the_cadence_is_a_day_at_least_and_zero_is_manual_only() {
    assert_eq!(effective_interval_secs(None), Some(DAY));
    assert_eq!(effective_interval_secs(Some(0)), None);
    assert_eq!(effective_interval_secs(Some(3_600)), Some(DAY));
    assert_eq!(effective_interval_secs(Some(2 * DAY)), Some(2 * DAY));
}

#[test]
fn only_enabled_local_sources_are_scheduled() {
    let sources = vec![
        source("notes", "folder", true),
        source("repo", "github_repo", true),
        source("feed", "rss_feed", true),
        source("page", "web_page", true),
        source("off", "folder", false),
        source("mail", "composio", true),
        source("chats", "conversation", true),
        source("search", "twitter_query", true),
    ];
    let due = due_sources(&sources, DAY, &HashMap::new(), Some(&HashMap::new()), now());
    assert_eq!(ids(&due), ["notes", "repo", "feed", "page"]);
}

/// The run log times a source across restarts: a recent success waits, an old
/// one or none at all is due, and a failure does not count as a run.
#[test]
fn the_run_log_decides_what_is_due_after_a_restart() {
    let sources = vec![
        source("fresh", "folder", true),
        source("stale", "folder", true),
        source("failing", "folder", true),
        source("never", "folder", true),
    ];
    let history = last_success_by_source(&[
        row("fresh", "folder", 2, false),
        row("stale", "folder", 30, false),
        row("stale", "folder", 26, false),
        row("failing", "folder", 1, true),
        row("mail", "composio", 1, false),
    ]);
    assert!(!history.contains_key("failing"), "a failure is not a run");
    assert!(
        !history.contains_key("mail"),
        "connectors are not timed here"
    );
    assert_eq!(
        history["stale"],
        now() - chrono::Duration::hours(26),
        "the newest success wins"
    );
    let due = due_sources(&sources, DAY, &HashMap::new(), Some(&history), now());
    assert_eq!(ids(&due), ["stale", "failing", "never"]);
}

/// This process's own runs time a source before the log does, and without a
/// readable log only those can be timed at all.
#[test]
fn this_process_times_what_it_ran() {
    let sources = vec![
        source("ran", "folder", true),
        source("unknown", "folder", true),
    ];
    let just_now = HashMap::from([("ran".to_string(), Duration::from_secs(60))]);
    let long_ago = HashMap::from([("ran".to_string(), now() - chrono::Duration::days(9))]);
    assert!(
        due_sources(&sources, DAY, &just_now, Some(&long_ago), now())
            .iter()
            .all(|source| source.id != "ran"),
        "a run in this process is newer than the log says"
    );
    let due = due_sources(&sources, DAY, &just_now, None, now());
    assert!(due.is_empty(), "without the log, an untimed source waits");

    let yesterday = HashMap::from([("ran".to_string(), Duration::from_secs(DAY + 60))]);
    assert_eq!(
        ids(&due_sources(&sources, DAY, &yesterday, None, now())),
        ["ran"]
    );
}

/// A log stamped in the future, from a clock that moved, reads as just run.
#[test]
fn a_run_stamped_in_the_future_is_not_due() {
    let sources = vec![source("skewed", "folder", true)];
    let history = HashMap::from([("skewed".to_string(), now() + chrono::Duration::hours(3))]);
    assert!(due_sources(&sources, DAY, &HashMap::new(), Some(&history), now()).is_empty());
}
