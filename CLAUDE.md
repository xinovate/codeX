# CLAUDE.md - Repository Structure

## Repository: xinovate/codex

This is a fork of OpenAI's Codex CLI with China provider support.

## Branch Structure

- **main** - Tracks upstream OpenAI codex directly. DO NOT modify this branch with custom code.
- **custom_provider** - Our custom branch with China provider support. All development happens here.

## How to sync with upstream

```bash
git fetch upstream
git checkout main
git merge upstream/main
git checkout custom_provider
git merge main
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
- macOS x64 (Intel)

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

## CI

- `cargo-deny`, `Codespell`, `ci` (Prettier + ASCII check) - Run on all pushes
- `rust-ci-full`, `sdk`, `Bazel` - Upstream CI, may fail on fork (missing runners/secrets), ignore these
- `release` - Triggered by `v*.*.*` tags from `custom_provider` branch only (has `if: github.event.base_ref == 'refs/heads/custom_provider'` guard), builds release binaries for Linux/Windows/macOS

## Upstream Remote

```bash
git remote -v
# origin    https://github.com/xinovate/codex.git (fetch/push)
# upstream  https://github.com/openai/codex.git (fetch/push)
```
