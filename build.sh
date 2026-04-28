#!/bin/bash
# Docker 编译脚本

set -e

echo "=== Codex 国内平台适配 - Docker 编译 ==="

# 检查 Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker 未安装"
    echo "请先安装 Docker: https://docs.docker.com/get-docker/"
    exit 1
fi

echo "✅ Docker 已安装"

# 检查 Docker 权限
if ! docker info > /dev/null 2>&1; then
    echo "⚠️ Docker 需要权限，尝试使用 sudo..."
    DOCKER_CMD="sudo docker"
else
    DOCKER_CMD="docker"
fi

# 构建镜像
echo ""
echo "=== 构建 Docker 镜像 ==="
$DOCKER_CMD build -t codex-china-build .

# 运行编译容器
echo ""
echo "=== 运行编译 ==="
$DOCKER_CMD run --rm -it \
    -v "$(pwd):/codex" \
    -w /codex/codex-rs \
    codex-china-build \
    bash -c "
        echo '=== 检查语法 ==='
        cargo check 2>&1 | head -50
        
        echo ''
        echo '=== 编译项目 ==='
        cargo build 2>&1 | tail -20
        
        echo ''
        echo '=== 运行测试 ==='
        cargo test --package codex-api chat_completions 2>&1 | tail -30
        cargo test --package model-provider china_provider 2>&1 | tail -30
    "

echo ""
echo "=== 编译完成 ==="
