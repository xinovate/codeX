# Codex CLI - China Provider Fork

A fork of OpenAI's Codex CLI (based on commit `7d72fc8f5`, 2026-04-28) with built-in support for Chinese model providers (Volcengine, Kimi, Doubao, XiaomiMimo, etc.) that use the OpenAI Chat Completions API.

## 安装 / Install

### Linux 用户

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载预编译二进制，无需安装 Rust。

#### 1. 下载并解压

根据你的 CPU 架构选择对应的包：

```shell
# x86_64（大多数 Intel/AMD 电脑）
curl -L https://github.com/xinovate/codex/releases/download/v0.1.0/codex-linux-x64.tar.gz | tar xz

# ARM64（如树莓派、Apple Silicon 虚拟机等）
curl -L https://github.com/xinovate/codex/releases/download/v0.1.0/codex-linux-arm64.tar.gz | tar xz
```

不确定架构？运行 `uname -m`，输出 `x86_64` 选 x64，输出 `aarch64` 选 arm64。

#### 2. 安装

将二进制移到 PATH 目录，使其在任意位置可用：

```shell
sudo mv codex /usr/local/bin/
```

验证安装：

```shell
codex --version
```

#### 3. 配置

创建配置文件 `~/.codex/config.toml`：

```shell
mkdir -p ~/.codex
cat > ~/.codex/config.toml << 'EOF'
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
EOF
```

#### 4. 设置 API Key

```shell
# 当前会话生效
export MIMO_API_KEY="你的API Key"
```

如需**永久生效**，写入 shell 配置文件：

```shell
# bash 用户
echo 'export MIMO_API_KEY="你的API Key"' >> ~/.bashrc

# zsh 用户
echo 'export MIMO_API_KEY="你的API Key"' >> ~/.zshrc
```

然后重新打开终端，或执行 `source ~/.bashrc`（或 `source ~/.zshrc`）。

#### 5. 运行

```shell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

### Windows 用户

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载预编译二进制，无需安装 Rust：

#### 1. 下载并解压

下载 `codex-windows-x64.zip`，解压到目录，如 `C:\codex`，得到 `codex.exe`。

#### 2. 添加到 PATH

将 `codex.exe` 所在目录添加到系统 PATH，使其在任意位置可用：

1. 按 `Win + S` 搜索 **"环境变量"**，点击 **"编辑系统环境变量"**
2. 点击 **"环境变量"** 按钮
3. 在 **"用户变量"** 中选中 `Path`，点击 **"编辑"**
4. 点击 **"新建"**，输入 `C:\codex`
5. 点击 **"确定"** 保存所有对话框
6. **重新打开** PowerShell 或 CMD 使 PATH 生效

验证安装：

```powershell
codex --version
```

#### 3. 配置

创建配置文件 `%USERPROFILE%\.codex\config.toml`（即 `C:\Users\你的用户名\.codex\config.toml`）：

```toml
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

也可以用 PowerShell 快速创建：

```powershell
mkdir "$env:USERPROFILE\.codex" -Force
notepad "$env:USERPROFILE\.codex\config.toml"
```

#### 4. 设置 API Key

在 PowerShell 中设置环境变量（当前会话生效）：

```powershell
$env:MIMO_API_KEY = "你的API Key"
```

如需**永久生效**（重启后仍可用），使用系统设置：

1. 按 `Win + S` 搜索 **"环境变量"**，点击 **"编辑系统环境变量"**
2. 点击 **"环境变量"** 按钮
3. 在 **"用户变量"** 中点击 **"新建"**
4. 变量名填 `MIMO_API_KEY`，变量值填你的 API Key
5. 点击 **"确定"** 保存

#### 5. 运行

```powershell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

### macOS 用户

macOS 没有预编译二进制，需要从源码构建。

#### 1. 安装 Rust

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

验证安装：

```shell
rustc --version
cargo --version
```

#### 2. 构建

```shell
git clone https://github.com/xinovate/codex.git
cd codex/codex-rs
cargo build --release --bin codex
```

编译产物在 `target/release/codex`。

#### 3. 安装

```shell
sudo cp target/release/codex /usr/local/bin/
```

验证安装：

```shell
codex --version
```

#### 4. 配置

创建配置文件 `~/.codex/config.toml`：

```shell
mkdir -p ~/.codex
cat > ~/.codex/config.toml << 'EOF'
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
EOF
```

#### 5. 设置 API Key

```shell
# 当前会话生效
export MIMO_API_KEY="你的API Key"
```

如需**永久生效**，写入 shell 配置文件（macOS 默认 zsh）：

```shell
echo 'export MIMO_API_KEY="你的API Key"' >> ~/.zshrc
source ~/.zshrc
```

#### 6. 运行

```shell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

## 配置 / Configuration

配置文件路径：
- Linux / macOS：`~/.codex/config.toml`
- Windows：`%USERPROFILE%\.codex\config.toml`

完整配置示例：

```toml
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

如需自定义模型元数据，可创建 `~/.codex/custom_models.json`（Windows: `%USERPROFILE%\.codex\custom_models.json`）并在配置中引用：

```toml
model_catalog_json = "/home/用户名/.codex/custom_models.json"
# Windows: model_catalog_json = "C:\\Users\\用户名\\.codex\\custom_models.json"
```

详细配置说明见 [`codex-rs/CHINA_PROVIDER.md`](codex-rs/CHINA_PROVIDER.md)。

## 中国提供商设置 / China Provider Setup

- [**中文说明**](codex-rs/CHINA_PROVIDER.md)
- [**English Guide**](codex-rs/CHINA_PROVIDER.md)

支持的提供商：Volcengine (火山引擎)、Kimi (月之暗面)、Doubao (豆包)、XiaomiMimo 等兼容 OpenAI Chat Completions API 的平台。

## Docs

- [**China Provider Guide**](codex-rs/CHINA_PROVIDER.md)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
