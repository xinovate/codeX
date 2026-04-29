# Codex CLI - China Provider Fork

A fork of OpenAI's Codex CLI with built-in support for Chinese model providers (Volcengine, Kimi, Doubao, XiaomiMimo, etc.) that use the OpenAI Chat Completions API.

## Installing

Build from source:

```shell
git clone https://github.com/xinovate/codex.git
cd codex/codex-rs
cargo build --release --bin codex
```

The binary will be at `target/release/codex`. Add it to your `PATH` or copy it somewhere convenient.

## China Provider Setup

See [`CHINA_PROVIDER.md`](CHINA_PROVIDER.md) for detailed setup instructions (in Chinese and English).

Quick start:
1. Build the binary (see above)
2. Configure `~/.codex/config.toml` with your provider settings
3. Run `codex`

## Docs

- [**China Provider Guide**](CHINA_PROVIDER.md)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
