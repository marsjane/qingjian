#!/usr/bin/env bash
# 打 Arch / Omarchy 的 pacman 包 fcitx5-qingjian（marsjane/qingjian fork 自用）：本机原生编译，不用容器。
#
#   apps/linux/packaging/arch/build.sh
#   QINGJIAN_SAMPLE=1 apps/linux/packaging/arch/build.sh   只带样例词库（没有产品数据时）
#
# 编译依赖：sudo pacman -S --needed base-devel rust cmake fcitx5 nlohmann-json
# 产品数据先用 tools/release/data-fetch.sh 下好。产物在 target/linux-pkg/out/。
# 布局与 deb / rpm 相同（前缀 /usr），但只装 Server、设置程序与 fcitx5 插件：Omarchy 用 fcitx5，不带 IBus 前端。
# Arch 是滚动发行，对着本机的 fcitx5 编就是对着用户的 fcitx5 编，所以不进容器。
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../../.." && pwd)
here="$root/apps/linux/packaging/arch"
work="$root/target/linux-pkg/arch"
out="$root/target/linux-pkg/out"

missing=()
for tool in cargo cmake c++ pkg-config makepkg python3; do
  command -v "$tool" >/dev/null 2>&1 || missing+=("$tool")
done
if command -v pkg-config >/dev/null 2>&1; then
  for module in Fcitx5Core Fcitx5Utils Fcitx5Config nlohmann_json openssl; do
    pkg-config --exists "$module" || missing+=("$module")
  done
fi
if ((${#missing[@]})); then
  echo "缺少：${missing[*]}" >&2
  echo 'sudo pacman -S --needed base-devel rust cmake fcitx5 nlohmann-json' >&2
  exit 1
fi
# 包版本要能追溯到提交：工作区有改动就不打
if [[ -n "$(git -C "$root" status --porcelain --untracked-files=no)" ]]; then
  echo '工作区有未提交的改动，先提交再打包' >&2
  exit 1
fi

# 版本：正好在 vX.Y.Z 标签上就是标签版本，否则是「标签.r提交数.g哈希」（排在下一个正式版之前）。
# pacman 版本里不能有 -，预发布的 0.1.5-beta.1 写成 0.1.5.beta.1。
described=$(git -C "$root" describe --tags --long --match 'v[0-9]*')
[[ "$described" =~ ^v(.+)-([0-9]+)-g([0-9a-f]+)$ ]] || { echo "认不出版本：$described" >&2; exit 1; }
pkgver=${BASH_REMATCH[1]//-/.}
((BASH_REMATCH[2] == 0)) || pkgver+=".r${BASH_REMATCH[2]}.g${BASH_REMATCH[3]}"
echo "==> 版本 $pkgver（$described）"

echo '==> Rust：Server 与设置程序'
cargo build --manifest-path "$root/Cargo.toml" --release --locked -p qingjian-linux-server -p qingjian-settings
bin="${CARGO_TARGET_DIR:-$root/target}/release"

echo '==> fcitx5 插件'
rm -rf "$work"
cmake -S "$root/apps/linux/fcitx5" -B "$work/fcitx5-build" -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTING=OFF >/dev/null
cmake --build "$work/fcitx5-build" --parallel >/dev/null

echo '==> 随包资源'
sample_flag=()
[[ "${QINGJIAN_SAMPLE:-}" != 1 ]] || sample_flag=(--sample)
stage="$work/stage"
python3 "$root/apps/linux/scripts/files.py" resources "$stage/usr/share/qingjian/resources" --root "$root" "${sample_flag[@]}"

install -Dm755 "$bin/qingjian-linux-server" "$stage/usr/bin/qingjian-linux-server"
install -Dm755 "$bin/qingjian-settings" "$stage/usr/bin/qingjian-settings"
install -Dm644 "$work/fcitx5-build/qingjian.so" "$stage/usr/lib/fcitx5/qingjian.so"
for kind in addon inputmethod; do
  install -Dm644 "$root/apps/linux/fcitx5/data/$kind/qingjian.conf" "$stage/usr/share/fcitx5/$kind/qingjian.conf"
done
mkdir -p "$stage/usr/lib/systemd/user"
sed 's|@EXEC@|/usr/bin/qingjian-linux-server|' "$root/apps/linux/scripts/qingjian-server.service.in" \
  > "$stage/usr/lib/systemd/user/qingjian-server.service"
install -Dm644 "$root/apps/settings/linux/qingjian-settings.desktop" "$stage/usr/share/applications/qingjian-settings.desktop"
for size in 16 22 24 32 48 64 128 256; do
  install -Dm644 "$root/assets/icon/linux/hicolor/${size}x${size}/apps/qingjian.png" \
    "$stage/usr/share/icons/hicolor/${size}x${size}/apps/qingjian.png"
done
install -Dm644 "$root/LICENSE" "$stage/usr/share/licenses/fcitx5-qingjian/LICENSE"
install -Dm644 "$root/assets/stroke/LICENSE-CNS11643.txt" "$stage/usr/share/licenses/fcitx5-qingjian/LICENSE-CNS11643.txt"
# 宿主机 umask 是 002 时拷出来的是 664 / 775
find "$stage" -type d -exec chmod 755 {} +
find "$stage" -type f ! -path '*/usr/bin/*' -exec chmod 644 {} +

echo '==> makepkg'
mkdir -p "$work/makepkg" "$out"
cp "$here/PKGBUILD" "$here/qingjian.install" "$work/makepkg/"
rm -f "$out"/fcitx5-qingjian-*.pkg.tar.zst
(cd "$work/makepkg" && QINGJIAN_STAGE="$stage" QINGJIAN_PKGVER="$pkgver" PKGDEST="$out" makepkg --force --nodeps)
package=$(ls "$out"/fcitx5-qingjian-"$pkgver"-*.pkg.tar.zst)

echo '==> 自检'
# 包里该有的都在，插件按名字能找到，没有把构建机的路径打进去
listing=$(tar -tf "$package")
for path in usr/bin/qingjian-linux-server usr/bin/qingjian-settings usr/lib/fcitx5/qingjian.so \
  usr/share/fcitx5/addon/qingjian.conf usr/share/fcitx5/inputmethod/qingjian.conf \
  usr/lib/systemd/user/qingjian-server.service usr/share/qingjian/resources/assets/; do
  grep -qx "$path" <<<"$listing" || { echo "包里缺 $path" >&2; exit 1; }
done
if [[ "${QINGJIAN_SAMPLE:-}" != 1 ]]; then
  grep -qx usr/share/qingjian/resources/data/generated/dict.qj <<<"$listing" || { echo '包里缺产品词库' >&2; exit 1; }
fi
(cd "$out" && sha256sum "$(basename "$package")" > SHA256SUMS)
echo "==> 产物：$package"
cat "$out/SHA256SUMS"
