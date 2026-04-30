# Codex CLI - China Provider

支持国内大模型的 Codex CLI（火山引擎/豆包、Kimi、小米 Mimo 等），通过 Chat Completions API 接入。

---

## 快速开始

### 1. 获取 Codex

**方式一：下载预编译二进制（推荐）**

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载，无需安装 Rust：

| 平台 | 下载包 |
|------|--------|
| Linux x86_64 | `codex-linux-x64.tar.gz` |
| Linux ARM64 | `codex-linux-arm64.tar.gz` |
| Windows x64 | `codex-windows-x64.zip` |
| macOS | 无预编译包，需从源码构建 |

**方式二：从源码构建（macOS 或需要最新代码时）**

需要先安装 [Rust](https://rustup.rs/)，然后：

```bash
git clone https://github.com/xinovate/codex.git
cd codex/codex-rs
cargo build --release --bin codex
```

编译产物：`target/release/codex`

### 2. 安装

**Linux / macOS：**

```bash
sudo cp codex /usr/local/bin/    # 或 target/release/codex
```

**Windows：**

将 `codex.exe` 所在目录添加到系统 PATH（详见 [README.md](../README.md)）。

验证：

```bash
codex --version
```

### 3. 配置

创建配置文件（Linux/macOS: `~/.codex/config.toml`，Windows: `%USERPROFILE%\.codex\config.toml`）。

以小米 Mimo 为例，完整配置如下：

```toml
model = "mimo-model-name"
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

各字段说明：

| 字段 | 必填 | 说明 |
|------|------|------|
| `model` | 是 | 模型名称，如 `"mimo-model-name"`、`"doubao-pro-32k"` |
| `model_provider` | 是 | Provider 标识，对应 `[model_providers.xxx]` 中的 `xxx` |
| `name` | 是 | Provider 显示名称 |
| `base_url` | 是 | API 地址，必须以 `/v1` 结尾 |
| `env_key` | 是 | API Key 对应的环境变量名 |
| `wire_api` | 是 | 固定填 `"chat"`（小写），国内 provider 必须设置 |

### 4. 设置 API Key

将 `env_key` 指定的环境变量设置为你的 API Key：

```bash
# Linux / macOS（当前会话）
export MIMO_API_KEY="sk-xxxxxxxx"

# Windows PowerShell（当前会话）
$env:MIMO_API_KEY = "sk-xxxxxxxx"
```

**永久生效：**

- **bash**：`echo 'export MIMO_API_KEY="sk-xxxxxxxx"' >> ~/.bashrc`
- **zsh**（macOS 默认）：`echo 'export MIMO_API_KEY="sk-xxxxxxxx"' >> ~/.zshrc`
- **Windows**：通过系统设置 → 环境变量添加用户变量

### 5. 使用

```bash
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

---

## 各 Provider 配置

以下为各 Provider 的完整配置，直接复制到 `config.toml` 即可使用。

### 小米 Mimo

```toml
model = "mimo-model-name"
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

环境变量：`export MIMO_API_KEY="你的API Key"`

### 火山引擎（豆包）

```toml
model = "doubao-pro-32k"
model_provider = "volcengine"

[model_providers.volcengine]
name = "Volcengine"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v1"
env_key = "VOLCENGINE_API_KEY"
wire_api = "chat"
```

环境变量：`export VOLCENGINE_API_KEY="你的API Key"`

### Kimi（月之暗面）

```toml
model = "moonshot-v1-32k"
model_provider = "kimi"

[model_providers.kimi]
name = "Kimi"
base_url = "https://api.moonshot.cn/v1"
env_key = "KIMI_API_KEY"
wire_api = "chat"
```

环境变量：`export KIMI_API_KEY="你的API Key"`

### 添加其他 Provider

任何兼容 OpenAI Chat Completions API 的国内平台都可以接入，格式如下：

```toml
model = "你的模型名称"
model_provider = "your-provider"

[model_providers.your-provider]
name = "显示名称"
base_url = "https://平台API地址/v1"
env_key = "你的环境变量名"
wire_api = "chat"
```

---

## 模型元数据配置（可选）

已知的国内模型（`deepseek-*`、`doubao-*`、`kimi-*`、`glm-*`、`qwen-*`、`mimo*` 等）会自动识别，无需额外配置。

如果使用非标准命名的模型，可以手动创建 `~/.codex/custom_models.json`（Windows: `%USERPROFILE%\.codex\custom_models.json`）来提供元数据：

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

关键字段：
- `slug`：必须与 `config.toml` 中的 `model` 一致
- `context_window` / `max_context_window`：模型上下文窗口大小（token 数）
- 其他字段保持默认值即可

### 2. 在 config.toml 中引用

```toml
# Linux / macOS
model_catalog_json = "/home/你的用户名/.codex/custom_models.json"

# Windows
model_catalog_json = "C:\\Users\\你的用户名\\.codex\\custom_models.json"
```

可以同时配置多个模型，`slug` 对应不同模型名称即可在 `config.toml` 中切换。

---

## 常见问题

### Q: `wire_api` 可以大写吗？

不行。serde 反序列化要求小写 `"chat"`，写 `"Chat"` 会报错。

### Q: 支持哪些功能？

- 工具调用（function calling）
- 多轮对话
- 推理内容（thinking / reasoning）
- CLI 自动将内部 Responses API 转换为 Chat Completions API 格式

### Q: 各 Provider 思考模式支持情况？

所有 Provider 的基础对话（非思考模式）均支持。思考模式支持情况：

| Provider | 思考模型示例 | 思考模式 | 说明 |
|----------|-------------|---------|------|
| DeepSeek | deepseek-r1, deepseek-v3 | 支持 | 流式返回 `reasoning_content`，后续请求不回传（已自动处理） |
| 火山引擎/豆包 | doubao-1.5-thinking-pro | 支持 | 同 DeepSeek 格式 |
| Kimi/月之暗面 | kimi-k2 | 支持 | 同 DeepSeek 格式 |
| GLM/智谱 | glm-z1, glm-z1-air | 支持 | 同 DeepSeek 格式 |
| XiaomiMimo | MiMo-7B | 取决于部署方式 | 开源模型，API 格式取决于服务框架 |

所有支持思考模式的 Provider 流式返回时使用 `reasoning_content` 字段传递思考内容。CLI 会自动处理该字段的接收，并在后续请求中正确剥离，无需额外配置。

使用思考模型时，直接在 `config.toml` 中指定模型名称即可：

```toml
model = "deepseek-r1"
model_provider = "deepseek"
```

### Q: API Key 会存入配置文件吗？

不会。API Key 通过环境变量配置，config.toml 中只写环境变量名（`env_key`），不写实际的 Key。

### Q: `personality` 有什么作用？

控制 CLI 的交互风格，支持 `pragmatic`（务实）、`friendly`（友好）等。可选配置，不填则使用默认值。

```toml
personality = "pragmatic"
```

---

# English Version

Codex CLI with support for Chinese LLM providers (Volcengine/Doubao, Kimi, XiaomiMimo, etc.) via Chat Completions API.

## Quick Start

### 1. Get Codex

**Option A: Pre-built binary (recommended)**

Download from [GitHub Releases](https://github.com/xinovate/codex/releases):

| Platform | Package |
|----------|---------|
| Linux x86_64 | `codex-linux-x64.tar.gz` |
| Linux ARM64 | `codex-linux-arm64.tar.gz` |
| Windows x64 | `codex-windows-x64.zip` |
| macOS | Build from source |

**Option B: Build from source**

Requires [Rust](https://rustup.rs/):

```bash
git clone https://github.com/xinovate/codex.git
cd codex/codex-rs
cargo build --release --bin codex
```

### 2. Install

**Linux / macOS:** `sudo cp codex /usr/local/bin/`

**Windows:** Add `codex.exe` directory to system PATH (see [README.md](../README.md)).

### 3. Configure

Create config file (Linux/macOS: `~/.codex/config.toml`, Windows: `%USERPROFILE%\.codex\config.toml`):

```toml
model = "mimo-model-name"
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

| Field | Required | Description |
|-------|----------|-------------|
| `model` | Yes | Model name, e.g. `"doubao-pro-32k"` |
| `model_provider` | Yes | Provider key, matches `[model_providers.xxx]` |
| `name` | Yes | Display name |
| `base_url` | Yes | API endpoint, must end with `/v1` |
| `env_key` | Yes | Environment variable name for API key |
| `wire_api` | Yes | Must be `"chat"` (lowercase) for all Chinese providers |

### 4. Set API Key

```bash
export MIMO_API_KEY="sk-xxxxxxxx"           # Linux/macOS
$env:MIMO_API_KEY = "sk-xxxxxxxx"           # Windows PowerShell
```

For persistence, add to `~/.bashrc`, `~/.zshrc`, or Windows system environment variables.

### 5. Run

```bash
codex              # Interactive mode
codex exec "task"  # One-shot mode
```

## Provider Examples

See the Chinese section above for complete configs for XiaomiMimo, Volcengine, and Kimi.

To add any OpenAI-compatible Chinese provider:

```toml
model = "your-model-name"
model_provider = "your-provider"

[model_providers.your-provider]
name = "Display Name"
base_url = "https://provider-api-endpoint/v1"
env_key = "YOUR_API_KEY_ENV_VAR"
wire_api = "chat"
```

## Notes

- `wire_api = "chat"` is required for all Chinese providers (lowercase only)
- The CLI auto-converts between Responses API (internal) and Chat Completions API (provider)
- Tool calls, multi-turn conversations, and reasoning content are supported
- API keys are stored in environment variables, never in config files
