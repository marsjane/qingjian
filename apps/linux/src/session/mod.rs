//! 每个 Fcitx5 输入框独立的 Engine、候选与组句状态。

mod keys;
#[cfg(test)]
mod tests;

use qingjian_core::{Candidate, CandidateList, Engine};
use qingjian_platform::Config;
use std::ffi::CString;

pub struct Session {
    /// 平台无关的输入引擎。
    pub engine: Engine,

    /// 本会话生效的配置。
    pub config: Config,

    /// 原始候选，选择时交回 Core 消耗对应的拼音。
    candidates: Vec<Candidate>,

    /// C ABI 可借用的候选显示文本。
    pub labels: Vec<CString>,

    /// C ABI 可借用的单语言译词。
    pub annotations: Vec<CString>,

    /// 行内预编辑文本。
    pub preedit: CString,

    /// 预编辑光标的 UTF-8 字节下标。
    pub cursor: usize,

    /// 本次事件产生的上屏文本。
    pub committed: CString,
}

pub fn cstring(s: impl AsRef<str>) -> CString {
    CString::new(s.as_ref().replace('\0', "")).expect("NUL removed")
}

impl Session {
    pub fn new(engine: Engine, config: Config) -> Self {
        Self {
            engine,
            config,
            candidates: Vec::new(),
            labels: Vec::new(),
            annotations: Vec::new(),
            preedit: CString::default(),
            cursor: 0,
            committed: CString::default(),
        }
    }

    pub fn refresh(&mut self) {
        self.labels.clear();
        self.annotations.clear();
        self.candidates.clear();
        self.preedit = cstring(self.engine.composition().text());
        self.cursor = self.engine.composition().cursor();
        if self.engine.composition().is_empty() {
            return;
        }
        if let Ok(query) = self.engine.query() {
            self.preedit = cstring(query.marked_text());
            self.cursor = query.marked_cursor();
            self.candidates = query.candidates.items;
            // 本地释义表不做网络调用；预先标注也覆盖鼠标翻页。
            self.labels = self.candidates.iter().map(|c| cstring(&c.text)).collect();
            self.annotations
                .resize(self.candidates.len(), CString::default());
            self.annotate_range(0, self.candidates.len());
        }
    }

    pub fn annotate_range(&mut self, start: usize, count: usize) {
        let end = start.saturating_add(count).min(self.candidates.len());
        if start >= end {
            return;
        }
        let mut list = CandidateList {
            items: self.candidates[start..end].to_vec(),
        };
        self.engine.annotate(&mut list);
        for (i, candidate) in list.items.into_iter().enumerate() {
            let annotation = candidate
                .translation
                .as_ref()
                .map(|t| {
                    t.senses()
                        .iter()
                        .map(|s| {
                            let pos = s
                                .part_of_speech
                                .map(|p| format!("{} ", p.abbreviation()))
                                .unwrap_or_default();
                            let reading = s
                                .reading
                                .as_ref()
                                .map(|r| format!(" ({r})"))
                                .unwrap_or_default();
                            format!("{pos}{}{reading}", s.text)
                        })
                        .collect::<Vec<_>>()
                        .join(" / ")
                })
                .unwrap_or_default();
            self.annotations[start + i] = cstring(annotation);
            self.candidates[start + i] = candidate;
        }
    }

    pub fn select(&mut self, index: usize, translation: Option<usize>) -> bool {
        self.committed = CString::default();
        self.annotate_range(index, 1);
        let Some(candidate) = self.candidates.get(index).cloned() else {
            return false;
        };
        let text = match translation {
            Some(sense) => self.engine.commit_translation(&candidate, sense),
            None => Some(self.engine.commit(&candidate)),
        };
        let Some(text) = text else {
            return false;
        };
        self.committed = cstring(text);
        self.refresh();
        true
    }

    pub fn reset(&mut self) {
        self.engine.clear();
        self.engine.break_chain();
        self.engine.history_mut().clear();
        self.engine.learner_mut().flush();
        self.committed = CString::default();
        self.refresh();
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.engine.learner_mut().flush();
    }
}
