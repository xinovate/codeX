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

use codex_api::Provider;
use codex_api::SharedAuthProvider;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_models_manager::collaboration_mode_presets::CollaborationModesConfig;
use codex_models_manager::manager::SharedModelsManager;
use codex_models_manager::manager::StaticModelsManager;
use codex_protocol::error::Result;
use codex_protocol::openai_models::ModelsResponse;

use crate::bearer_auth_provider::BearerAuthProvider;
use crate::provider::ModelProvider;
use crate::provider::ProviderAccountResult;
use crate::provider::ProviderAccountState;

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

    async fn api_provider(&self) -> Result<Provider> {
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

    async fn api_auth(&self) -> Result<SharedAuthProvider> {
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
        config_model_catalog: Option<ModelsResponse>,
        collaboration_modes_config: CollaborationModesConfig,
    ) -> SharedModelsManager {
        // China providers always use static models manager since
        // the /models endpoint may not be available.
        let model_catalog =
            config_model_catalog.unwrap_or_else(|| ModelsResponse { models: vec![] });
        Arc::new(StaticModelsManager::new(
            None,
            model_catalog,
            collaboration_modes_config,
        ))
    }
}
