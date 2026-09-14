# 青简 Linux / Fcitx5（alpha）

面向 Arch Linux 与 Omarchy（Hyprland / Wayland）的源码适配。Linux 壳使用 Fcitx5 原生候选窗口，Rust Core 负责候选生成、拼音解析、释义与标点。

## 安装已编译版本

从 [本 fork 的 Releases](https://github.com/marsjane/qingjian/releases) 下载 `.pkg.tar.zst`，然后运行：

```bash
sudo pacman -U ./fcitx5-qingjian-*.pkg.tar.zst
# 从本仓库旧 install.sh 用户级安装升级时，运行一次以清理旧插件覆盖
qingjian-remove-user-install
systemctl --user restart omarchy-fcitx5.service
fcitx5-configtool
```

在 configtool 点击添加，取消「仅显示当前语言」，搜索 `Qingjian` /「青简」。若左侧已经列出青简，无需重复添加。
可保留 Rime；将青简放在键盘项之后、其他中文输入法之前并应用，即可让中英切换优先使用青简。

安装二进制包无需编译环境。编译、发布和同步原项目的流程见 [维护说明](MAINTAINING.md)。

## 构建和用户级安装

需要 Rust 1.96+、支持 C++20 的 GCC/Clang、pkg-config、Fcitx5 5.1.9+、Python 3.12+、curl。
Arch 的 `fcitx5` 包已经包含开发头文件；不需要寻找 Debian 风格的 `-dev` 包。

```bash
# Omarchy：安装缺少的依赖
omarchy pkg add base-devel rust fcitx5 fcitx5-configtool python curl
# 普通 Arch 可用 sudo pacman -S --needed base-devel rust fcitx5 fcitx5-configtool python curl

# 在仓库根目录运行
apps/linux/scripts/fetch-data.sh
apps/linux/scripts/build.sh
apps/linux/scripts/test.sh
apps/linux/scripts/install.sh
```

`fetch-data.sh` 下载上游官方 `data` Release 的词库、bigram 语言模型与译词表，按同一 Release 的 SHA256SUMS 校验。
实际摘要保存在 `data/generated/SOURCE-SHA256SUMS`。上游数据是滚动发布，复现构建需同时保留该数据目录。
不会下载云端生成工具的中间数据或未接入的神经模型。

安装默认写入 `~/.local/lib/fcitx5/`、`~/.local/share/qingjian/`、XDG 数据目录下的 Fcitx 注册文件和 XDG 配置目录下的青简配置。
无需 sudo。设置 `QINGJIAN_PREFIX` 可更改资源与库的安装前缀；`XDG_DATA_HOME`、`XDG_CONFIG_HOME` 也会被尊重。
插件注册文件使用绝对库路径，不需改 `LD_LIBRARY_PATH` 或 Hyprland 启动配置。已有青简配置不会覆盖。

安装后在 Omarchy 执行：

```bash
systemctl --user restart omarchy-fcitx5.service
fcitx5-configtool
```

在 Fcitx5「输入法」页点击添加，取消「仅显示当前语言」，搜索 `Qingjian` 或「青简」。可保留 Rime 并按需要切换。
若普通 Arch 已有其他 Fcitx5 启动方式，沿用现有方式重启，不要同时启动第二个实例。

## 功能与按键

- 全拼、Core 已有的四种双拼、模糊音、自定义短语、词库整句转换、英文或日文单语言译词。
- `Space` 选高亮候选；`1`–`9` 选当前页对应候选，前缀候选上屏后继续输入剩余拼音；鼠标选词可用。
- `Enter` 原样上屏；`Esc` 清除组句；`Backspace`、`Delete`、`←`、`→`、`Home`、`End` 编辑拼音。
- `[` / `]` 和 `PageUp` / `PageDown` 翻页；同时遵从 Fcitx5 全局翻页键、上一／下一候选键设置。
- `Alt + 数字` 上屏第一条译词；`Shift + Alt + 数字` 上屏第二条译词，两组修饰键可以在 `[shortcut]` 配置。
- 中英切换由 Fcitx5 负责，沿用已有快捷键；Omarchy 通常为 `Ctrl + Space`，本机已有左 `Shift` 备用切换键。
- Caps Lock 和密码／敏感输入框直通；各输入框独立组句、共享基本词频与选词偏好。Ctrl/Alt/Super 应用快捷键继续放行。

配置位于 `${XDG_CONFIG_HOME:-~/.config}/qingjian/config.toml`，修改后重启 Fcitx5。
示例见同目录 [config.toml](config.toml)。`learning_language` 支持 `en` / `ja`，`shuangpin` 支持空串（全拼）、`xiaohe`、`ziranma`、`microsoft`、`sogou`。

## 当前边界

这是本地 alpha 适配，尚未与 macOS / Windows 功能完全对齐：

- 尚未接入神经重排、云联想、选中文字翻译、英文候选补全、注音、emoji 和词汇统计。
- 尚未接入个人 n-gram 与自动造词；基本词频、按输入串记的选词偏好、敲错记录跨输入框共享，并在重置／退出时保存。
- 不写输入内容日志，不发网络请求。`[model]` / `[predict]` / `input_log` 当前不启用对应能力。
- 候选窗使用 Fcitx5 的外观配置；青简的 `theme` / `layout` / `preedit` 设置暂不生效，固定竖排、窗口与行内均显示拼音。
- 切换应用或停用输入法时取消尚未上屏的组句。配置暂不热加载，无青简专用图形设置页。
- 数字选词与 Alt 译词键已测试 US 键盘布局；其他布局和所有 GTK/Qt/Electron 应用仍需用户实测。

## 验证

```bash
cargo fmt --all --check
cargo clippy --workspace --exclude qingjian-macos --all-targets --locked -- -D warnings
cargo test --workspace --exclude qingjian-macos --locked
apps/linux/scripts/build.sh
apps/linux/scripts/test.sh
```

`test.sh` 将插件安装到临时前缀，使用真实 Fcitx 动态加载器和正式词库模拟两个输入框，验证译词、鼠标选词、翻页数字选词、输入框隔离、快捷键、按键释放和密码框直通。
该测试禁用桌面前端，不触碰正在运行的输入法。它覆盖接口行为，不能代替各应用中的 Wayland 人工输入验收。

## 卸载

pacman 包用 `sudo pacman -R fcitx5-qingjian` 卸载。以下说明仅适用于旧用户级安装。

先在 Fcitx5 配置中移除青简，然后删除自己安装的 `qingjian.so`、`libqingjian_linux.so`、Fcitx 的 `addon/qingjian.conf` 与 `inputmethod/qingjian.conf`，再重启 Fcitx5。
可执行 `apps/linux/scripts/uninstall.sh` 删除默认用户安装的这四个程序／注册文件。配置、词库和学习数据保留，不删除 Rime 或 Fcitx5。
