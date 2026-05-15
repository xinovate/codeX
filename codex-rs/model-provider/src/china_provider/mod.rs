//! China provider model provider.
//!
//! Runtime provider for Chinese Chat API-compatible endpoints (Volcengine,
//! Doubao, Kimi, etc.) that only support the OpenAI Chat Completions API,
//! not the Responses API.
//!
//! This provider:
//! - Overrides `api_provider()` to set a Chat API base URL
//! - Uses bearer token auth from environment variables
//! - Signals `WireApi::Chat` so the core client dispatches to `ChatCompletionsClient`

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use codex_api::Provider;
use codex_api::SharedAuthProvider;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_models_manager::manager::SharedModelsManager;
use codex_models_manager::manager::StaticModelsManager;
use codex_models_manager::model_info;
use codex_protocol::openai_models::ModelInfo;
use codex_protocol::openai_models::ModelVisibility;
use codex_protocol::openai_models::ModelsResponse;
use tracing::warn;

use crate::bearer_auth_provider::BearerAuthProvider;
use crate::provider::ModelProvider;
use crate::provider::ProviderAccountResult;
use crate::provider::ProviderAccountState;

const MODELS_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

/// Fetch model IDs from the provider's `/models` endpoint (blocking).
/// Runs in a dedicated thread to avoid nested tokio runtime panic.
fn fetch_provider_models(info: &ModelProviderInfo) -> Vec<ModelInfo> {
    let info = info.clone();
    std::thread::spawn(move || fetch_provider_models_blocking(&info))
        .join()
        .unwrap_or_default()
}

fn fetch_provider_models_blocking(info: &ModelProviderInfo) -> Vec<ModelInfo> {
    let Some(base_url) = info.base_url.as_deref() else {
        return Vec::new();
    };

    let url = format!("{base_url}/models");
    let token = info
        .api_key()
        .ok()
        .flatten()
        .or_else(|| info.experimental_bearer_token.clone());

    let client = reqwest::blocking::Client::builder()
        .timeout(MODELS_REQUEST_TIMEOUT)
        .build();

    let Ok(client) = client else {
        return Vec::new();
    };

    let mut req = client
        .get(&url)
        .header("User-Agent", "codex-cli/0.1");

    if let Some(token) = token {
        req = req.bearer_auth(&token);
    }

    let resp = match req.send() {
        Ok(resp) => resp,
        Err(e) => {
            warn!("china provider /models request failed: {e}");
            return Vec::new();
        }
    };

    if !resp.status().is_success() {
        warn!("china provider /models returned {}", resp.status());
        return Vec::new();
    }

    let body = match resp.text() {
        Ok(body) => body,
        Err(e) => {
            warn!("china provider /models read body failed: {e}");
            return Vec::new();
        }
    };

    // Parse OpenAI-compatible {"data": [{"id": "..."}]} format
    let val: serde_json::Value = match serde_json::from_str(&body) {
        Ok(v) => v,
        Err(e) => {
            warn!("china provider /models parse failed: {e}");
            return Vec::new();
        }
    };

    let Some(data) = val.get("data").and_then(|d| d.as_array()) else {
        return Vec::new();
    };

    let mut models: Vec<ModelInfo> = data
        .iter()
        .rev() // Provider returns ascending (oldest first); reverse so newest is first
        .filter_map(|entry| {
            let id = entry.get("id")?.as_str()?.to_string();
            let mut model = model_info::model_info_from_slug(&id);
            model.visibility = ModelVisibility::List;
            model.display_name = id.clone();
            model.slug = id;
            Some(model)
        })
        .collect();

    // Assign priorities: priority 0 = highest (listed first).
    // After .rev(), models[0] is the newest model, so it gets priority 0.
    for (i, model) in models.iter_mut().enumerate() {
        model.priority = i as i32;
    }

    models
}

/// Runtime provider for Chinese platform Chat API endpoints.
///
/// Follows the same pattern as `AmazonBedrockModelProvider`: overrides
/// `api_provider()` and `api_auth()` while leaving the rest to defaults.
#[derive(Clone, Debug)]
pub(crate) struct ChinaModelProvider {
    pub(crate) info: ModelProviderInfo,
}

#[async_trait::async_trait]
impl ModelProvider for ChinaModelProvider {
    fn info(&self) -> &ModelProviderInfo {
        &self.info
    }

    fn auth_manager(&self) -> Option<Arc<AuthManager>> {
        None
    }

    async fn auth(&self) -> Option<CodexAuth> {
        None
    }

    fn account_state(&self) -> ProviderAccountResult {
        Ok(ProviderAccountState {
            account: None,
            requires_openai_auth: false,
        })
    }

    async fn api_provider(&self) -> codex_protocol::error::Result<Provider> {
        // Use the configured base_url directly.
        // The ChatCompletionsClient will append /chat/completions.
        let mut provider = self.info.to_api_provider(None)?;
        // Some China providers (e.g. Kimi) require a coding-agent User-Agent
        // header to allow access to the /chat/completions endpoint.
        // Set a default if not already configured by the user.
        if !provider.headers.contains_key(http::header::USER_AGENT) {
            provider.headers.insert(
                http::header::USER_AGENT,
                http::HeaderValue::from_static("claude-code/0.1"),
            );
        }
        Ok(provider)
    }

    async fn api_auth(&self) -> codex_protocol::error::Result<SharedAuthProvider> {
        // Resolve the API key from env_key or experimental_bearer_token
        let token = self.info.api_key().ok().flatten();
        // Also check experimental_bearer_token as fallback
        let token = token.or_else(|| self.info.experimental_bearer_token.clone());
        Ok(Arc::new(BearerAuthProvider {
            token,
            account_id: None,
            is_fedramp_account: false,
        }))
    }

    fn models_manager(
        &self,
        _codex_home: PathBuf,
        _config_model_catalog: Option<ModelsResponse>,
    ) -> SharedModelsManager {
        let models = fetch_provider_models(&self.info);
        if models.is_empty() {
            warn!("china provider /models returned empty list, /model picker will be unavailable");
        }
        let catalog = ModelsResponse { models };
        Arc::new(StaticModelsManager::new(None, catalog))
    }
}
