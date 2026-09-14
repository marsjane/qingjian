# Linux Fcitx5 alpha

Linux 壳优先接入 Arch / Omarchy 已有的 Fcitx5。C++ 插件处理输入框生命周期、系统按键和原生候选列表；Rust 会话只通过 Core 的 Engine 门面完成拼音解析、候选与译词、标点和上屏。

当前 main 中尚无可用的共享渲染器；Linux alpha 暂用 Fcitx5 原生候选窗口，借用其 Wayland 弹窗定位与焦点管理。后续再对齐青简的自绘显示面与主题，alpha 不开放尚未实现的外观选项。

每个输入框拥有独立 Engine。只共享基本词频／选择／敲错记录，不共享组句、输入历史或候选。尚未接入需要借用个人词库和 n-gram 的完整学习接口。密码／敏感输入绕过输入引擎，普通组句在失焦或停用时清空。

二进制安装到用户目录，使用绝对插件路径和相邻 Rust 库的 `$ORIGIN` RPATH，不修改 Omarchy 系统配置。词库使用官方 data Release 校验过的文件，所有网络操作只发生在显式下载脚本中。

第一版不实现云联想、神经重排或图形设置；边界在 Linux README 和用户试用文档中列明，避免把 alpha 当作 macOS / Windows 的完整移植。
