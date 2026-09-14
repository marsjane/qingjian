#!/usr/bin/env bash
# 从当前提交构建、验证，并生成可直接 pacman -U 安装的包。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
python3 apps/linux/packaging/package.py "$@"
