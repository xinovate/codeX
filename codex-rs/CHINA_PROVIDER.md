# Codex CLI - China Provider

> **与小米 Mimo 联合共建** - 小米大模型 core 团队为本项目提供 API 资源，共同推进国内 AI 编程工具生态

## 为什么需要这个项目？

OpenAI Codex CLI 上游在 2026 年初删除了 `wire_api = "chat"`（Chat Completions API）支持，**要求所有 provider 必须实现 OpenAI Responses API**（`/v1/responses`）。

然而国内大模型（DeepSeek、智谱 GLM、Kimi、小米 Mimo 等）普遍只提供 **OpenAI Chat Completions API**（`/v1/chat/completions`），不支持 Responses API。这意味着上游 Codex CLI **无法直接使用**任何国产大模型。

本项目实现了 **Responses API → Chat Completions 协议转换层**，并提供预编译二进制和自更新机制，让国产模型开箱即用。

---

## 已验证的服务商

| 服务商 | 模型示例 | 接入方式 | 状态 |
|--------|---------|---------|------|
| 小米 Mimo (TokenPlan) | `mimo-v2.5-pro` | 套餐模式 | ✅ 多轮/工具/分析 全部通过 |
| 小米 Mimo (API) | `mimo-v2.5-pro` | 按量计费 | ✅ 多轮/工具/分析 全部通过 |
| DeepSeek | `deepseek-v4-flash`、`deepseek-v4-pro` | API Key | ✅ 多轮/工具/分析 全部通过 |
| 智谱 GLM | `glm-5.1`、`glm-4.7-flash` | Coding Plan | ✅ 多轮/工具/分析 全部通过 |
| Kimi Code | `kimi-k2.6` | 订阅制 | ✅ 多轮/工具/分析 全部通过 |
| 火山引擎/豆包 | `doubao-seed-2.0-code` | Coding Plan | ✅ 多轮/工具/分析 全部通过 |

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
| macOS ARM64 (Apple Silicon) | `codex-macos-arm64.tar.gz` |

**方式二：从源码构建（需要最新代码时）**

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

以智谱 GLM 为例，完整配置如下：

```toml
model = "glm-4.7-flash"
model_provider = "zhipu"

[model_providers.zhipu]
name = "ZhiPuGLM"
base_url = "https://open.bigmodel.cn/api/coding/paas/v4"
env_key = "ZHIPU_API_KEY"
wire_api = "chat"
```

各字段说明：

| 字段 | 必填 | 说明 |
|------|------|------|
| `model` | 是 | 模型名称，如 `"glm-4.7-flash"`、`"deepseek-v4-flash"` |
| `model_provider` | 是 | Provider 标识，对应 `[model_providers.xxx]` 中的 `xxx` |
| `name` | 是 | Provider 显示名称 |
| `base_url` | 是 | API 地址，如 `https://open.bigmodel.cn/api/coding/paas/v4` |
| `env_key` | 是 | API Key 对应的环境变量名 |
| `wire_api` | 是 | 固定填 `"chat"`（小写），国内 provider 必须设置 |

### 4. 设置 API Key

将 `env_key` 指定的环境变量设置为你的 API Key：

```bash
# Linux / macOS（当前会话）
export ZHIPU_API_KEY="你的API Key"

# Windows PowerShell（当前会话）
$env:ZHIPU_API_KEY = "你的API Key"
```

**永久生效：**

- **bash**：`echo 'export ZHIPU_API_KEY="你的API Key"' >> ~/.bashrc`
- **zsh**（macOS 默认）：`echo 'export ZHIPU_API_KEY="你的API Key"' >> ~/.zshrc`
- **Windows**：通过系统设置 → 环境变量添加用户变量

### 5. 使用

```bash
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

### 6. 更新

安装后可使用命令自动更新到最新版本：

```bash
codex update
```

该命令会自动检测平台，从 GitHub Releases 下载最新版本并替换当前二进制。

---

## 各 Provider 配置

以下为各 Provider 的完整配置，直接复制到 `config.toml` 即可使用。

### 小米 Mimo (TokenPlan)

套餐模式，推荐使用。

```toml
model = "mimo-v2.5-pro"
model_provider = "mimo-tp"

[model_providers.mimo-tp]
name = "XiaomiMimoTokenPlan"
base_url = "https://token-plan-cn.xiaomimimo.com/v1"
env_key = "MIMO_TP_API_KEY"
wire_api = "chat"
```

环境变量：`export MIMO_TP_API_KEY="你的TokenPlan Key"`

### 小米 Mimo (API)

按量计费模式。

```toml
model = "mimo-v2.5-pro"
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

环境变量：`export MIMO_API_KEY="你的API Key"`

### DeepSeek

```toml
model = "deepseek-v4-flash"
model_provider = "deepseek"

[model_providers.deepseek]
name = "DeepSeek"
base_url = "https://api.deepseek.com"
env_key = "DEEPSEEK_API_KEY"
wire_api = "chat"
```

环境变量：`export DEEPSEEK_API_KEY="你的API Key"`

可用模型：`deepseek-v4-flash`（非思考）、`deepseek-v4-pro`（思考）。旧名称 `deepseek-chat`、`deepseek-reasoner` 将于 2026/07/24 弃用。

> **注意**：DeepSeek 的 `deepseek-v4-flash` 默认开启思考模式，Codex 会自动禁用（发送 `thinking: {type: "disabled"}`）。

### Kimi Code

```toml
model = "kimi-k2.6"
model_provider = "kimi"

[model_providers.kimi]
name = "KimiCode"
base_url = "https://api.kimi.com/coding/v1"
env_key = "KIMI_CODE_API_KEY"
wire_api = "chat"
```

环境变量：`export KIMI_CODE_API_KEY="你的API Key"`

> **注意**：Kimi Coding API 需要特定的 User-Agent 标识，Codex 会自动添加 `User-Agent: claude-code/0.1`。需要已订阅 Kimi 会员并开通 Kimi Code 权益。

### 火山引擎（豆包）

```toml
model = "doubao-seed-2.0-code"
model_provider = "volcengine"

[model_providers.volcengine]
name = "Volcengine"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v3"
env_key = "VOLCENGINE_API_KEY"
wire_api = "chat"
```

环境变量：`export VOLCENGINE_API_KEY="你的API Key"`

> **注意**：火山引擎 Coding Plan 使用 `/api/coding/v3`（OpenAI 兼容），不要用 `/api/v3`（会产生额外费用）。

### 智谱 GLM

```toml
model = "glm-5.1"
model_provider = "zhipu"

[model_providers.zhipu]
name = "ZhiPuGLM"
base_url = "https://open.bigmodel.cn/api/coding/paas/v4"
env_key = "ZHIPU_API_KEY"
wire_api = "chat"
```

环境变量：`export ZHIPU_API_KEY="你的API Key"`

可用模型：`glm-5.1`（旗舰）、`glm-4.7-flash`（快速）、`glm-z1`（思考）、`glm-z1-air`（轻量思考）。

> **注意**：智谱 Coding Plan 必须使用专属端点 `api/coding/paas/v4`，通用端点 `api/paas/v4` 不适用。`reasoning_content` 和 `thinking.type` 参数与 DeepSeek 格式一致。

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

已知的国内模型（`deepseek-*`、`doubao-*`、`kimi-*`、`glm-*`、`mimo*` 等）会自动识别，无需额外配置。

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
- MCP 工具（Web 搜索、网页阅读等第三方工具服务器）
- 多轮对话
- 推理内容（thinking / reasoning）
- CLI 自动将内部 Responses API 转换为 Chat Completions API 格式

### Q: 各 Provider 思考模式支持情况？

所有 Provider 的基础对话（非思考模式）均支持。Codex 默认禁用思考模式（发送 `thinking: {type: "disabled"}`），以避免 `reasoning_content` 回传问题。

| Provider | 思考模型示例 | 思考模式 | 说明 |
|----------|-------------|---------|------|
| XiaomiMimo (TokenPlan/API) | `mimo-v2.5-pro` | 不支持 | Codex 未开启思考模式 |
| DeepSeek | `deepseek-v4-pro` | 支持但默认禁用 | `deepseek-v4-flash` 默认开启思考，Codex 自动禁用 |
| Kimi Code | `kimi-k2` | 支持但默认禁用 | 同 DeepSeek 格式 |
| 火山引擎/豆包 | `doubao-1.5-thinking-pro` | 支持但默认禁用 | 同 DeepSeek 格式 |
| 智谱 GLM | `glm-z1`、`glm-z1-air` | 支持但默认禁用 | 同 DeepSeek 格式，使用 `thinking.type` 参数 |

> **注意**：如果需要使用思考模式，需要修改代码中的 `thinking: {type: "disabled"}` 为 `"enabled"`，并确保正确处理 `reasoning_content` 的回传。

### Q: API Key 会存入配置文件吗？

不会。API Key 通过环境变量配置，config.toml 中只写环境变量名（`env_key`），不写实际的 Key。

### Q: `personality` 有什么作用？

控制 CLI 的交互风格，支持 `pragmatic`（务实）、`friendly`（友好）等。可选配置，不填则使用默认值。

```toml
personality = "pragmatic"
```

### Q: 如何配置 MCP 服务器？

在 `config.toml` 中添加 `[mcp_servers]` 段即可。MCP 工具会在对话中自动注入给模型使用。

**HTTP transport（推荐）：**

```toml
[mcp_servers.web-search]
url = "https://example.com/mcp"
bearer_token_env_var = "YOUR_API_KEY_ENV"
```

**STDIO transport：**

```toml
[mcp_servers.my-tool]
command = "my-mcp-server"
args = ["--port", "8080"]
env_vars = ["MY_API_KEY"]
```

**工具审批配置（可选）：**

```toml
[mcp_servers.web-search.tools.search]
approval_mode = "approve"   # 自动批准，无需每次确认
```

示例 — 智谱 GLM + MCP 搜索和网页阅读完整配置：

```toml
model = "glm-5.1"
model_provider = "zhipu"

[model_providers.zhipu]
name = "ZhiPuGLM"
base_url = "https://open.bigmodel.cn/api/coding/paas/v4"
env_key = "ZHIPU_API_KEY"
wire_api = "chat"

[mcp_servers.web-search-prime]
url = "https://open.bigmodel.cn/api/mcp/web_search_prime/mcp"
bearer_token_env_var = "ZHIPU_API_KEY"

[mcp_servers.web-search-prime.tools.web_search_prime]
approval_mode = "approve"

[mcp_servers.web-reader]
url = "https://open.bigmodel.cn/api/mcp/web_reader/mcp"
bearer_token_env_var = "ZHIPU_API_KEY"
```
