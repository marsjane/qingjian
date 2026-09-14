# Omarchy fork 维护

仓库：<https://github.com/marsjane/qingjian>。原项目：<https://github.com/qingjian-team/qingjian>。

- `origin` 指向个人 fork，代码与 Linux 安装包发布到这里。
- `upstream` 指向原仓库，保持只读使用。
- `main` 保留上游历史；`omarchy` 是本 fork 的默认分支，包含 Fcitx5 接入和打包支持。
- Linux 标签使用 `linux-v<版本>`，不会触发原项目 macOS / Windows 的标签发版任务。

## 首次 clone

```bash
git clone https://github.com/marsjane/qingjian.git
cd qingjian
git remote add upstream https://github.com/qingjian-team/qingjian.git
```

## 同步上游

先提交本地改动，工作区保持干净。不要对带适配的分支执行强制同步或强推。

```bash
git fetch upstream
# main 只跟进上游
git switch main
git merge --ff-only upstream/main
git push origin main
# 将更新合到自己的适配分支
git switch omarchy
git merge main
# 若有冲突：修好冲突、git add 对应文件，然后 git commit
cargo test --workspace --exclude qingjian-macos --locked
cargo clippy --workspace --exclude qingjian-macos --all-targets --locked -- -D warnings
apps/linux/scripts/build.sh
apps/linux/scripts/test.sh
git push origin omarchy
```

在 GitHub 页面同步时，先切到 `main` 分支同步上游，再在本地把 `main` 合到 `omarchy`。
保留适配分支里的 `Cargo.toml` workspace 成员、Linux crate 和 Linux 脚本；发生冲突时按新 Core API 调整接入层。

## 构建开发版本

Rust 1.96+、C++20 编译器、Fcitx5 5.1.9+ 头文件、Python、curl 与 pkg-config 足够，不使用 CMake。
Omarchy 缺少依赖时运行：

```bash
omarchy pkg add base-devel rust fcitx5 fcitx5-configtool python curl
```

本地 Git 钩子可以使用 Linux 版本：

```bash
git config core.hooksPath apps/linux/githooks
```

## 打包发布

先把 `apps/linux/Cargo.toml` 的版本设为正式的 alpha / beta 版本（例如 `0.1.0-alpha.1`），更新 Cargo.lock 并提交。
打包要求工作区干净，保证二进制能精确追溯到 source archive 和 Git 提交。

```bash
apps/linux/scripts/fetch-data.sh  # 已有对应数据可跳过
apps/linux/scripts/package.sh
```

脚本先编译与测试，再用 makepkg 生成标准 `.pkg.tar.zst`，最后提取包、验证注册与资源路径，并通过真实 Fcitx 动态加载器测试包里的插件。
makepkg 只打包本步骤刚编译出的文件，用户安装二进制包无需 Rust 或 GCC。

产物在 `dist/linux-v<版本>/`：

- `fcitx5-qingjian-*.pkg.tar.zst`：Arch / Omarchy x86_64 安装包。
- `qingjian-*-source.tar.gz`：对应 Git 提交的完整源码。
- `qingjian-*-data.tar.zst`：该版本的离线数据，解到源码的 `data/generated/` 后可重新构建。
- `build-info.json`：源码提交、编译器版本、Fcitx5 版本和每个数据文件的 SHA256。
- `SHA256SUMS`：发布文件校验和。

发到自己的仓库，先推代码再建不可变的版本标签；不覆盖已发布版本：

```bash
git push origin omarchy
git tag -a linux-v0.1.0-alpha.1 -m 'Linux：Omarchy alpha 1'
git push origin linux-v0.1.0-alpha.1
gh release create linux-v0.1.0-alpha.1 --repo marsjane/qingjian --prerelease \
  --title 'Omarchy / Arch alpha 1' --notes-file release-notes.md \
  dist/linux-v0.1.0-alpha.1/*
```

## 安装与升级

```bash
sha256sum -c SHA256SUMS
sudo pacman -U ./fcitx5-qingjian-*.pkg.tar.zst
# 仅从旧 install.sh 用户级安装迁过来时运行一次；否则旧注册文件会遮住系统包
qingjian-remove-user-install
systemctl --user restart omarchy-fcitx5.service
fcitx5-configtool
```

新机器直接安装包即可；`qingjian-remove-user-install` 在没有旧用户级文件时不修改文件。
配置保留在 XDG 用户目录；没有用户配置时读取 `/usr/share/qingjian/config.toml`。
升级用新版本的 `pacman -U`；卸载用 `sudo pacman -R fcitx5-qingjian`，用户学习数据和个人设置不随包删除。

本包只验证 Arch / Omarchy x86_64，依赖编译时同版本或更新的 Fcitx5。若以后 Fcitx5 改变库 ABI，需重新编译发布。
