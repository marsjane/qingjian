//! Fcitx 输入框属性；每个输入框独占组句状态。
#pragma once
#include "bridge.h"
#include <memory>
#include <fcitx/inputcontextproperty.h>
struct QingjianState : fcitx::InputContextProperty {
    /// 仅切到青简时初始化，普通英文输入框不加载词库。
    std::unique_ptr<QjSession, decltype(&qj_session_free)> session{nullptr, qj_session_free};
};
