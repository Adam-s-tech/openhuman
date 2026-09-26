//! OpenHuman authentication adapter for hosted speech-to-text.

use crate::api::config::effective_backend_api_url;
use crate::api::BackendOAuthClient;
use crate::config::Config;
use crate::rpc::RpcOutcome;

pub use tinyinference_voice::cloud::{CloudTranscribeOptions, CloudTranscribeResult};

/// Transcribe renderer-supplied base64 audio through the hosted backend.
pub async fn transcribe_cloud(
    config: &Config,
    audio_base64: &str,
    options: &CloudTranscribeOptions,
) -> Result<RpcOutcome<CloudTranscribeResult>, String> {
    // The session JWT or the TinyHumans API key. The vendored client sends it
    // as `Authorization: Bearer`, which the backend accepts for either.
    let token = crate::security::credentials::session_support::resolve_backend_credential(config)?
        .into_secret();
    let client = BackendOAuthClient::new(&effective_backend_api_url(&config.api_url))
        .map_err(|error| error.to_string())?;
    let url = client
        .url_for("/openai/v1/audio/transcriptions")
        .map_err(|error| error.to_string())?;
    let http = client
        .raw_client()
        .map_err(crate::api::flatten_authed_error)?;
    let result =
        tinyinference_voice::cloud::transcribe(&http, url, &token, audio_base64, options).await?;
    Ok(RpcOutcome::single_log(
        result,
        "cloud STT via POST /openai/v1/audio/transcriptions",
    ))
}
