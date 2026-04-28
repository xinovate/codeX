# Codex 国内平台适配 - 编译测试指南

## 修改文件清单

### 新增文件
| 文件 | 说明 |
|------|------|
| `codex-api/src/endpoint/chat_completions.rs` | ChatCompletionsClient - 协议转换客户端 (417行) |
| `model-provider/src/china_provider/mod.rs` | ChinaModelProvider - 国内平台 Provider (64行) |

### 修改文件
| 文件 | 修改内容 |
|------|----------|
| `model-provider-info/src/lib.rs` | 恢复 `WireApi::Chat`，添加 `is_china_provider()` |
| `model-provider/src/lib.rs` | 注册 `china_provider` 模块 |
| `model-provider/src/provider.rs` | 注册 `ChinaModelProvider` 工厂，自动设置 `wire_api = Chat` |
| `codex-api/src/endpoint/mod.rs` | 导出 `ChatCompletionsClient` |
| `codex-api/src/lib.rs` | 导出 `ChatCompletionsClient` |
| `core/src/client.rs` | 添加 `WireApi::Chat` 分发和 `stream_chat_completions_api` |
| `tui/src/status/card.rs` | 支持 `WireApi::Chat` 显示 reasoning effort |
| `utils/sandbox-summary/src/config_summary.rs` | 支持 `WireApi::Chat` |
| `exec/src/event_processor_with_human_output.rs` | 支持 `WireApi::Chat` |

## 编译步骤

### 前提条件
- Rust 工具链 (rustc + cargo)
- 或 Bazel 构建系统

## 使用 Docker 编译（推荐）

如果本地没有 Rust 环境，可以使用 Docker：

```bash
# 1. 进入项目目录
cd /Users/wangxin/agent-mastery-sources/codex

# 2. 运行编译脚本
./build.sh
```

### 手动 Docker 步骤

```bash
# 构建镜像
docker build -t codex-china-build .

# 运行编译
docker run --rm -it -v "$(pwd):/codex" -w /codex/codex-rs codex-china-build bash

# 在容器内
cargo check
cargo build
cargo test
```

## 编译方式

### 方式 1：使用 Docker（推荐）

```bash
cd /Users/wangxin/agent-mastery-sources/codex
./build.sh
```

### 方式 2：本地安装 Rust

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 编译
cd /Users/wangxin/agent-mastery-sources/codex/codex-rs
cargo check
cargo build
```

### 方式 3：使用 GitHub Actions

已配置 `.github/workflows/build.yml`，推送代码后自动编译测试。

### 方式 4：使用 Bazel

```bash
cd /Users/wangxin/agent-mastery-sources/codex
bazel build //codex-rs/cli:codex
```

## ⚠️ 当前环境限制

当前环境（王鑫的 MacBook Pro）无法直接编译：
- ❌ Rust 安装被系统限制（SIGKILL）
- ❌ Docker 需要 sudo 权限
- ❌ Bazel 未安装

**建议**：在以下环境编译测试：
1. 个人开发机（有管理员权限）
2. GitHub Actions（已配置 workflow）
3. 远程服务器（有 Rust/Docker 环境）

```bash
cd /Users/wangxin/agent-mastery-sources/codex/codex-rs

# 检查语法
cargo check

# 编译整个工作区
cargo build

# 运行单元测试
cargo test --package codex-api -- chat_completions
cargo test --package model-provider -- china_provider
cargo test --package model-provider-info -- china_provider

# 运行核心客户端测试
cargo test --package codex-core -- client
```

### 使用 Bazel 编译

```bash
cd /Users/wangxin/agent-mastery-sources/codex

# 编译 CLI
bazel build //codex-rs/cli:codex

# 运行测试
bazel test //codex-rs/codex-api/...
bazel test //codex-rs/model-provider/...
bazel test //codex-rs/model-provider-info/...
```

## 配置使用

### 1. 配置 provider

编辑 `~/.codex/config.toml`：

```toml
[model_providers.volcengine]
name = "Volcengine"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v3"
env_key = "VOLCENGINE_API_KEY"
wire_api = "chat"

[model_providers.kimi]
name = "Kimi"
base_url = "https://api.moonshot.cn/v1"
env_key = "KIMI_API_KEY"
wire_api = "chat"
```

### 2. 设置 API Key

```bash
export VOLCENGINE_API_KEY="your-api-key"
export KIMI_API_KEY="your-api-key"
```

### 3. 运行 Codex

```bash
codex --provider volcengine --model doubao-seed-2.0-pro
```

## 测试验证

### 单元测试

```bash
# 测试请求转换
cargo test --package codex-api chat_completions::tests::converts_request_body

# 测试响应转换
cargo test --package model-provider china_provider::response::tests

# 测试 provider 识别
cargo test --package model-provider-info is_china_provider
```

### 集成测试

```bash
# 使用 Python 验证脚本
python3 validate_conversion.py

# 手动测试
codex --provider volcengine
> hello
# 观察是否正常响应
```

## 故障排查

### 编译错误

| 错误 | 解决 |
|------|------|
| `WireApi::Chat` 未定义 | 确认 `model-provider-info/src/lib.rs` 已修改 |
| `ChatCompletionsClient` 未找到 | 确认 `codex-api/src/endpoint/mod.rs` 和 `lib.rs` 已导出 |
| `ContentPart` 未找到 | 已修复为 `ContentItem` |
| `end_turn` 字段缺失 | 已添加 `end_turn: None` |

### 运行时错误

| 错误 | 解决 |
|------|------|
| 404 Not Found | 检查 `base_url` 是否正确，是否包含 `/v1` |
| 401 Unauthorized | 检查 `env_key` 对应的环境变量是否设置 |
| 响应格式错误 | 检查 provider 是否返回标准 Chat API 格式 |

## 注意事项

1. **WebSocket 不支持**：国内平台通常不支持 WebSocket，Codex 会自动回退到 HTTP
2. **Tool Calls**：当前实现支持基本的 tool_calls 转换，但复杂场景可能需要额外测试
3. **Reasoning Content**：火山引擎的 `reasoning_content` 已支持，会转换为 `reasoning_text.delta`
4. **Stream Idle Timeout**：默认 5 分钟，可在配置中调整

## 回滚

如需回滚修改：

```bash
cd /Users/wangxin/agent-mastery-sources/codex
git checkout -- codex-rs/
```

或手动删除新增文件并恢复修改文件。

---

**文档版本**: v1.0
**创建日期**: 2026-04-28
**维护者**: V妹
