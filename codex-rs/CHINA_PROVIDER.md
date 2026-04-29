# Codex CLI - China Provider

Codex CLI with support for Chinese LLM providers (Volcengine/Doubao, Kimi, XiaomiMimo, etc.) via Chat Completions API.

## Quick Start

### 1. Build

```bash
cd codex-rs
cargo build --package codex-cli
```

Binary: `target/debug/codex`

### 2. Configure

Create `~/.codex/config.toml`:

```toml
model = "your-model-name"
model_provider = "your-provider"
personality = "pragmatic"

[model_providers.your-provider]
name = "YourProviderName"
base_url = "https://your-provider-api-endpoint/v1"
env_key = "YOUR_API_KEY_ENV_VAR"
wire_api = "chat"

[projects."/path/to/your/project"]
trust_level = "trusted"
```

Then set the API key environment variable:

```bash
export YOUR_API_KEY_ENV_VAR="your-api-key-here"
```

### 3. Use

```bash
# Non-interactive (one-shot)
codex exec "your task here"

# Interactive
codex
```

## Provider Examples

### XiaomiMimo

```toml
[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

### Volcengine (Doubao)

```toml
[model_providers.volcengine]
name = "Volcengine"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v1"
env_key = "VOLCENGINE_API_KEY"
wire_api = "chat"
```

### Kimi

```toml
[model_providers.kimi]
name = "Kimi"
base_url = "https://api.moonshot.cn/v1"
env_key = "KIMI_API_KEY"
wire_api = "chat"
```

## Model Metadata (Optional)

To remove the "Model metadata not found" warning, create `~/.codex/custom_models.json`:

```json
{
  "models": [
    {
      "slug": "your-model-name",
      "display_name": "Your Model",
      "description": "Your model description.",
      "default_reasoning_level": null,
      "supported_reasoning_levels": [],
      "shell_type": "shell_command",
      "visibility": "list",
      "supported_in_api": true,
      "priority": 50,
      "base_instructions": "You are a helpful coding assistant.",
      "model_messages": null,
      "supports_reasoning_summaries": false,
      "default_reasoning_summary": "auto",
      "support_verbosity": false,
      "truncation_policy": {"mode": "bytes", "limit": 10000},
      "supports_parallel_tool_calls": false,
      "context_window": 128000,
      "max_context_window": 128000,
      "effective_context_window_percent": 95,
      "input_modalities": ["text"],
      "supports_search_tool": false
    }
  ]
}
```

Add to `~/.codex/config.toml`:

```toml
model_catalog_json = "/Users/your-username/.codex/custom_models.json"
```

## Notes

- `wire_api = "chat"` is required for all Chinese providers
- The CLI auto-converts between Responses API (internal) and Chat Completions API (provider)
- Tool calls, multi-turn conversations, and reasoning content are all supported
- API keys are stored in environment variables, never in config files
