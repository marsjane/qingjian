# Arch / Omarchy 安装包（marsjane fork）

本目录只存在于 fork 的 `omarchy` 分支。它在上游 Linux 版之上加一个 pacman 包 `fcitx5-qingjian`，其余代码与上游一致。

- `origin`：<https://github.com/marsjane/qingjian>，发布 Arch 包。
- `upstream`：<https://github.com/qingjian-team/qingjian>，只读。
- `main` 只跟上游；`omarchy` = `main` + 本目录。
- 旧的进程内 Fcitx5 实现（`linux-v0.1.0-alpha.1`）保存在 `omarchy-alpha` 分支，不再维护。

## 包里有什么

布局与上游 deb / rpm 相同（前缀 `/usr`），只装 Fcitx5 需要的部分：

| 路径 | 内容 |
| --- | --- |
| `/usr/bin/qingjian-linux-server` | 引擎（Server），systemd 用户服务 `qingjian-server.service` |
| `/usr/bin/qingjian-settings` | 设置程序，应用菜单「青简设置」 |
| `/usr/lib/fcitx5/qingjian.so` | Fcitx5 插件，经 Unix socket 连 Server |
| `/usr/share/qingjian/resources` | 词库、语言模型、释义表 |

不带 IBus 前端。装包时对所有用户 `systemctl --global enable` Server，并重启已登录用户的 Server。卸载时撤销。
配置在 `~/.config/qingjian/config.toml`，包不写它。

## 同步上游并打包

1. 同步 `main`：

   ```bash
   git fetch upstream --tags
   git switch main && git merge --ff-only upstream/main
   ```

2. 把 `main` 合进 `omarchy`：

   ```bash
   git switch omarchy && git merge main
   ```

   本分支只加了 `apps/linux/packaging/arch/`，正常不会冲突。

3. 安装编译依赖（一次）：

   ```bash
   sudo pacman -S --needed base-devel rust cmake fcitx5 nlohmann-json
   ```

4. 下载产品数据，再打包：

   ```bash
   tools/release/data-fetch.sh
   apps/linux/packaging/arch/build.sh
   ```

   结果：`target/linux-pkg/out/` 下有 `fcitx5-qingjian-<版本>-1-x86_64.pkg.tar.zst` 与 `SHA256SUMS`。
   版本在 `vX.Y.Z` 标签上是 `X.Y.Z`，否则是 `X.Y.Z.r<提交数>.g<哈希>`。工作区有未提交改动时脚本拒绝打包。

5. 上游改了 Linux 安装的文件清单（`apps/linux/scripts/files.py`、`apps/linux/packaging/build.sh` 的 `stage_core`）时，
   对照着改 `build.sh`。脚本末尾的自检会报缺失的文件。

## 发布

标签用 `linux-v<pkgver>`，不触发上游的发版任务。已发布的版本不覆盖：

```bash
git push origin omarchy
git tag -a linux-v<pkgver> -m 'Linux：Omarchy <pkgver>'
git push origin linux-v<pkgver>
gh release create linux-v<pkgver> --repo marsjane/qingjian --prerelease \
  --title 'Omarchy / Arch <pkgver>' target/linux-pkg/out/*
```

dots 仓库的 `config/qingjian/version.lock` 锁定 release 的 tag、版本与 SHA256，由 `scripts/install-qingjian.sh` 安装。

## 本机试装

```bash
sudo pacman -U target/linux-pkg/out/fcitx5-qingjian-*.pkg.tar.zst
systemctl --user status qingjian-server.service   # 应为 active (running)
systemctl --user restart omarchy-fcitx5.service
fcitx5-configtool                                 # 取消「仅显示当前语言」，添加「青简」
```

Fcitx5 默认把左 `Shift` 设为「在当前和第一个输入法之间切换」，它先于青简拿到按键。
要用青简的 `Shift` 中英切换，在 Fcitx5「全局选项 → 快捷键」清空这一项。
