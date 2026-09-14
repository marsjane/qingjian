#!/usr/bin/env bash
# 用户级安装；已有配置保留，不改 Hyprland 或现有 Fcitx 输入法列表。
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
PREFIX="${QINGJIAN_PREFIX:-$HOME/.local}"
DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
TARGET="${CARGO_TARGET_DIR:-$ROOT/target}"
[[ "$TARGET" = /* ]] || TARGET="$ROOT/$TARGET"
[[ "$PREFIX" = /* ]] || { echo 'QINGJIAN_PREFIX 必须是绝对路径' >&2; exit 1; }
[[ -f "$TARGET/linux/qingjian.so" ]] || { echo '先运行 build.sh' >&2; exit 1; }
[[ -f "$ROOT/data/generated/dict.qj" ]] || { echo '先运行 fetch-data.sh 下载正式词库' >&2; exit 1; }
# Fcitx Library 以分号分隔，不能接受带换行或分号的前缀。
[[ "$PREFIX" != *';'* && "$PREFIX" != *$'\n'* ]] || exit 1
mkdir -p "$PREFIX/lib/fcitx5" "$PREFIX/share/qingjian"   "$DATA_HOME/fcitx5/addon" "$DATA_HOME/fcitx5/inputmethod" "$CONFIG_HOME/qingjian"
# 原子替换库文件，避免截断正在被 fcitx mmap 的旧文件。
for f in qingjian.so libqingjian_linux.so; do
  install -m755 "$TARGET/linux/$f" "$PREFIX/lib/fcitx5/$f.new"
  mv -f "$PREFIX/lib/fcitx5/$f.new" "$PREFIX/lib/fcitx5/$f"
done
mkdir -p "$PREFIX/share/qingjian/data/generated" "$PREFIX/share/qingjian/assets"
# 安装数据也用临时文件 + rename；当前进程继续使用旧 mmap。
python3 - "$ROOT" "$PREFIX/share/qingjian" <<'PYCOPY'
import os, pathlib, shutil, sys
root, dest = map(pathlib.Path, sys.argv[1:])
for sub in ['data/generated', 'assets/levels', 'assets/emoji']:
    for source in (root/sub).rglob('*'):
        if source.is_file():
            target = dest/source.relative_to(root)
            target.parent.mkdir(parents=True, exist_ok=True)
            temp = target.with_name(target.name+'.new')
            shutil.copyfile(source, temp)
            os.replace(temp, target)
shutil.copyfile(root/'LICENSE', dest/'LICENSE')
PYCOPY
python3 - "$ROOT/apps/linux/fcitx5/addon.conf" "$PREFIX" "$DATA_HOME/fcitx5/addon/qingjian.conf" <<'PYCONF'
import pathlib,sys
source,prefix,dest=sys.argv[1:]
text=pathlib.Path(source).read_text().replace('Library=qingjian', 'Library='+prefix+'/lib/fcitx5/qingjian')
pathlib.Path(dest).write_text(text)
PYCONF
install -m644 "$ROOT/apps/linux/fcitx5/inputmethod.conf" "$DATA_HOME/fcitx5/inputmethod/qingjian.conf"
if [[ ! -e "$CONFIG_HOME/qingjian/config.toml" ]]; then
  install -m600 "$ROOT/apps/linux/config.toml" "$CONFIG_HOME/qingjian/config.toml"
fi
echo "青简已安装。重启 Fcitx5 后，在 fcitx5-configtool 添加「青简」。"
echo "Omarchy：systemctl --user restart omarchy-fcitx5.service"
echo "配置：$CONFIG_HOME/qingjian/config.toml"
