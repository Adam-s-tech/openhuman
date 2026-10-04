use super::*;
use serde_json::json;

async fn setup() -> (tempfile::TempDir, crate::config::Config) {
    let temp = tempfile::tempdir().unwrap();
    let mut config = crate::config::Config::default();
    config.workspace_dir = temp.path().join("internal");
    config.action_dir = temp.path().join("acting");
    conversations::blocking::ensure_thread(
        config.workspace_dir.clone(),
        conversations::CreateConversationThread {
            id: "uploads".into(),
            title: "Uploads".into(),
            created_at: "2026-10-04T00:00:00Z".into(),
            parent_thread_id: None,
            labels: None,
            personality_id: None,
        },
    )
    .await
    .unwrap();
    (temp, config)
}

fn request(id: &str, content: &str, sender: &str) -> AppendConversationMessageRequest {
    AppendConversationMessageRequest {
        thread_id: "uploads".into(),
        message: ConversationMessageRecord {
            id: id.into(),
            content: content.into(),
            message_type: "text".into(),
            extra_metadata: json!({"attachmentNames":["image.png"], "attachmentKinds":["image"],
                "attachmentDataUris":["data:image/png;base64,iVBORw0KGgo="], "attachmentData":"bytes",
                "attachmentPosters":["data:image/png;base64,iVBORw0KGgo="],
                "attachmentPreview":"data:image/png;base64,iVBORw0KGgo=", "other":"kept"}),
            sender: sender.into(),
            created_at: "2026-10-04T00:00:00Z".into(),
        },
    }
}

#[tokio::test]
async fn user_append_stages_original_strips_upload_bytes_and_reuses_returned_content() {
    let (_temp, config) = setup().await;
    let first = message_append_with_config(
        request(
            "one",
            "look [IMAGE:data:image/png;base64,iVBORw0KGgo=]",
            "user",
        ),
        &config,
    )
    .await
    .unwrap()
    .value
    .data
    .unwrap();
    let (_, files) = crate::agent::attachments::parse(&first.content);
    assert_eq!(files.len(), 1);
    assert_eq!(
        tokio::fs::read(config.action_dir.join(&files[0].path))
            .await
            .unwrap(),
        b"\x89PNG\r\n\x1a\n"
    );
    assert!(first.extra_metadata.get("attachmentDataUris").is_none());
    assert!(first.extra_metadata.get("attachmentData").is_none());
    assert!(first.extra_metadata.get("attachmentPosters").is_none());
    assert!(first.extra_metadata.get("attachmentPreview").is_none());
    assert_eq!(
        first.extra_metadata["attachmentNames"],
        json!(["image.png"])
    );
    assert_eq!(first.extra_metadata["attachmentKinds"], json!(["image"]));
    assert_eq!(first.extra_metadata["other"], "kept");
    let second = message_append_with_config(request("two", &first.content, "user"), &config)
        .await
        .unwrap()
        .value
        .data
        .unwrap();
    assert_eq!(second.content, first.content);
    let staged = crate::agent::attachments::stage_turn(
        &second.content,
        Some(&config),
        None,
        Some("uploads"),
    )
    .await
    .unwrap();
    assert_eq!(staged, first.content);
    let persisted =
        conversations::blocking::get_messages(config.workspace_dir.clone(), "uploads".into())
            .await
            .unwrap();
    assert_eq!(persisted.len(), 2);
    assert!(!serde_json::to_string(&persisted)
        .unwrap()
        .contains("base64,"));
}

#[tokio::test]
async fn rejected_upload_never_appends_a_message() {
    let (_temp, mut config) = setup().await;
    config.multimodal_files.max_files = 0;
    assert!(message_append_with_config(
        request(
            "disabled",
            "[IMAGE:data:image/png;base64,iVBORw0KGgo=]",
            "user"
        ),
        &config
    )
    .await
    .is_err());
    config.multimodal_files.max_files = 3;
    assert!(message_append_with_config(
        request("bad", "[FILE:data:application/zip;base64,!]", "user"),
        &config
    )
    .await
    .is_err());
    assert!(
        message_append_with_config(request("missing", "image.png", "user"), &config)
            .await
            .is_err()
    );
    assert!(
        conversations::blocking::get_messages(config.workspace_dir.clone(), "uploads".into())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn ordinary_user_text_and_agent_images_are_preserved() {
    let (_temp, config) = setup().await;
    let mut user = request("text", "ordinary text", "user");
    user.message.extra_metadata = json!({"other":"kept"});
    let stored = message_append_with_config(user, &config)
        .await
        .unwrap()
        .value
        .data
        .unwrap();
    assert_eq!(stored.content, "ordinary text");
    assert_eq!(stored.extra_metadata, json!({"other":"kept"}));
    let assistant = request(
        "output",
        "[IMAGE:data:image/png;base64,iVBORw0KGgo=]",
        "assistant",
    );
    let expected = serde_json::to_value(&assistant.message).unwrap();
    let stored = message_append_with_config(assistant, &config)
        .await
        .unwrap()
        .value
        .data
        .unwrap();
    assert_eq!(serde_json::to_value(stored).unwrap(), expected);
}

#[tokio::test]
async fn poster_only_user_metadata_is_rejected_before_persistence() {
    let (_temp, config) = setup().await;
    let mut upload = request("poster", "video.mp4", "user");
    upload.message.extra_metadata = json!({
        "attachmentNames": ["video.mp4"],
        "attachmentPosters": ["data:image/png;base64,iVBORw0KGgo="]
    });
    assert!(message_append_with_config(upload, &config).await.is_err());
    assert!(
        conversations::blocking::get_messages(config.workspace_dir.clone(), "uploads".into())
            .await
            .unwrap()
            .is_empty()
    );
}
