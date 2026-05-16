# Codex CLI - China Provider Fork

> **与小米 Mimo 联合共建** - 小米大模型 core 团队为本项目提供 API 资源，共同推进国内 AI 编程工具生态

基于 OpenAI Codex CLI 的 fork，让国内大模型服务商（小米 Mimo、DeepSeek、Kimi Code、火山引擎/豆包 等）可以直接接入 Codex，开箱即用。

## 为什么需要这个项目？

OpenAI Codex CLI 上游在 2026 年初删除了 `wire_api = "chat"`（Chat Completions API）支持，**要求所有 provider 必须实现 OpenAI Responses API**（`/v1/responses`）。

然而国内大模型（DeepSeek、智谱 GLM、Kimi、小米 Mimo 等）普遍只提供 **OpenAI Chat Completions API**（`/v1/chat/completions`），不支持 Responses API。

这意味着：
- 上游 Codex CLI **无法直接使用**任何国产大模型
- 即使通过 `config.toml` 配置自定义 provider，`wire_api = "chat"` 也会直接报错
- 社区的替代方案（One-API 代理、Bifrost 网关等）只是协议转发，无法解决 Codex 对 Responses API 特有功能（structured outputs、tool calls 并行、multi-turn context）的依赖

**本项目的核心价值：**

1. **协议转换层** — 完整实现 Responses API → Chat Completions 双向转换，透明处理 tool calls、reasoning content、multi-turn 上下文等差异
2. **MCP 工具支持** — 通过 MCP 服务器扩展能力（Web 搜索、网页阅读等），国内 Provider 下可正常调用
3. **预编译分发** — GitHub Releases 提供开箱即用的预编译二进制（Linux/Windows/macOS），无需安装 Rust 或 Node.js
4. **自更新机制** — `codex update` 一键升级，自动检测平台下载最新版本
5. **中文文档** — 面向国内开发者的完整安装和配置指南

**简单来说：上游关了门，我们建了桥。**

## 安装

### Linux

从 [GitHub Releases](https://github.com/xinovate/codex/releases) 下载预编译二进制，无需安装 Rust。

#### 1. 下载解压

根据 CPU 架构选择对应包：

```shell
# x86_64（大多数 Intel/AMD 电脑）
curl -L https://github.com/xinovate/codex/releases/latest/download/codex-linux-x64.tar.gz | tar xz

# ARM64（树莓派、Apple Silicon 虚拟机等）
curl -L https://github.com/xinovate/codex/releases/latest/download/codex-linux-arm64.tar.gz | tar xz
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
model_provider = "mimo-tp"

[model_providers.mimo-tp]
name = "XiaomiMimoTokenPlan"
base_url = "https://token-plan-cn.xiaomimimo.com/v1"
env_key = "MIMO_TP_API_KEY"
wire_api = "chat"
EOF
```

#### 4. 设置 API Key

```shell
# 当前会话
export MIMO_TP_API_KEY="你的API Key"
```

永久生效，添加到 shell 配置：

```shell
# bash 用户
echo 'export MIMO_TP_API_KEY="你的API Key"' >> ~/.bashrc

# zsh 用户
echo 'export MIMO_TP_API_KEY="你的API Key"' >> ~/.zshrc
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
model_provider = "mimo-tp"

[model_providers.mimo-tp]
name = "XiaomiMimoTokenPlan"
base_url = "https://token-plan-cn.xiaomimimo.com/v1"
env_key = "MIMO_TP_API_KEY"
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
$env:MIMO_TP_API_KEY = "你的API Key"
```

永久生效（重启后保留），使用系统设置：

1. 按 `Win + S`，搜索 **"环境变量"**，点击 **"编辑系统环境变量"**
2. 点击 **"环境变量"** 按钮
3. 在 **"用户变量"** 中，点击 **"新建"**
4. 变量名：`MIMO_TP_API_KEY`，变量值：你的 API Key
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

仅支持 Apple Silicon (M1/M2/M3/M4)：

```shell
curl -L https://github.com/xinovate/codex/releases/latest/download/codex-macos-arm64.tar.gz | tar xz
```

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
model_provider = "mimo-tp"

[model_providers.mimo-tp]
name = "XiaomiMimoTokenPlan"
base_url = "https://token-plan-cn.xiaomimimo.com/v1"
env_key = "MIMO_TP_API_KEY"
wire_api = "chat"
EOF
```

#### 4. 设置 API Key

```shell
# 当前会话
export MIMO_TP_API_KEY="你的API Key"
```

永久生效，添加到 shell 配置（macOS 默认 zsh）：

```shell
echo 'export MIMO_TP_API_KEY="你的API Key"' >> ~/.zshrc
source ~/.zshrc
```

#### 5. 使用

```shell
# 交互模式
codex

# 单次任务
codex exec "用Python写一个Hello World"
```

## 更新

安装后可使用命令自动更新到最新版本：

```shell
codex update
```

该命令会自动检测平台，从 GitHub Releases 下载最新版本并替换当前二进制。

TUI 启动时也会自动检查新版本（每 20 小时一次），有新版本时会提示更新。

## 配置说明

配置文件位置：

- Linux / macOS：`~/.codex/config.toml`
- Windows：`%USERPROFILE%\.codex\config.toml`

完整配置示例：

```toml
model_provider = "mimo-tp"

[model_providers.mimo-tp]
name = "XiaomiMimoTokenPlan"
base_url = "https://token-plan-cn.xiaomimimo.com/v1"
env_key = "MIMO_TP_API_KEY"
wire_api = "chat"
```

自定义模型元数据：创建 `~/.codex/custom_models.json`（Windows: `%USERPROFILE%\.codex\custom_models.json`），在配置中引用：

```toml
model_catalog_json = "/home/用户名/.codex/custom_models.json"
# Windows: model_catalog_json = "C:\\Users\\用户名\\.codex\\custom_models.json"
```

详细配置见 [`codex-rs/CHINA_PROVIDER.md`](codex-rs/CHINA_PROVIDER.md)。

### MCP 服务器配置

在 `config.toml` 中添加 `[mcp_servers]` 段即可启用第三方工具（Web 搜索、网页阅读等），详见 [CHINA_PROVIDER.md 的 MCP 配置章节](codex-rs/CHINA_PROVIDER.md)。

### 图片识别配置

通过 `[image_analysis]` 段配置 MCP 图片识别工具，粘贴图片时自动识别内容。详见 [CHINA_PROVIDER.md 的图片识别章节](codex-rs/CHINA_PROVIDER.md)。

```toml
[image_analysis]
mcp_server = "zai-mcp-server"
tool_name = "image_analysis"
```

## 国内服务商配置

- [**国内服务商配置指南**](codex-rs/CHINA_PROVIDER.md)

支持的服务商：小米 Mimo（TokenPlan / API）、DeepSeek、Kimi Code、火山引擎/豆包、智谱 GLM，以及任何兼容 OpenAI Chat Completions API 的国内平台。

## 文档

- [**国内服务商配置指南**](codex-rs/CHINA_PROVIDER.md)
- [**贡献指南**](./docs/contributing.md)
- [**安装与构建**](./docs/install.md)

本项目基于 [Apache-2.0 License](LICENSE) 开源。
