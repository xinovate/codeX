# Codex CLI - China Provider Fork

A fork of OpenAI's Codex CLI with built-in support for Chinese model providers (Volcengine, Kimi, Doubao, XiaomiMimo, etc.) that use the OpenAI Chat Completions API.

## 安装 / Install

### 方式一：下载预编译二进制（推荐）

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载对应平台的二进制文件：

| 平台 | 文件 |
|------|------|
| Linux (x86_64) | `codex-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `codex-aarch64-unknown-linux-gnu.tar.gz` |

```shell
# 示例：Linux x86_64
tar xzf codex-x86_64-unknown-linux-gnu.tar.gz
sudo mv codex /usr/local/bin/
```

> macOS 用户请使用方式二从源码构建。

### 方式二：从源码构建

需要先安装 [Rust](https://rustup.rs/)：

```shell
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 构建
git clone https://github.com/xinovate/codex.git
cd codex/codex-rs
cargo build --release --bin codex

# 安装
sudo cp target/release/codex /usr/local/bin/
```

## 配置 / Configuration

创建配置文件 `~/.codex/config.toml`：

```toml
[model_provider]
name = "mimo"
base_url = "https://api.mimo.com/v1"
wire_api = "Chat"
```

如需自定义模型元数据，可创建 `~/.codex/custom_models.json` 并在配置中引用：

```toml
model_catalog_json = "/path/to/custom_models.json"
```

详细配置说明见 [`codex-rs/CHINA_PROVIDER.md`](codex-rs/CHINA_PROVIDER.md)。

## 运行 / Run

```shell
codex
```

## 中国提供商设置 / China Provider Setup

- [**中文说明**](codex-rs/CHINA_PROVIDER.md)
- [**English Guide**](codex-rs/CHINA_PROVIDER.md)

支持的提供商：Volcengine (火山引擎)、Kimi (月之暗面)、Doubao (豆包)、XiaomiMimo 等兼容 OpenAI Chat Completions API 的平台。

## Docs

- [**China Provider Guide**](codex-rs/CHINA_PROVIDER.md)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
