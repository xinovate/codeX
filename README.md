# Codex CLI - China Provider Fork

A fork of OpenAI's Codex CLI (based on commit `7d72fc8f5`, 2026-04-28) with built-in support for Chinese model providers (Volcengine, Kimi, Doubao, XiaomiMimo, etc.) that use the OpenAI Chat Completions API.

## Install

### Linux

Download pre-built binaries from [GitHub Releases](https://github.com/xinovate/codeX/releases). No Rust installation required.

#### 1. Download and Extract

Choose the package matching your CPU architecture:

```shell
# x86_64 (most Intel/AMD computers)
curl -L https://github.com/xinovate/codeX/releases/download/v0.1.2/codex-linux-x64.tar.gz | tar xz

# ARM64 (Raspberry Pi, Apple Silicon VMs, etc.)
curl -L https://github.com/xinovate/codeX/releases/download/v0.1.2/codex-linux-arm64.tar.gz | tar xz
```

Not sure about your architecture? Run `uname -m` -- `x86_64` means x64, `aarch64` means arm64.

#### 2. Install

Move the binary to a directory in your PATH:

```shell
sudo mv codex /usr/local/bin/
```

Verify installation:

```shell
codex --version
```

#### 3. Configure

Create the config file `~/.codex/config.toml`:

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

#### 4. Set API Key

```shell
# Current session only
export MIMO_API_KEY="your-api-key-here"
```

For permanent setup, add to your shell config:

```shell
# bash users
echo 'export MIMO_API_KEY="your-api-key-here"' >> ~/.bashrc

# zsh users
echo 'export MIMO_API_KEY="your-api-key-here"' >> ~/.zshrc
```

Then reopen your terminal or run `source ~/.bashrc` (or `source ~/.zshrc`).

#### 5. Run

```shell
# Interactive mode
codex

# Single task
codex exec "Write a Hello World in Python"
```

### Windows

Download pre-built binaries from [GitHub Releases](https://github.com/xinovate/codeX/releases). No Rust installation required.

#### 1. Download and Extract

Download `codex-windows-x64.zip`, extract to a directory like `C:\codex`, which gives you `codex.exe`.

#### 2. Add to PATH

Add the directory containing `codex.exe` to your system PATH:

1. Press `Win + S`, search for **"Environment Variables"**, click **"Edit the system environment variables"**
2. Click the **"Environment Variables"** button
3. Under **"User variables"**, select `Path`, click **"Edit"**
4. Click **"New"**, enter `C:\codex`
5. Click **"OK"** to save all dialogs
6. **Reopen** PowerShell or CMD for PATH changes to take effect

Verify installation:

```powershell
codex --version
```

#### 3. Configure

Create the config file `%USERPROFILE%\.codex\config.toml` (i.e., `C:\Users\<username>\.codex\config.toml`):

```toml
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

Or use PowerShell to create it quickly:

```powershell
mkdir "$env:USERPROFILE\.codex" -Force
notepad "$env:USERPROFILE\.codex\config.toml"
```

#### 4. Set API Key

Set the environment variable in PowerShell (current session only):

```powershell
$env:MIMO_API_KEY = "your-api-key-here"
```

For permanent setup (survives reboots), use System Settings:

1. Press `Win + S`, search for **"Environment Variables"**, click **"Edit the system environment variables"**
2. Click the **"Environment Variables"** button
3. Under **"User variables"**, click **"New"**
4. Variable name: `MIMO_API_KEY`, Variable value: your API key
5. Click **"OK"** to save

#### 5. Run

```powershell
# Interactive mode
codex

# Single task
codex exec "Write a Hello World in Python"
```

### macOS

Download pre-built binaries from [GitHub Releases](https://github.com/xinovate/codeX/releases). No Rust installation required.

#### 1. Download and Extract

Choose the package matching your CPU architecture:

```shell
# Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/xinovate/codeX/releases/download/v0.1.2/codex-macos-arm64.tar.gz | tar xz

# Intel
curl -L https://github.com/xinovate/codeX/releases/download/v0.1.2/codex-macos-x64.tar.gz | tar xz
```

Not sure? Run `uname -m`. If it says `arm64`, use arm64; if `x86_64`, use x64.

#### 2. Install

```shell
sudo mv codex /usr/local/bin/
```

Verify installation:

```shell
codex --version
```

#### 3. Configure

Create the config file `~/.codex/config.toml`:

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

#### 4. Set API Key

```shell
# Current session only
export MIMO_API_KEY="your-api-key-here"
```

For permanent setup, add to your shell config (macOS uses zsh by default):

```shell
echo 'export MIMO_API_KEY="your-api-key-here"' >> ~/.zshrc
source ~/.zshrc
```

#### 5. Run

```shell
# Interactive mode
codex

# Single task
codex exec "Write a Hello World in Python"
```

## Configuration

Config file location:

- Linux / macOS: `~/.codex/config.toml`
- Windows: `%USERPROFILE%\.codex\config.toml`

Full configuration example:

```toml
model_provider = "mimo"

[model_providers.mimo]
name = "XiaomiMimo"
base_url = "https://api.xiaomimimo.com/v1"
env_key = "MIMO_API_KEY"
wire_api = "chat"
```

To customize model metadata, create `~/.codex/custom_models.json` (Windows: `%USERPROFILE%\.codex\custom_models.json`) and reference it in your config:

```toml
model_catalog_json = "/home/username/.codex/custom_models.json"
# Windows: model_catalog_json = "C:\\Users\\username\\.codex\\custom_models.json"
```

For detailed configuration, see [`codex-rs/CHINA_PROVIDER.md`](codex-rs/CHINA_PROVIDER.md).

## China Provider Setup

- [**China Provider Guide**](codex-rs/CHINA_PROVIDER.md)

Supported providers: Volcengine, Kimi, Doubao, XiaomiMimo, and other platforms compatible with the OpenAI Chat Completions API.

## Docs

- [**China Provider Guide**](codex-rs/CHINA_PROVIDER.md)
- [**Contributing**](./docs/contributing.md)
- [**Installing & building**](./docs/install.md)

This repository is licensed under the [Apache-2.0 License](LICENSE).
