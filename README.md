# Codex CLI - China Provider Fork

基于 OpenAI Codex CLI 的 fork（基于 commit `7d72fc8f5`，2026-04-28），内置支持国内大模型服务商（火山引擎/豆包、Kimi Code、小米 Mimo、DeepSeek 等），通过 OpenAI Chat Completions API 接入。

## 安装

### Linux

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载预编译二进制，无需安装 Rust。

#### 1. 下载解压

根据 CPU 架构选择对应包：

```shell
# x86_64（大多数 Intel/AMD 电脑）
curl -L https://github.com/xinovate/codex/releases/download/v0.1.2/codex-linux-x64.tar.gz | tar xz

# ARM64（树莓派、Apple Silicon 虚拟机等）
curl -L https://github.com/xinovate/codex/releases/download/v0.1.2/codex-linux-arm64.tar.gz | tar xz
```

不确定架构？运行 `uname -m`，`x86_64` 是 x64，`aarch64` 是 arm64。

#### 2. 安装

将二进制文件移到 PATH 目录：

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
# 当前会话
export MIMO_API_KEY="你的API Key"
```

永久生效，添加到 shell 配置：

```shell
# bash 用户
echo 'export MIMO_API_KEY="你的API Key"' >> ~/.bashrc

# zsh 用户
echo 'export MIMO_API_KEY="你的API Key"' >> ~/.zshrc
```

然后重新打开终端，或执行 `source ~/.bashrc`（或 `source ~/.zshrc`）。

#### 5. 使用

```shell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

### Windows

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载预编译二进制，无需安装 Rust。

#### 1. 下载解压

下载 `codex-windows-x64.zip`，解压到目录如 `C:\codex`，得到 `codex.exe`。

#### 2. 添加到 PATH

将 `codex.exe` 所在目录添加到系统 PATH：

1. 按 `Win + S`，搜索 **"环境变量"**，点击 **"编辑系统环境变量"**
2. 点击 **"环境变量"** 按钮
3. 在 **"用户变量"** 中，选择 `Path`，点击 **"编辑"**
4. 点击 **"新建"**，输入 `C:\codex`
5. 点击 **"确定"** 保存所有对话框
6. **重新打开** PowerShell 或 CMD 使 PATH 生效

验证安装：

```powershell
codex --version
```

#### 3. 配置

创建配置文件 `%USERPROFILE%\.codex\config.toml`（即 `C:\Users\<用户名>\.codex\config.toml`）：

```toml
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

或用 PowerShell 快速创建：

```powershell
mkdir "$env:USERPROFILE\.codex" -Force
notepad "$env:USERPROFILE\.codex\config.toml"
```

#### 4. 设置 API Key

PowerShell 中设置环境变量（当前会话）：

```powershell
$env:MIMO_API_KEY = "你的API Key"
```

永久生效（重启后保留），使用系统设置：

1. 按 `Win + S`，搜索 **"环境变量"**，点击 **"编辑系统环境变量"**
2. 点击 **"环境变量"** 按钮
3. 在 **"用户变量"** 中，点击 **"新建"**
4. 变量名：`MIMO_API_KEY`，变量值：你的 API Key
5. 点击 **"确定"** 保存

#### 5. 使用

```powershell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

### macOS

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载预编译二进制，无需安装 Rust。

#### 1. 下载解压

根据 CPU 架构选择对应包：

```shell
# Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/xinovate/codex/releases/download/v0.1.2/codex-macos-arm64.tar.gz | tar xz

# Intel
curl -L https://github.com/xinovate/codex/releases/download/v0.1.2/codex-macos-x64.tar.gz | tar xz
```

不确定？运行 `uname -m`，`arm64` 用 arm64，`x86_64` 用 x64。

#### 2. 安装

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
# 当前会话
export MIMO_API_KEY="你的API Key"
```

永久生效，添加到 shell 配置（macOS 默认 zsh）：

```shell
echo 'export MIMO_API_KEY="你的API Key"' >> ~/.zshrc
source ~/.zshrc
```

#### 5. 使用

```shell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

## 配置说明

配置文件位置：

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

自定义模型元数据：创建 `~/.codex/custom_models.json`（Windows: `%USERPROFILE%\.codex\custom_models.json`），在配置中引用：

```toml
model_catalog_json = "/home/用户名/.codex/custom_models.json"
# Windows: model_catalog_json = "C:\\Users\\用户名\\.codex\\custom_models.json"
```

详细配置见 [`codex-rs/CHINA_PROVIDER.md`](codex-rs/CHINA_PROVIDER.md)。

## 国内服务商配置

- [**国内服务商配置指南**](codex-rs/CHINA_PROVIDER.md)

支持的服务商：火山引擎/豆包、Kimi Code、DeepSeek、小米 Mimo，以及任何兼容 OpenAI Chat Completions API 的国内平台。

## 文档

- [**国内服务商配置指南**](codex-rs/CHINA_PROVIDER.md)
- [**贡献指南**](./docs/contributing.md)
- [**安装与构建**](./docs/install.md)

本项目基于 [Apache-2.0 License](LICENSE) 开源。
