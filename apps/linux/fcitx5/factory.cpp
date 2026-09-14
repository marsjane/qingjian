//! Fcitx5 动态插件入口。
#include "engine.h"
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
class QingjianFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *manager) override {
        return new QingjianEngine(manager->instance());
    }
};
FCITX_ADDON_FACTORY_V2(qingjian, QingjianFactory);
