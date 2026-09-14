//! Fcitx5 输入法引擎，负责事件接入和候选窗口。
#pragma once
#include "state.h"
#include <fcitx/inputmethodengine.h>
#include <fcitx/instance.h>
#include <fcitx/candidatelist.h>
class QingjianEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit QingjianEngine(fcitx::Instance *instance);
    void activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &) override;
    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &) override;
    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &) override;
    void select(fcitx::InputContext *, size_t index, int sense = -1);
    void update(fcitx::InputContext *);
private:
    QjSession *session(fcitx::InputContext *);
    void commitAndUpdate(fcitx::InputContext *);
    /// Fcitx 全局配置，复用用户设置的翻页与选词键。
    fcitx::Instance *instance_;
    /// 跨会话共享的配置路径和学习记录。
    std::unique_ptr<QjRuntime, decltype(&qj_runtime_free)> runtime_{nullptr, qj_runtime_free};
    /// 比 runtime 先析构，确保所有会话先释放。
    fcitx::FactoryFor<QingjianState> factory_;
};
