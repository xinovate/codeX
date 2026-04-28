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

use std::sync::Arc;

use codex_api::Provider;
use codex_api::SharedAuthProvider;
use codex_login::AuthManager;
use codex_login::CodexAuth;
use codex_model_provider_info::ModelProviderInfo;
use codex_protocol::error::Result;

use crate::bearer_auth_provider::BearerAuthProvider;
use crate::provider::ModelProvider;

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

    async fn api_provider(&self) -> Result<Provider> {
        // Use the configured base_url directly.
        // The ChatCompletionsClient will append /chat/completions.
        self.info.to_api_provider(None)
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
}
