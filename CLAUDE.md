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

Releases are triggered by pushing a tag on the `custom_provider` branch:

```bash
git checkout custom_provider
git tag v0.1.x
git push origin v0.1.x
```

The release workflow (`.github/workflows/release.yml`) builds binaries for:
- Linux x64 / arm64
- Windows x64
- macOS arm64 (Apple Silicon)
- macOS x64 (Intel)

## Key Files Modified from Upstream

- `codex-rs/codex-api/src/endpoint/chat_completions.rs` - Chat Completions API conversion (Responses API -> OpenAI Chat format)
- `codex-rs/model-provider/src/china_provider/mod.rs` - China provider runtime (User-Agent header, models manager)
- `codex-rs/model-provider-info/src/lib.rs` - China provider detection, WireApi::Chat variant
- `codex-rs/core/src/client.rs` - ChatCompletionsClient integration
- `.github/workflows/release.yml` - Added macOS build targets
- `README.md` - Installation docs for China providers
- `codex-rs/CHINA_PROVIDER.md` - China provider setup guide

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
