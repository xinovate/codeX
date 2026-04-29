# Codex CLI - China Provider Fork

A fork of OpenAI's Codex CLI with built-in support for Chinese model providers (Volcengine, Kimi, Doubao, XiaomiMimo, etc.) that use the OpenAI Chat Completions API.

## 环境准备 / Prerequisites

### 1. 安装 Git

```shell
# macOS (已预装，或通过 Homebrew)
brew install git

# Ubuntu/Debian
sudo apt install git

# Windows
# 下载安装 https://git-scm.com/download/win
```

### 2. 安装 Rust 工具链

Codex CLI 是 Rust 项目，需要安装 Rust 编译器和 Cargo 包管理器。

```shell
# macOS / Linux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Windows
# 下载安装 https://win.rustup.rs/
```

验证安装：

```shell
rustc --version
cargo --version
```

### 3. 系统依赖 (仅 Linux)

```shell
# Ubuntu/Debian
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# CentOS/RHEL/Fedora
sudo yum install -y gcc openssl-devel pkgconfig
```

macOS 和 Windows 通常不需要额外依赖。

## 构建 / Build

```shell
git clone https://github.com/xinovate/codex.git
cd codex/codex-rs
cargo build --release --bin codex
```

构建完成后，二进制文件在 `target/release/codex`。

### 配置 PATH

```shell
# macOS / Linux - 添加到 ~/.bashrc 或 ~/.zshrc
export PATH="$HOME/codex/codex-rs/target/release:$PATH"

# 或者复制到系统目录
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
