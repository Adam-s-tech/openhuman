//! This host's [`MemoryToolHost`]: what `tinymemory-tools` needs that only
//! OpenHuman can answer.
//!
//! The memory agent tools (tree retrieval, raw chunk and entity search, hybrid
//! and vector search, tool-scoped rules) live in `tinymemory-tools`, generic
//! over a host. This is the one host they run under here:
//!
//! - the **guarded** driver, so the seven policy steps (tier, source scope,
//!   taint, budgets, redaction, audit) run inside every call the tools make;
//! - the per-turn memory-source allowlist, read from the host's task-local;
//! - the embedding model, resolved from the current `Config`;
//! - `memory_tree`'s `ingest_document` mode, which writes through the host's own
//!   tree-ingest path.

use std::sync::Arc;

use async_trait::async_trait;
use tinymemory_tools::{MemoryToolHost, QueryEmbedder};
use tinytools::{Tool, ToolResult};

use crate::config::rpc as config_rpc;
use crate::inference::embedding_host::{provider_from_config, EmbeddingProvider};
use crate::memory::api::provider::MemoryProvider;
use crate::memory::ops::guard::active_memory_guard;
use crate::memory::query::MemoryTreeIngestDocumentTool;

/// The host the memory agent tools run under in this process.
#[derive(Clone, Copy, Debug, Default)]
pub struct HostMemoryTools;

/// The host's embedding provider, in the tools' vocabulary.
struct HostEmbedder(Box<dyn EmbeddingProvider>);

#[async_trait]
impl QueryEmbedder for HostEmbedder {
    async fn embed_one(&self, text: &str) -> Result<Vec<f32>, String> {
        self.0.embed_one(text).await.map_err(|e| e.to_string())
    }

    fn signature(&self) -> String {
        self.0.signature()
    }
}

#[async_trait]
impl MemoryToolHost for HostMemoryTools {
    async fn provider(&self) -> Result<Arc<dyn MemoryProvider>, String> {
        let guard = active_memory_guard().await?;
        Ok(guard)
    }

    fn chunk_source_allowed(&self, tags: &[String], source_id: &str) -> bool {
        crate::memory::source_scope::chunk_source_allowed(tags, source_id)
    }

    async fn embedder(&self) -> Result<Box<dyn QueryEmbedder>, String> {
        let config = config_rpc::load_config_with_timeout()
            .await
            .map_err(|e| format!("load config failed: {e}"))?;
        let provider = provider_from_config(&config)
            .map_err(|e| format!("embedding provider failed: {e}"))?;
        Ok(Box::new(HostEmbedder(provider)))
    }

    async fn ingest_document(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        MemoryTreeIngestDocumentTool.execute(args).await
    }
}
