#!/usr/bin/env bash
# 从旧用户级安装迁到 pacman 包；备份旧程序和注册文件，保留设置及学习记录。
set -euo pipefail
[[ -f /usr/share/fcitx5/inputmethod/qingjian.conf ]] || {
  echo '请先安装 fcitx5-qingjian 的 pacman 包，再运行本命令。' >&2
  exit 1
}
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
STATE_HOME="${XDG_STATE_HOME:-$HOME/.local/state}"
ADDON="$DATA_HOME/fcitx5/addon/qingjian.conf"
if [[ -f "$ADDON" ]] && ! grep -Fxq "Library=$HOME/.local/lib/fcitx5/qingjian" "$ADDON"; then
  echo "已有自定义插件路径，请先检查 $ADDON；本命令未修改任何文件。" >&2
  exit 1
fi
BACKUP="$STATE_HOME/qingjian/local-install-$(date +%Y%m%d-%H%M%S)"
for item in "$ADDON" "$DATA_HOME/fcitx5/inputmethod/qingjian.conf"   "$HOME/.local/lib/fcitx5/qingjian.so" "$HOME/.local/lib/fcitx5/libqingjian_linux.so"; do
  if [[ -e "$item" ]]; then
    relative="${item#"$HOME/"}"
    mkdir -p "$BACKUP/$(dirname "$relative")"
    mv -- "$item" "$BACKUP/$relative"
  fi
done
echo "旧用户级程序已移走（若存在），设置与学习记录保留。备份：$BACKUP"
echo '重启 Fcitx5 后将使用 pacman 安装的版本。'
