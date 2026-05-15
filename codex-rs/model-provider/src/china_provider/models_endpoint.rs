//! Models endpoint client for China providers.
//!
//! Fetches available models from the provider's `/models` API (OpenAI-compatible)
//! and converts them to `ModelInfo` for use in the Codex model picker.

use async_trait::async_trait;
use codex_login::default_client::build_reqwest_client;
use codex_model_provider_info::ModelProviderInfo;
use codex_models_manager::manager::ModelsEndpointClient;
use codex_models_manager::model_info;
use codex_protocol::error::Result as CoreResult;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelVisibility;
use std::time::Duration;
use tracing::warn;

const MODELS_REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Parse model IDs from an OpenAI-compatible `/v1/models` JSON response.
fn parse_model_ids(body: &str) -> Vec<String> {
    let val: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    val.get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|entry| entry.get("id").and_then(|id| id.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Models endpoint client that calls the provider's `/models` API.
#[derive(Debug)]
pub(crate) struct ChinaModelsEndpoint {
    provider_info: ModelProviderInfo,
}

impl ChinaModelsEndpoint {
    pub(crate) fn new(provider_info: ModelProviderInfo) -> Self {
        Self { provider_info }
    }
}

#[async_trait]
impl ModelsEndpointClient for ChinaModelsEndpoint {
    fn has_command_auth(&self) -> bool {
        // Return true so OpenAiModelsManager will actually call our endpoint.
        // China providers always use bearer token auth from env vars.
        true
    }

    async fn uses_codex_backend(&self) -> bool {
        false
    }

    async fn list_models(
        &self,
        _client_version: &str,
    ) -> CoreResult<(Vec<ModelInfo>, Option<String>)> {
        let Some(base_url) = self.provider_info.base_url.as_deref() else {
            warn!("china provider has no base_url configured");
            return Ok((Vec::new(), None));
        };

        let url = format!("{base_url}/models");

        let token = self
            .provider_info
            .api_key()
            .ok()
            .flatten()
            .or_else(|| self.provider_info.experimental_bearer_token.clone());

        let client = build_reqwest_client();
        let mut req = client
            .get(&url)
            .timeout(MODELS_REQUEST_TIMEOUT)
            .header("User-Agent", "codex-cli/0.1");

        if let Some(token) = token {
            req = req.bearer_auth(&token);
        }

        let resp = match req.send().await {
            Ok(resp) => resp,
            Err(e) => {
                warn!("china provider models request failed: {e}");
                return Ok((Vec::new(), None));
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            warn!("china provider /models returned {status}: {body}");
            return Ok((Vec::new(), None));
        }

        let body = resp.text().await.map_err(|e| {
            warn!("failed to read models response body: {e}");
            codex_protocol::error::CodexErr::InvalidRequest("failed to read response".into())
        })?;

        let model_ids = parse_model_ids(&body);
        if model_ids.is_empty() {
            warn!("china provider /models returned no models");
            return Ok((Vec::new(), None));
        }

        let models: Vec<ModelInfo> = model_ids
            .into_iter()
            .map(|id| {
                let mut info = model_info::model_info_from_slug(&id);
                // Make the model visible in the picker.
                info.visibility = ModelVisibility::List;
                info.display_name = id.clone();
                info.slug = id;
                info
            })
            .collect();

        Ok((models, None))
    }
}
