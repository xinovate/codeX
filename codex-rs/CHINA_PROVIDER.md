# Codex CLI - China Provider

支持国内大模型的 Codex CLI（火山引擎/豆包、Kimi、小米 Mimo 等），通过 Chat Completions API 接入。

---

## 快速开始

### 1. 编译

```bash
cd codex-rs
cargo build --package codex-cli
```

编译产物：`target/debug/codex`

建议把路径加到 PATH：

```bash
export PATH="$PWD/target/debug:$PATH"
```

### 2. 配置

创建 `~/.codex/config.toml`：

```toml
model = "你的模型名称"
model_provider = "你的provider标识"
personality = "pragmatic"

[model_providers.你的provider标识]
name = "Provider显示名称"
base_url = "https://你的API地址/v1"
env_key = "环境变量名"
wire_api = "chat"
```

然后设置 API Key 环境变量：

```bash
export 环境变量名="你的API Key"
```

建议写入 `~/.bashrc` 或 `~/.zshrc` 持久化。

### 3. 使用

```bash
# 非交互模式（单次任务）
codex exec "你的任务描述"

# 交互模式（对话式）
codex
```

## 各 Provider 配置示例

### 小米 Mimo

```toml
[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

### 火山引擎（豆包）

```toml
[model_providers.volcengine]
name = "Volcengine"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v1"
env_key = "VOLCENGINE_API_KEY"
wire_api = "chat"
```

### Kimi（月之暗面）

```toml
[model_providers.kimi]
name = "Kimi"
base_url = "https://api.moonshot.cn/v1"
env_key = "KIMI_API_KEY"
wire_api = "chat"
```

## 模型元数据配置（可选）

如果不配置，启动时会提示 `Model metadata not found`，不影响基本功能。要消除提示，创建 `~/.codex/custom_models.json`：

```json
{
  "models": [
    {
      "slug": "你的模型名称",
      "display_name": "模型显示名",
      "description": "模型描述",
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

在 `~/.codex/config.toml` 中添加：

```toml
model_catalog_json = "/Users/你的用户名/.codex/custom_models.json"
```

## 注意事项

- `wire_api = "chat"` 是必须的，所有国内 provider 都需要设置
- CLI 会自动将内部 Responses API 转换为 Chat Completions API 格式
- 支持工具调用、多轮对话、推理内容（thinking）
- API Key 通过环境变量配置，不会存入代码仓库
- `personality` 支持 `pragmatic`（务实）、`friendly`（友好）等风格

---

# Codex CLI - China Provider (English)

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
