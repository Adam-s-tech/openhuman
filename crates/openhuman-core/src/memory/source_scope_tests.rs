use super::*;

#[tokio::test]
async fn unrestricted_outside_scope() {
    assert!(current_source_scope().is_none());
    assert!(scope_allowed("anything"));
}

#[tokio::test]
async fn restricts_to_allowlisted_scopes() {
    with_source_scope(
        Some(vec!["slack:#eng".into(), "  gmail:me  ".into()]),
        async {
            let set = current_source_scope().expect("scope set");
            assert_eq!(set.len(), 2);
            assert!(scope_allowed("slack:#eng"));
            assert!(scope_allowed("gmail:me")); // trimmed
            assert!(!scope_allowed("notion:team"));
        },
    )
    .await;
    // Must not leak past the scope.
    assert!(current_source_scope().is_none());
    assert!(scope_allowed("notion:team"));
}

#[tokio::test]
async fn empty_allowlist_blocks_everything() {
    with_source_scope(Some(vec![]), async {
        assert!(current_source_scope().is_some());
        assert!(!scope_allowed("slack:#eng"));
    })
    .await;
}

#[tokio::test]
async fn explicit_none_is_unrestricted() {
    with_source_scope(None, async {
        assert!(current_source_scope().is_none());
        assert!(scope_allowed("slack:#eng"));
    })
    .await;
}

#[tokio::test]
async fn chunk_gate_passes_non_source_chunks_and_gates_tagged_ones() {
    let src_tags = vec!["memory_sources".to_string(), "document".to_string()];
    let other_tags = vec!["conversation".to_string()];

    with_source_scope(
        Some(vec!["slack:#eng".into(), "src-rss-42".into()]),
        async {
            // Non-source chunk (no memory_sources tag) always passes.
            assert!(chunk_source_allowed(&other_tags, "thr_123:user"));
            // Composio/channel source chunk: raw source_id == scope.
            assert!(chunk_source_allowed(&src_tags, "slack:#eng"));
            assert!(!chunk_source_allowed(&src_tags, "gmail:alice"));
            // Reader-based composite: extracted registry id matches.
            assert!(chunk_source_allowed(
                &src_tags,
                "mem_src:src-rss-42:https://example.com/item-7"
            ));
            assert!(!chunk_source_allowed(
                &src_tags,
                "mem_src:src-folder-9:/notes/a.md"
            ));
        },
    )
    .await;
}

#[tokio::test]
async fn chunk_gate_unrestricted_without_scope() {
    let src_tags = vec!["memory_sources".to_string()];
    // Outside any scope, even tagged source chunks pass.
    assert!(chunk_source_allowed(&src_tags, "gmail:alice"));
}

#[tokio::test]
async fn chunk_gate_empty_allowlist_blocks_tagged_sources_only() {
    let src_tags = vec!["memory_sources".to_string()];
    let other_tags: Vec<String> = vec![];
    with_source_scope(Some(vec![]), async {
        assert!(!chunk_source_allowed(&src_tags, "slack:#eng"));
        // Non-source chunks still pass even under an empty allowlist.
        assert!(chunk_source_allowed(&other_tags, "thr_1:user"));
    })
    .await;
}

// `as_bus_scope` renders the task-local scope for the memory module's bus
// argument. `None` means unrestricted; `Some(empty)` denies every
// source-attributed item. Collapsing one onto the other inverts the policy.

#[tokio::test]
async fn unrestricted_recall_renders_as_no_scope_not_an_empty_allowlist() {
    assert!(as_bus_scope().is_none());

    with_source_scope(None, async {
        assert!(as_bus_scope().is_none());
    })
    .await;
}

#[tokio::test]
async fn an_empty_allowlist_renders_as_a_scope_that_denies_every_source() {
    with_source_scope(Some(vec![]), async {
        let scope = as_bus_scope().expect("an empty allowlist is a restriction");
        assert!(scope.is_empty());
        assert!(!scope.allows_source_id("mem_src:anything:item-1"));
    })
    .await;
}

#[tokio::test]
async fn the_allowlist_crosses_the_bus_and_matches_by_the_drivers_rule() {
    let allowed = vec!["gmail:work".to_string(), "src-abc".to_string()];

    with_source_scope(Some(allowed.clone()), async {
        let scope = as_bus_scope().expect("a non-empty allowlist must render as Some");

        let carried: std::collections::HashSet<&str> =
            scope.allow.iter().map(String::as_str).collect();
        assert_eq!(
            carried,
            allowed
                .iter()
                .map(String::as_str)
                .collect::<std::collections::HashSet<_>>()
        );

        assert!(scope.allows_source_id("src-abc"));
        assert!(scope.allows_source_id("mem_src:src-abc:item-1"));
        assert!(!scope.allows_source_id("src-xyz"));
    })
    .await;
}
