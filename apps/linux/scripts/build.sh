#!/usr/bin/env bash
# 构建 Rust 引擎与 Fcitx5 插件，不写系统目录。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
command -v cargo >/dev/null || { echo "缺少 cargo，请安装 Rust 1.96+" >&2; exit 1; }
pkg-config --atleast-version=5.1.9 Fcitx5Core || { echo "缺少 Fcitx5 5.1.9+ 开发文件" >&2; exit 1; }
cargo build -p qingjian-linux --release --locked
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
[[ "$TARGET" = /* ]] || TARGET="$ROOT/$TARGET"
OUT="$TARGET/linux"
mkdir -p "$OUT"
read -r -a FCITX_FLAGS <<< "$(pkg-config --cflags --libs Fcitx5Core)"
"${CXX:-c++}" -std=c++20 -O2 -Wall -Wextra -Werror -fPIC -shared   apps/linux/fcitx5/engine.cpp apps/linux/fcitx5/factory.cpp   "${FCITX_FLAGS[@]}" -L"$TARGET/release" -lqingjian_linux -ldl   -Wl,-rpath,'$ORIGIN' -Wl,-z,defs -o "$OUT/qingjian.so"
cp "$TARGET/release/libqingjian_linux.so" "$OUT/"
echo "已构建：$OUT/qingjian.so"
