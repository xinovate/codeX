# Codex 国内平台适配 - Docker 编译环境

FROM rust:1.85-slim-bookworm

# 安装系统依赖
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# 安装 Bazelisk (Bazel 版本管理器)
RUN curl -Lo /usr/local/bin/bazel https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64 \
    && chmod +x /usr/local/bin/bazel

# 设置工作目录
WORKDIR /codex

# 复制项目文件
COPY . .

# 使用 Cargo 编译
RUN cargo check 2>&1 | head -100 || true

# 或使用 Bazel 编译
# RUN bazel build //codex-rs/cli:codex

CMD ["/bin/bash"]
