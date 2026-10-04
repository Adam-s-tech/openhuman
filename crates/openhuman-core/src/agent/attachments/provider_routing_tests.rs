use super::*;
use futures::StreamExt;
use std::sync::Mutex;
use tinyinference_llm::message::UserMessage;

/// A transport whose advertised media support is independent of model facts.
struct CaptureModel {
    profile: ModelProfile,
    transport_inputs: Vec<InputModality>,
    requests: Mutex<Vec<(&'static str, ModelRequest)>>,
    reply: &'static str,
}
impl CaptureModel {
    fn new(transport_inputs: Vec<InputModality>) -> Self {
        Self {
            profile: ModelProfile::default(),
            transport_inputs,
            requests: Mutex::new(Vec::new()),
            reply: "main reply",
        }
    }
}
#[async_trait]
impl ChatModel<()> for CaptureModel {
    fn profile(&self) -> Option<&ModelProfile> {
        Some(&self.profile)
    }
    fn cache_identity(&self) -> Option<String> {
        Some("attachment-routing-probe-v1".into())
    }
    fn supports_input(&self, modality: InputModality, mime: &str, source: InputSource) -> bool {
        source == InputSource::Base64
            && self.transport_inputs.contains(&modality)
            && matches!(mime, "audio/wav" | "application/pdf" | "image/jpeg")
    }
    async fn invoke(
        &self,
        _: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelResponse> {
        self.requests.lock().unwrap().push(("invoke", request));
        Ok(ModelResponse::assistant(self.reply))
    }
    async fn stream(
        &self,
        _: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelStream> {
        self.requests.lock().unwrap().push(("stream", request));
        Ok(ModelStream::new(Box::pin(futures::stream::empty())))
    }
}

fn offline_config(root: &std::path::Path) -> Arc<Config> {
    let mut config = Config::default();
    config.action_dir = root.into();
    config.workspace_dir = root.join("internal");
    config.config_path = config.workspace_dir.join("config.toml");
    config.modules.enabled = false;
    config.modules.allow_download = false;
    Arc::new(config)
}
fn controlled_wrapper(
    inner: Arc<CaptureModel>,
    config: Arc<Config>,
    model_inputs: &[InputModality],
) -> AttachmentModel {
    let mut profile = ModelProfile::default();
    profile.modalities.image_in = model_inputs.contains(&InputModality::Image);
    profile.modalities.audio_in = model_inputs.contains(&InputModality::Audio);
    profile.modalities.document_in = model_inputs.contains(&InputModality::Document);
    AttachmentModel {
        inner,
        config,
        profile,
        fallback_cache: Mutex::new(Default::default()),
    }
}
fn user_request(blocks: Vec<ContentBlock>) -> ModelRequest {
    ModelRequest::new(vec![Message::User(UserMessage { content: blocks })])
}
fn local_media(modality: InputModality, path: &str, mime: &str) -> ContentBlock {
    let media = MediaRef::Path {
        path: path.into(),
        media_type: Some(mime.into()),
    };
    match modality {
        InputModality::Audio => ContentBlock::Audio(media),
        InputModality::Document => ContentBlock::Document(media),
        _ => panic!("fixture supports audio and documents"),
    }
}

#[tokio::test]
async fn native_audio_and_pdf_require_both_model_facts_and_transport_support() {
    let temp = tempfile::tempdir().unwrap();
    let config = offline_config(temp.path());
    for (modality, path, mime, bytes) in [
        (
            InputModality::Audio,
            "voice.wav",
            "audio/wav",
            b"original audio".as_slice(),
        ),
        // An intentionally unparsable PDF proves native forwarding does not
        // depend on TinyDocs parsing, a loaded module, or a text readout.
        (
            InputModality::Document,
            "report.pdf",
            "application/pdf",
            b"original pdf".as_slice(),
        ),
    ] {
        tokio::fs::write(temp.path().join(path), bytes)
            .await
            .unwrap();
        for (model_accepts, transport_accepts) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            let transport = Arc::new(CaptureModel::new(if transport_accepts {
                vec![modality]
            } else {
                vec![]
            }));
            let model = controlled_wrapper(
                transport.clone(),
                config.clone(),
                if model_accepts {
                    std::slice::from_ref(&modality)
                } else {
                    &[]
                },
            );
            let request = user_request(vec![local_media(modality, path, mime)]);
            assert_eq!(
                model.invoke(&(), request.clone()).await.unwrap().text(),
                "main reply"
            );
            let captured = transport.requests.lock().unwrap().clone();
            assert_eq!(captured.len(), 1);
            let Message::User(user) = &captured[0].1.messages[0] else {
                panic!("expected user message")
            };
            assert_eq!(user.content.len(), 1);
            if model_accepts && transport_accepts {
                let media = match &user.content[0] {
                    ContentBlock::Audio(media) if modality == InputModality::Audio => media,
                    ContentBlock::Document(media) if modality == InputModality::Document => media,
                    _ => panic!("native modality must reach the transport"),
                };
                assert_eq!(*media, MediaRef::base64(STANDARD.encode(bytes), mime));
            } else {
                let ContentBlock::Text(text) = &user.content[0] else {
                    panic!("unsupported media must use fallback text")
                };
                assert!(text.contains(path));
                assert!(text.contains(mime));
                assert!(!text.contains("base64"));
                #[cfg(feature = "documents")]
                if modality == InputModality::Document {
                    assert!(text.contains("PDF text extraction unavailable"));
                    assert!(text.contains("modules are disabled"));
                }
            }
            assert_eq!(
                tokio::fs::read(temp.path().join(path)).await.unwrap(),
                bytes
            );
            assert!(serde_json::to_string(&request).unwrap().contains(path));
        }
    }
}

#[tokio::test]
async fn text_archive_and_unavailable_pdf_dispatch_to_distinct_fallbacks() {
    use std::io::Write;
    let temp = tempfile::tempdir().unwrap();
    let config = offline_config(temp.path());
    tokio::fs::write(temp.path().join("note.txt"), b"visible text payload")
        .await
        .unwrap();
    tokio::fs::write(temp.path().join("report.pdf"), b"original pdf bytes")
        .await
        .unwrap();
    let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    archive
        .start_file("inside.txt", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive
        .write_all(b"member body must not be extracted")
        .unwrap();
    let archive = archive.finish().unwrap().into_inner();
    tokio::fs::write(temp.path().join("bundle.zip"), &archive)
        .await
        .unwrap();
    let transport = Arc::new(CaptureModel::new(vec![]));
    let model = controlled_wrapper(transport.clone(), config, &[]);
    let request = user_request(vec![
        ContentBlock::Text("before".into()),
        local_media(InputModality::Document, "note.txt", "text/plain"),
        local_media(InputModality::Document, "bundle.zip", "application/zip"),
        local_media(InputModality::Document, "report.pdf", "application/pdf"),
        ContentBlock::Text("after".into()),
    ]);
    model.invoke(&(), request.clone()).await.unwrap();
    let captured = transport.requests.lock().unwrap().clone();
    assert_eq!(captured.len(), 1);
    let Message::User(user) = &captured[0].1.messages[0] else {
        panic!("expected user message")
    };
    assert_eq!(user.content.len(), 5);
    assert_eq!(user.content[0], ContentBlock::Text("before".into()));
    assert_eq!(user.content[4], ContentBlock::Text("after".into()));
    let text = |index: usize| match &user.content[index] {
        ContentBlock::Text(text) => text.as_str(),
        _ => panic!("expected fallback text"),
    };
    assert!(text(1).contains("Document content (untrusted)"));
    assert!(text(1).contains("visible text payload"));
    assert!(text(2).contains("Archive listing"));
    assert!(text(2).contains("inside.txt"));
    assert!(!text(2).contains("member body must not be extracted"));
    #[cfg(feature = "documents")]
    {
        assert!(text(3).contains("using the published PDF text-layer reader"));
        assert!(text(3).contains("PDF text extraction unavailable"));
        assert!(text(3).contains("modules are disabled"));
    }
    assert!(text(3).contains("report.pdf"));
    assert!(!temp.path().join("inside.txt").exists());
    assert_eq!(
        tokio::fs::read(temp.path().join("bundle.zip"))
            .await
            .unwrap(),
        archive
    );
    assert!(matches!(&request.messages[0], Message::User(user)
        if matches!(&user.content[1], ContentBlock::Document(MediaRef::Path { path, .. }) if path == "note.txt")));
}

#[tokio::test]
async fn paid_vision_readout_is_reused_across_recreated_wrappers_and_invalidated_by_source_changes()
{
    let temp = tempfile::tempdir().unwrap();
    let original = temp.path().join("uploads/thread/image/original.jpg");
    tokio::fs::create_dir_all(original.parent().unwrap())
        .await
        .unwrap();
    let first_bytes = b"\xff\xd8\xff\xe0original";
    tokio::fs::write(&original, first_bytes).await.unwrap();
    let config = offline_config(temp.path());
    let mut vision = CaptureModel::new(vec![InputModality::Image]);
    vision.profile.provider = Some("injected".into());
    vision.profile.modalities.image_in = true;
    vision.reply = "mock vision readout";
    let vision = Arc::new(vision);
    let _override =
        crate::inference::provider::factory::test_provider_override::install_model(vision.clone());
    let main = Arc::new(CaptureModel::new(vec![]));
    let request = user_request(vec![ContentBlock::Image(ImageRef {
        url: "uploads/thread/image/original.jpg".into(),
        mime_type: Some("image/jpeg".into()),
    })]);
    for expected_vision_calls in [1, 1] {
        let model = controlled_wrapper(main.clone(), config.clone(), &[]);
        model.invoke(&(), request.clone()).await.unwrap();
        assert_eq!(vision.requests.lock().unwrap().len(), expected_vision_calls);
    }
    assert_eq!(tokio::fs::read(&original).await.unwrap(), first_bytes);
    tokio::fs::write(&original, b"\xff\xd8\xff\xe0changed")
        .await
        .unwrap();
    let model = controlled_wrapper(main.clone(), config, &[]);
    model.invoke(&(), request.clone()).await.unwrap();
    let vision_requests = vision.requests.lock().unwrap().clone();
    assert_eq!(vision_requests.len(), 2);
    for (_, call) in &vision_requests {
        assert_eq!(call.max_tokens, Some(4096));
        let Message::User(user) = &call.messages[0] else {
            panic!("expected vision user message")
        };
        assert!(user.content.iter().any(|block| matches!(block, ContentBlock::Image(image) if image.url.starts_with("data:image/jpeg;base64,"))));
    }
    let calls = main.requests.lock().unwrap().clone();
    assert_eq!(calls.len(), 3);
    for (_, call) in &calls {
        assert!(call.messages[0]
            .text()
            .contains("Image readout:\nmock vision readout"));
        let Message::User(user) = &call.messages[0] else {
            panic!("expected main user message")
        };
        assert!(user
            .content
            .iter()
            .all(|block| matches!(block, ContentBlock::Text(_))));
    }
    assert!(
        matches!(&request.messages[0], Message::User(user) if matches!(&user.content[0], ContentBlock::Image(image) if image.url == "uploads/thread/image/original.jpg"))
    );
}

#[tokio::test]
async fn streaming_prepares_only_the_ephemeral_request_and_preserves_order_and_options() {
    let temp = tempfile::tempdir().unwrap();
    let config = offline_config(temp.path());
    tokio::fs::write(temp.path().join("voice.wav"), b"stream audio")
        .await
        .unwrap();
    tokio::fs::write(temp.path().join("report.pdf"), b"stream pdf")
        .await
        .unwrap();
    let transport = Arc::new(CaptureModel::new(vec![
        InputModality::Audio,
        InputModality::Document,
    ]));
    let model = controlled_wrapper(
        transport.clone(),
        config,
        &[InputModality::Audio, InputModality::Document],
    );
    let mut request = ModelRequest::new(vec![
        Message::system("system prompt"),
        Message::assistant("prior reply"),
        Message::User(UserMessage {
            content: vec![
                ContentBlock::Text("before".into()),
                local_media(InputModality::Audio, "voice.wav", "audio/wav"),
                ContentBlock::Text("between".into()),
                local_media(InputModality::Document, "report.pdf", "application/pdf"),
                ContentBlock::Text("after".into()),
            ],
        }),
    ]);
    request.temperature = Some(0.37);
    request.max_tokens = Some(128);
    let durable = serde_json::to_value(&request).unwrap();
    assert!(model
        .stream(&(), request.clone())
        .await
        .unwrap()
        .collect::<Vec<_>>()
        .await
        .is_empty());
    assert_eq!(serde_json::to_value(&request).unwrap(), durable);
    let captured = transport.requests.lock().unwrap().clone();
    assert_eq!(captured.len(), 1);
    assert_eq!(captured[0].0, "stream");
    let forwarded = &captured[0].1;
    assert_eq!(forwarded.temperature, request.temperature);
    assert_eq!(forwarded.max_tokens, request.max_tokens);
    assert_eq!(
        serde_json::to_value(&forwarded.messages[..2]).unwrap(),
        serde_json::to_value(&request.messages[..2]).unwrap()
    );
    let Message::User(user) = &forwarded.messages[2] else {
        panic!("expected user message")
    };
    assert_eq!(user.content.len(), 5);
    assert_eq!(user.content[0], ContentBlock::Text("before".into()));
    assert_eq!(
        user.content[1],
        ContentBlock::Audio(MediaRef::base64(
            STANDARD.encode(b"stream audio"),
            "audio/wav"
        ))
    );
    assert_eq!(user.content[2], ContentBlock::Text("between".into()));
    assert_eq!(
        user.content[3],
        ContentBlock::Document(MediaRef::base64(
            STANDARD.encode(b"stream pdf"),
            "application/pdf"
        ))
    );
    assert_eq!(user.content[4], ContentBlock::Text("after".into()));
    assert_eq!(
        tokio::fs::read(temp.path().join("voice.wav"))
            .await
            .unwrap(),
        b"stream audio"
    );
    assert_eq!(
        tokio::fs::read(temp.path().join("report.pdf"))
            .await
            .unwrap(),
        b"stream pdf"
    );
}
