//! 候选只保存索引；上屏必须回到 Core，不能直接提交显示文本。
#pragma once
#include "engine.h"
class QingjianCandidate : public fcitx::CandidateWord {
public:
    QingjianCandidate(QingjianEngine *engine, size_t index, const char *text, const char *annotation)
        : CandidateWord(fcitx::Text(text)), engine_(engine), index_(index) {
        setComment(fcitx::Text(annotation));
    }
    void select(fcitx::InputContext *ic) const override { engine_->select(ic, index_); }
private:
    /// 候选列表所属的引擎。
    QingjianEngine *engine_;
    /// Core 返回的原始候选下标。
    size_t index_;
};
