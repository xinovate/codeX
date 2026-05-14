# CLAUDE.md - Repository Structure

## Repository: xinovate/codex

This is a fork of OpenAI's Codex CLI with China provider support.

## Branch Structure

- **main** - Tracks upstream OpenAI codex directly. DO NOT modify this branch with custom code.
- **custom_provider** - Our custom branch with China provider support. All development happens here.

## How to sync with upstream

> **注意：当前不建议主动同步上游。** 上游已删除 `wire_api = "chat"` 支持，合并会产生大量冲突。
> 除非上游有明确需要的功能（如新的 sandbox 能力），否则保持现状。详见下方 "Upstream Divergence" 章节。

```bash
git fetch upstream
git checkout main
git merge upstream/main
git checkout custom_provider
# 优先 cherry-pick 具体 commit，而非 merge 整个分支
git cherry-pick <commit-hash>
```

Resolve conflicts if any, then push both branches.

## Release Process

Releases are triggered by pushing a tag on the `custom_provider` branch.

**完整发版步骤：**

```bash
# 1. 修改版本号（必须！否则 codex --version 显示旧版本）
#    文件: codex-rs/Cargo.toml 第 112 行
#    version = "0.1.3" -> version = "0.1.4"

# 2. 提交版本号变更
git add codex-rs/Cargo.toml
git commit -m "bump version to 0.1.4"

# 3. 打 tag 并推送
git tag v0.1.4
git push origin custom_provider
git push origin v0.1.4
```

GitHub Actions 自动构建以下平台：
- Linux x64 / arm64
- Windows x64
- macOS arm64 (Apple Silicon)

**版本号位置：** `codex-rs/Cargo.toml` 第 112 行 `version = "x.y.z"`
- 编译时写入二进制，影响 `codex --version` 输出
- `codex update` 命令用此版本与 GitHub releases 对比

**用户更新方式：** `codex update` — 自动检测平台，从 GitHub releases 下载最新版本并替换当前二进制

## Key Files Modified from Upstream

- `codex-rs/Cargo.toml` - Workspace version (line 112, affects `codex --version`)
- `codex-rs/codex-api/src/endpoint/chat_completions.rs` - Chat Completions API conversion (Responses API -> OpenAI Chat format)
- `codex-rs/model-provider/src/china_provider/mod.rs` - China provider runtime (User-Agent header, models manager)
- `codex-rs/model-provider-info/src/lib.rs` - China provider detection, WireApi::Chat variant
- `codex-rs/core/src/client.rs` - ChatCompletionsClient integration
- `codex-rs/tui/src/updates.rs` - Update check URL (points to xinovate/codex GitHub releases)
- `codex-rs/tui/src/update_versions.rs` - Tag prefix parsing (uses `v*` instead of `rust-v*`)
- `codex-rs/tui/src/update_action.rs` - Standalone update points to our releases page
- `codex-rs/cli/src/main.rs` - `codex update` command: auto-downloads from GitHub releases
- `codex-rs/tui/tooltips.txt` - Removed OpenAI-specific tips
- `.github/workflows/release.yml` - Added macOS build targets, restricted to custom_provider branch
- `README.md` - Installation docs for China providers (Chinese)
- `codex-rs/CHINA_PROVIDER.md` - China provider setup guide (Chinese)

## Supported China Providers

| Provider | Base URL | Env Key |
|----------|----------|---------|
| DeepSeek | https://api.deepseek.com | DEEPSEEK_API_KEY |
| Volcengine | https://ark.cn-beijing.volces.com/api/coding/v3 | VOLCENGINE_API_KEY |
| Kimi Code | https://api.kimi.com/coding/v1 | KIMI_CODE_API_KEY |
| Xiaomi Mimo | https://api.xiaomimimo.com/v1 | MIMO_API_KEY |
| 智谱 GLM | https://open.bigmodel.cn/api/coding/paas/v4 | ZHIPU_API_KEY |

## CI

- `cargo-deny`, `Codespell`, `ci` (Prettier + ASCII check) - Run on all pushes
- `rust-ci-full`, `sdk`, `Bazel` - Upstream CI, may fail on fork (missing runners/secrets), ignore these
- `release` - Triggered by `v*.*.*` tags from `custom_provider` branch only (has `if: github.event.base_ref == 'refs/heads/custom_provider'` guard), builds release binaries for Linux/Windows/macOS

## Upstream Divergence (Updated 2026-05-14)

**结论：不要主动合并上游，fork 已是独立项目。**

上游在 `#10157`（2026-02-03）彻底删除了 Chat Completions 支持（-2931 行），`WireApi` enum 只剩 `Responses` 变体。
所有 provider 现在必须实现 `/v1/responses` 端点，而国产模型（DeepSeek、GLM 等）只提供 `/v1/chat/completions`。

本 fork 的核心价值是 **Responses API → Chat Completions 协议转换层**（`chat_completions.rs` + `ChatCompletionsClient`），
这是国产模型接入 Codex 的唯一桥梁。

| 能力 | 上游 main | 本 fork |
|------|-----------|---------|
| `wire_api = "chat"` | 已删除，报错提示改用 responses | 保留并适配 |
| `ChatCompletionsClient` | 不存在 | 完整实现 |
| 国产模型直接配置使用 | 不可能 | DeepSeek/Volcengine/Kimi/Mimo |
| 预编译二进制分发 | 无（npm 源码安装） | GitHub Releases 多平台 |
| `codex update` 自更新 | 无 | 有 |

### Merge 策略

- **不要定期 sync upstream** — 方向已分叉，合并只会带来冲突
- **如需某个上游功能** — cherry-pick 具体 commit，不要 merge 整个分支
- **关注风险** — 上游如果大幅改动 Responses API 请求/响应格式，协议转换层可能需要同步调整
- **main 分支** — 仍然跟踪 upstream，但不主动拉取。仅在确实需要 sync 时操作

### 上游关键时间线

- `2025-05-08` 首次加入 Chat Completions 支持（`#862`）
- `2025-08-05` 完善 streaming chat completions（`#1846`）
- `2026-02-03` **删除 Chat Completions API**（`#10157`），上游要求所有 provider 走 Responses API
- `2026-04-30` 本 fork 的 main 分支所基于的 upstream commit（`8a97f3cf`）

## Upstream Remote

```bash
git remote -v
# origin    https://github.com/xinovate/codex.git (fetch/push)
# upstream  https://github.com/openai/codex.git (fetch/push)
```
