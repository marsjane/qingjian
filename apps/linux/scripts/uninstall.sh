#!/usr/bin/env bash
# 只移除本插件的程序和注册文件，保留配置、学习数据与其他输入法。
set -euo pipefail
PREFIX="${QINGJIAN_PREFIX:-$HOME/.local}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
rm -f "$PREFIX/lib/fcitx5/qingjian.so" "$PREFIX/lib/fcitx5/libqingjian_linux.so"       "$DATA_HOME/fcitx5/addon/qingjian.conf" "$DATA_HOME/fcitx5/inputmethod/qingjian.conf"
echo '青简插件已移除，配置与学习记录已保留。请从 Fcitx5 输入法列表移除青简并重启 Fcitx5。'
