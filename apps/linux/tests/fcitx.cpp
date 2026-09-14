//! 在隔离配置下通过真实 Fcitx 插件加载器验证按键、候选、隐私和多输入框。
#include <cassert>
#include <iostream>
#include <fcitx/instance.h>
#include <fcitx/addonmanager.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/candidatelist.h>
#include <fcitx-utils/capabilityflags.h>

class TestContext : public fcitx::InputContext {
public:
    explicit TestContext(fcitx::InputContextManager &manager) : InputContext(manager, "qingjian-test") {
        setCapabilityFlags(fcitx::CapabilityFlag::Preedit); created();
    }
    ~TestContext() override { destroy(); }
    const char *frontend() const override { return "qingjian-test"; }
    std::string committed;
private:
    void commitStringImpl(const std::string &text) override { committed += text; }
    void deleteSurroundingTextImpl(int, unsigned) override {}
    void forwardKeyImpl(const fcitx::ForwardKeyEvent &) override {}
    void updatePreeditImpl() override {}
};

int main(int argc, char **argv) {
    fcitx::Instance instance(argc, argv);
    instance.addonManager().registerDefaultLoader(nullptr);
    instance.addonManager().load({"qingjian"}, {"all"});
    auto *engine = dynamic_cast<fcitx::InputMethodEngine *>(instance.addonManager().addon("qingjian", true));
    assert(engine && "Fcitx must load the installed plugin");
    fcitx::InputMethodEntry entry("qingjian", "Qingjian", "zh_CN", "qingjian");
    TestContext a(instance.inputContextManager()), b(instance.inputContextManager());
    auto press = [&](TestContext &ic, const fcitx::Key &key, bool release = false) {
        fcitx::KeyEvent event(&ic, key, release); engine->keyEvent(entry, event); return event.accepted();
    };
    auto type = [&](TestContext &ic, const std::string &text) {
        for (unsigned char c : text) { assert(press(ic, fcitx::Key(static_cast<fcitx::KeySym>(c)))); }
    };
    auto reset = [&](TestContext &ic) {
        fcitx::InputContextEvent event(&ic, fcitx::EventType::InputContextReset); engine->reset(entry, event);
    };
    type(a, "kaifa");
    auto list = std::dynamic_pointer_cast<fcitx::CommonCandidateList>(a.inputPanel().candidateList());
    assert(list && list->totalSize() > 5);
    assert(list->pageSize() == 5 && "must load installed defaults when user config is absent");
    bool found = false;
    for (int i = 0; i < list->totalSize(); ++i) {
        auto &candidate = list->candidateFromAll(i);
        if (candidate.text().toString() == "开发") {
            assert(!candidate.comment().toString().empty());
            candidate.select(&a); found = true; break;
        }
    }
    assert(found && a.committed == "开发");
    assert(a.inputPanel().preedit().toString().empty());
    type(a, "kaifa");
    list = std::dynamic_pointer_cast<fcitx::CommonCandidateList>(a.inputPanel().candidateList());
    assert(list->candidate(0).comment().toString().find(" / ") != std::string::npos);
    a.committed.clear();
    assert(press(a, fcitx::Key(FcitxKey_exclam, fcitx::KeyStates(fcitx::KeyState::Shift) | fcitx::KeyState::Alt)));
    assert(!a.committed.empty() && a.committed != "开发");
    type(a, "shi");
    assert(press(a, fcitx::Key(FcitxKey_Page_Down)));
    list = std::dynamic_pointer_cast<fcitx::CommonCandidateList>(a.inputPanel().candidateList());
    assert(list->currentPage() == 1);
    const auto expected = list->candidate(0).text().toString();
    a.committed.clear();
    assert(press(a, fcitx::Key(FcitxKey_1)));
    assert(a.committed == expected);
    type(a, "kaifa"); type(b, "biancheng");
    const auto before = b.inputPanel().preedit().toString();
    reset(a); assert(b.inputPanel().preedit().toString() == before);
    assert(!press(b, fcitx::Key(FcitxKey_c, fcitx::KeyState::Ctrl)));
    assert(b.inputPanel().preedit().toString() == before);
    assert(!press(b, fcitx::Key(FcitxKey_a), true));
    b.setCapabilityFlags(fcitx::CapabilityFlag::Password);
    b.committed.clear();
    assert(!press(b, fcitx::Key(FcitxKey_a)));
    assert(b.inputPanel().preedit().toString().empty() && b.committed.empty());
    b.setCapabilityFlags(fcitx::CapabilityFlag::Preedit);
    assert(!press(b, fcitx::Key(FcitxKey_A, fcitx::KeyState::CapsLock)));
    assert(b.inputPanel().preedit().toString().empty());
    type(b, "kaifa");
    assert(press(b, fcitx::Key(FcitxKey_Return)));
    assert(b.committed == "kaifa");
    std::cout << "Fcitx integration passed: plugin loading, dictionary + gloss, mouse select, page select, context isolation, shortcuts, key release, password bypass, raw commit.\n";
}
