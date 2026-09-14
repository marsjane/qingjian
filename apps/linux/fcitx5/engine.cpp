//! 将 Fcitx 的输入事件转成 Core 调用，交给现有 Wayland UI 显示。
#include "engine.h"
#include "candidate.h"
#include <cstdlib>
#include <dlfcn.h>
#include <filesystem>
#include <stdexcept>
#include <fcitx/inputcontext.h>
#include <fcitx/inputcontextmanager.h>
#include <fcitx/inputpanel.h>
#include <fcitx/globalconfig.h>
#include <fcitx-utils/capabilityflags.h>
#include <fcitx-utils/log.h>

namespace {
std::string envPath(const char *name, const std::filesystem::path &fallback) {
    const auto *value = std::getenv(name);
    return value && *value ? value : fallback.string();
}
uint32_t modifiers(const fcitx::Key &key) {
    const auto s = key.states();
    return (s.test(fcitx::KeyState::Shift) ? 1 : 0) |
           (s.test(fcitx::KeyState::Ctrl) ? 2 : 0) |
           (s.test(fcitx::KeyState::Alt) ? 4 : 0) |
           (s.testAny(fcitx::KeyStates(fcitx::KeyState::Super) | fcitx::KeyState::Super2 |
                      fcitx::KeyState::Hyper | fcitx::KeyState::Meta) ? 8 : 0);
}
}

QingjianEngine::QingjianEngine(fcitx::Instance *instance)
    : instance_(instance), factory_([](fcitx::InputContext &) { return new QingjianState; }) {
    const auto *home = std::getenv("HOME");
    if (!home) { throw std::runtime_error("青简：缺少 HOME"); }
    const auto data = envPath("XDG_DATA_HOME", std::filesystem::path(home) / ".local/share");
    const auto config = envPath("XDG_CONFIG_HOME", std::filesystem::path(home) / ".config");
    // 通过插件本身定位安装前缀，不依赖 fcitx5 可执行文件所在目录。
    Dl_info info{};
    if (!dladdr(reinterpret_cast<void *>(&qj_runtime_new), &info)) {
        throw std::runtime_error("青简：无法定位插件资源");
    }
    const auto prefix = std::filesystem::path(info.dli_fname).parent_path().parent_path().parent_path();
    const auto root = envPath("QINGJIAN_DATA_DIR", prefix / "share/qingjian");
    runtime_.reset(qj_runtime_new(root.c_str(), (std::filesystem::path(config) / "qingjian/config.toml").c_str(),
                                 (std::filesystem::path(data) / "qingjian/user").c_str()));
    if (!runtime_) { throw std::runtime_error("青简：资源加载失败，请检查安装目录"); }
    instance->inputContextManager().registerProperty("qingjianState", &factory_);
}

QjSession *QingjianEngine::session(fcitx::InputContext *ic) {
    auto *state = ic->propertyFor(&factory_);
    if (!state->session) { state->session.reset(qj_session_new(runtime_.get())); }
    return state->session.get();
}

void QingjianEngine::activate(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    auto *ic = event.inputContext();
    if (session(ic)) { update(ic); }
}

void QingjianEngine::reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &event) {
    auto *ic = event.inputContext();
    auto *state = ic->propertyFor(&factory_);
    if (state->session) { qj_reset(state->session.get()); }
    ic->inputPanel().reset();
    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

void QingjianEngine::select(fcitx::InputContext *ic, size_t index, int sense) {
    auto *s = session(ic);
    if (s && !ic->capabilityFlags().testAny(fcitx::CapabilityFlag::PasswordOrSensitive) && qj_select(s, index, sense)) {
        commitAndUpdate(ic);
    }
}

void QingjianEngine::commitAndUpdate(fcitx::InputContext *ic) {
    auto *s = session(ic);
    const std::string text = qj_text(s, 1, 0);
    if (!text.empty()) { ic->commitString(text); }
    update(ic);
}

void QingjianEngine::update(fcitx::InputContext *ic) {
    auto *s = session(ic);
    auto &panel = ic->inputPanel();
    panel.reset();
    if (s && qj_value(s, 5)) {
        fcitx::Text preedit(qj_text(s, 0, 0), fcitx::TextFormatFlag::Underline);
        preedit.setCursor(static_cast<int>(qj_value(s, 1)));
        panel.setPreedit(preedit);
        panel.setClientPreedit(preedit);
        const auto count = qj_value(s, 0);
        if (count) {
            auto list = std::make_unique<fcitx::CommonCandidateList>();
            list->setPageSize(static_cast<int>(qj_value(s, 2)));
            list->setLayoutHint(fcitx::CandidateLayoutHint::Vertical);
            list->setSelectionKey(fcitx::Key::keyListFromString("1 2 3 4 5 6 7 8 9"));
            list->setCursorKeepInSamePage(false);
            for (size_t i = 0; i < count; ++i) {
                list->append<QingjianCandidate>(this, i, qj_text(s, 2, i), qj_text(s, 3, i));
            }
            list->setGlobalCursorIndex(0);
            panel.setCandidateList(std::move(list));
        }
    }
    ic->updatePreedit();
    ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
}

void QingjianEngine::keyEvent(const fcitx::InputMethodEntry &entry, fcitx::KeyEvent &event) {
    if (event.isRelease() || event.key().isModifier()) { return; }
    auto *ic = event.inputContext();
    auto *s = session(ic);
    if (!s) { return; }
    const bool privateInput = ic->capabilityFlags().testAny(fcitx::CapabilityFlag::PasswordOrSensitive);
    qj_private(s, privateInput);
    if (privateInput || event.origKey().states().test(fcitx::KeyState::CapsLock)) {
        reset(entry, event);
        return;
    }
    const auto key = event.key();
    const auto mods = modifiers(event.origKey());
    const auto sym = key.sym();
    const auto unicode = fcitx::Key::keySymToUnicode(sym);
    auto list = std::dynamic_pointer_cast<fcitx::CommonCandidateList>(ic->inputPanel().candidateList());
    size_t selected = list && list->globalCursorIndex() >= 0 ? list->globalCursorIndex() : 0;
    if (list && qj_value(s, 5)) {
        // 数字选择按当前页计算；表达式中的数字必须交回 Core。
        const auto originalSym = event.origKey().sym();
        auto digit = originalSym >= FcitxKey_1 && originalSym <= FcitxKey_9 ? originalSym - FcitxKey_1 : 99;
        // US 布局下 Shift+数字的 keysym 是 !/@/...；保留底层数字选择含义。
        if (digit == 99 && (mods & 1) && unicode < 128) {
            const auto index = std::string("!@#$%^&*(").find(static_cast<char>(unicode));
            if (index != std::string::npos) { digit = static_cast<int>(index); }
        }
        if (digit < 9 && !qj_value(s, 8)) {
            int sense = -2;
            if (mods == 0) { sense = -1; }
            else if (mods == qj_value(s, 6)) { sense = 0; }
            else if (mods == qj_value(s, 7)) { sense = 1; }
            if (sense != -2) {
                if (digit < list->size()) { select(ic, list->currentPage() * list->pageSize() + digit, sense); }
                event.filterAndAccept(); return;
            }
        }
        const auto &global = instance_->globalConfig();
        const bool prevPage = key.keyListIndex(global.defaultPrevPage()) >= 0 ||
            (mods == 0 && (sym == FcitxKey_Page_Up || unicode == qj_value(s, 3)));
        const bool nextPage = key.keyListIndex(global.defaultNextPage()) >= 0 ||
            (mods == 0 && (sym == FcitxKey_Page_Down || unicode == qj_value(s, 4)));
        if (prevPage || nextPage) {
            if (prevPage) { list->prev(); } else { list->next(); }
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            event.filterAndAccept(); return;
        }
        const bool prevCandidate = key.keyListIndex(global.defaultPrevCandidate()) >= 0;
        const bool nextCandidate = key.keyListIndex(global.defaultNextCandidate()) >= 0;
        if (prevCandidate || nextCandidate) {
            if (prevCandidate) { list->prevCandidate(); } else { list->nextCandidate(); }
            ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
            event.filterAndAccept(); return;
        }
    }
    if (qj_key(s, sym, unicode, mods, selected)) {
        commitAndUpdate(ic);
        event.filterAndAccept();
    }
}
