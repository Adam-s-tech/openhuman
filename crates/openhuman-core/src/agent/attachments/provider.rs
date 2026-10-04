//! Ephemeral provider request preparation for durable workspace attachments.
use super::AttachmentAccessScope;
use crate::config::Config;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::sync::Arc;
use tinyinference_llm::{
    message::{ContentBlock, ImageRef, MediaRef, Message},
    model::{
        ChatModel, InputModality, InputSource, ModelProfile, ModelRequest, ModelResponse,
        ModelStream,
    },
};

pub(crate) fn wrap(
    inner: Arc<dyn ChatModel<()>>,
    config: Arc<Config>,
    model: &str,
    provider: &str,
) -> Arc<dyn ChatModel<()>> {
    let mut profile = inner.profile().cloned().unwrap_or_default();
    // Transport profiles describe serialization support, not selected-model facts.
    let catalog = tinyagents_registry::catalog::ModelCatalog::seed().ok();
    profile.modalities = catalog
        .as_ref()
        .and_then(|c| {
            c.profile(provider, model).or_else(|| {
                c.get_by_model_id(model)
                    .and_then(|e| c.profile(&e.provider, &e.model_id))
            })
        })
        .map(|p| p.modalities)
        .unwrap_or_default();
    if provider == "openhuman" || provider == "managed" {
        profile.modalities.image_in =
            crate::inference::provider::factory::oh_tier_supports_vision(model);
    }
    let route = if matches!(provider, "openhuman" | "managed") {
        "openhuman".to_string()
    } else {
        format!("{provider}:{model}")
    };
    if let Some(request) =
        crate::inference::provider::factory::model_limits_request("chat", &route, model, &config)
    {
        if let Some(inputs) = tinyinference_llm::model::discover::model_limits_cache()
            .get_variant(&request.endpoint, &request.model, &request.cache_variant())
            .effective()
            .and_then(|l| l.input_modalities)
        {
            profile.modalities.image_in = inputs.iter().any(|m| m == "image");
            profile.modalities.audio_in = inputs.iter().any(|m| m == "audio");
            profile.modalities.video_in = inputs.iter().any(|m| m == "video");
            profile.modalities.document_in = inputs
                .iter()
                .any(|m| matches!(m.as_str(), "document" | "pdf" | "file"));
        }
    }
    if tinyinference_local::profile::is_local_provider_string(provider) {
        profile.modalities.image_in |= tinyinference_llm::model::model_id_supports_vision(model);
    }
    profile.modalities.image_in |=
        crate::inference::model_context::model_vision_enabled(model, &config);
    Arc::new(AttachmentModel {
        inner,
        config,
        profile,
        fallback_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
    })
}

/// Injected models supply their own selected-model facts through ModelProfile.
/// Transport support is still required for every native media block.
pub(crate) fn wrap_injected(
    inner: Arc<dyn ChatModel<()>>,
    config: Arc<Config>,
) -> Arc<dyn ChatModel<()>> {
    let profile = inner.profile().cloned().unwrap_or_default();
    Arc::new(AttachmentModel {
        inner,
        config,
        profile,
        fallback_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
    })
}

struct AttachmentModel {
    inner: Arc<dyn ChatModel<()>>,
    config: Arc<Config>,
    profile: ModelProfile,
    fallback_cache: std::sync::Mutex<std::collections::HashMap<String, String>>,
}
impl AttachmentModel {
    fn native(&self, modality: InputModality, mime: &str) -> bool {
        let known = match modality {
            InputModality::Image => self.profile.modalities.image_in,
            InputModality::Audio => self.profile.modalities.audio_in,
            InputModality::Video => self.profile.modalities.video_in,
            InputModality::Document => self.profile.modalities.document_in,
        };
        known
            && self
                .inner
                .supports_input(modality, mime, InputSource::Base64)
    }
    async fn read(
        &self,
        path: &str,
        modality: InputModality,
        scope: &AttachmentAccessScope,
    ) -> tinyinference_llm::Result<Vec<u8>> {
        if scope.external_channel {
            return Err(tinyinference_llm::Error::Model(
                "local attachment reads are disabled for external channel input".into(),
            ));
        }
        let path = super::resolve_path(&self.config, path, scope)
            .await
            .map_err(|e| tinyinference_llm::Error::Model(e.to_string()))?;
        let (_, file_mb, _) = self.config.multimodal_files.effective_limits();
        let (_, image_mb) = self.config.multimodal.effective_limits();
        if modality == InputModality::Image && self.config.multimodal.max_images == 0 {
            return Err(tinyinference_llm::Error::Model(
                "image attachments are disabled".into(),
            ));
        }
        let mb = if modality == InputModality::Image {
            image_mb
        } else {
            file_mb
        };
        let metadata = tokio::fs::metadata(&path)
            .await
            .map_err(|e| tinyinference_llm::Error::Model(e.to_string()))?;
        if !metadata.is_file() || metadata.len() > (mb * 1024 * 1024) as u64 {
            return Err(tinyinference_llm::Error::Model(
                "attachment exceeds configured file limit".into(),
            ));
        }
        use tokio::io::AsyncReadExt;
        let file = tokio::fs::File::open(path)
            .await
            .map_err(|e| tinyinference_llm::Error::Model(e.to_string()))?;
        let mut bytes = Vec::new();
        file.take((mb * 1024 * 1024 + 1) as u64)
            .read_to_end(&mut bytes)
            .await
            .map_err(|e| tinyinference_llm::Error::Model(e.to_string()))?;
        if bytes.len() > mb * 1024 * 1024 {
            return Err(tinyinference_llm::Error::Model(
                "attachment exceeds configured file limit".into(),
            ));
        }
        Ok(bytes)
    }
    async fn prepare(&self, mut request: ModelRequest) -> tinyinference_llm::Result<ModelRequest> {
        let scope = super::take_request_scope(&mut request.metadata);
        if let Some(Message::User(user)) = request
            .messages
            .iter()
            .rev()
            .find(|m| matches!(m, Message::User(_)))
        {
            let images = user
                .content
                .iter()
                .filter(|b| matches!(b, ContentBlock::Image(_)))
                .count();
            let files = user
                .content
                .iter()
                .filter(|b| {
                    matches!(
                        b,
                        ContentBlock::Audio(_) | ContentBlock::Video(_) | ContentBlock::Document(_)
                    )
                })
                .count();
            let (max_images, _) = self.config.multimodal.effective_limits();
            let (max_files, _, _) = self.config.multimodal_files.effective_limits();
            if images > max_images
                || files > max_files
                || self.config.multimodal_files.max_files == 0 && images + files > 0
            {
                return Err(tinyinference_llm::Error::Model(
                    "attachment count exceeds configured limit".into(),
                ));
            }
        }
        for message in &mut request.messages {
            let Message::User(user) = message else {
                continue;
            };
            let mut out = Vec::new();
            for block in std::mem::take(&mut user.content) {
                let Some((modality, path, mut mime, bytes, recovered)) =
                    self.resolve_block(&block, &scope).await?
                else {
                    out.push(block);
                    continue;
                };
                if modality == InputModality::Image && bytes.is_empty() {
                    return Err(tinyinference_llm::Error::Model(
                        "image reference has no bytes".into(),
                    ));
                }
                if modality == InputModality::Image {
                    mime = tinyagents_harness::multimodal::mime::image_mime_from_magic(&bytes)
                        .map(str::to_owned)
                        .or_else(|| {
                            tinyagents_harness::multimodal::mime::detect_image_mime(
                                Some(std::path::Path::new(&path)),
                                &bytes,
                                None,
                            )
                        })
                        .ok_or_else(|| {
                            tinyinference_llm::Error::Model(
                                "image attachment type cannot be determined".into(),
                            )
                        })?;
                }
                if self.native(modality, &mime) {
                    if recovered {
                        out.push(ContentBlock::Text(format!(
                            "[Recovered attachment: {mime}; {} bytes; workspace path: {path}]",
                            bytes.len()
                        )));
                    }

                    let bytes = if mime == "image/png" {
                        tinyagents_harness::multimodal::optimize_png_lossless(&bytes)
                            .unwrap_or(bytes)
                    } else {
                        bytes
                    };
                    let data = STANDARD.encode(bytes);
                    out.push(match modality {
                        InputModality::Image => ContentBlock::Image(ImageRef {
                            url: format!("data:{mime};base64,{data}"),
                            mime_type: Some(mime),
                        }),
                        InputModality::Audio => ContentBlock::Audio(MediaRef::base64(data, mime)),
                        InputModality::Video => ContentBlock::Video(MediaRef::base64(data, mime)),
                        InputModality::Document => {
                            ContentBlock::Document(MediaRef::base64(data, mime))
                        }
                    });
                } else {
                    let key = self.fallback_key(&path, &mime, &bytes);
                    let cached = self
                        .fallback_cache
                        .lock()
                        .expect("fallback cache")
                        .get(&key)
                        .cloned();
                    let cached = match cached {
                        Some(text) => Some(text),
                        None => self.cached_fallback(&path, &key, &scope).await,
                    };
                    let text = match cached {
                        Some(text) => text,
                        None => {
                            let (text, cacheable) =
                                self.fallback(modality, &path, &mime, &bytes).await?;
                            if cacheable {
                                self.save_fallback(&path, &key, &text, &scope).await;
                            }
                            let mut cache = self.fallback_cache.lock().expect("fallback cache");
                            if cache.len() < 16 {
                                cache.insert(key, text.clone());
                            }
                            text
                        }
                    };
                    out.push(ContentBlock::Text(text));
                }
            }
            user.content = out;
        }
        Ok(request)
    }
}
#[async_trait]
impl ChatModel<()> for AttachmentModel {
    fn profile(&self) -> Option<&ModelProfile> {
        Some(&self.profile)
    }
    fn cache_identity(&self) -> Option<String> {
        self.inner.cache_identity()
    }
    fn supports_input(&self, m: InputModality, t: &str, s: InputSource) -> bool {
        self.inner.supports_input(m, t, s)
    }
    async fn invoke(
        &self,
        state: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelResponse> {
        self.inner.invoke(state, self.prepare(request).await?).await
    }
    async fn stream(
        &self,
        state: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelStream> {
        self.inner.stream(state, self.prepare(request).await?).await
    }
}

#[path = "provider_cache.rs"]
mod cache;
#[path = "provider_fallback.rs"]
mod fallback;
#[path = "provider_source.rs"]
mod source;
#[cfg(test)]
#[path = "provider_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "provider_routing_tests.rs"]
mod routing_tests;
