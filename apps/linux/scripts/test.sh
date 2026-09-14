#!/usr/bin/env bash
# 真 Fcitx 插件加载测试，用临时 XDG 目录，不连接桌面会话。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
[[ "$TARGET" = /* ]] || TARGET="$ROOT/$TARGET"
TEMP="$(mktemp -d "$TARGET/linux-test.XXXXXX")"
trap 'rm -rf "$TEMP"' EXIT
export QINGJIAN_PREFIX="$TEMP/prefix" XDG_DATA_HOME="$TEMP/data" XDG_CONFIG_HOME="$TEMP/config"
export XDG_CACHE_HOME="$TEMP/cache" XDG_RUNTIME_DIR="$TEMP/runtime"
mkdir -m700 "$XDG_RUNTIME_DIR"
unset DBUS_SESSION_BUS_ADDRESS DISPLAY WAYLAND_DISPLAY QINGJIAN_DATA_DIR
if [[ $# -eq 0 ]]; then
  apps/linux/scripts/install.sh
elif [[ $# -eq 2 && "$1" == --prefix && -d "$2" ]]; then
  export QINGJIAN_PREFIX="$2"
  export FCITX_ADDON_DIRS="$2/lib/fcitx5"
  export XDG_DATA_DIRS="$2/share:/usr/share"
else
  echo '用法：test.sh [--prefix 已提取安装包的/usr目录]' >&2
  exit 1
fi
read -r -a FCITX_FLAGS <<< "$(pkg-config --cflags --libs Fcitx5Core)"
"${CXX:-c++}" -std=c++20 -Wall -Wextra -Werror apps/linux/tests/fcitx.cpp "${FCITX_FLAGS[@]}" -o "$TEMP/fcitx-test"
"$TEMP/fcitx-test" --disable=all --enable=qingjian,keyboard
