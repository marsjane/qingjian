//! Fcitx 已解码的按键映射到 Engine；候选导航交给 Fcitx 自己处理。

use super::{Session, cstring};
use qingjian_platform::Modifiers;

// 值与 XKB / Fcitx keysym 相同；普通文本另外传 Unicode 标量。
pub const BACKSPACE: u32 = 0xff08;
pub const RETURN: u32 = 0xff0d;
pub const ESCAPE: u32 = 0xff1b;
pub const DELETE: u32 = 0xffff;
pub const LEFT: u32 = 0xff51;
pub const RIGHT: u32 = 0xff53;
pub const HOME: u32 = 0xff50;
pub const END: u32 = 0xff57;

impl Session {
    pub fn key(&mut self, sym: u32, unicode: u32, modifiers: u32, selected: usize) -> bool {
        self.committed = Default::default();
        let mods = Modifiers {
            shift: modifiers & 1 != 0,
            control: modifiers & 2 != 0,
            option: modifiers & 4 != 0,
            command: modifiers & 8 != 0,
        };
        let composing = !self.engine.composition().is_empty();
        // Alt/Control/Super 等应用快捷键原样放行；译词选择由 C++ 转成 select 调用。
        if mods.control || mods.option || mods.command {
            return false;
        }
        if composing {
            match sym {
                ESCAPE => {
                    self.reset();
                    return true;
                }
                RETURN | 0xff8d => {
                    self.committed = cstring(self.engine.take_raw());
                    self.refresh();
                    return true;
                }
                BACKSPACE => {
                    self.engine.backspace();
                }
                DELETE => {
                    self.engine.delete_forward();
                }
                LEFT => {
                    self.engine.move_cursor_left();
                }
                RIGHT => {
                    self.engine.move_cursor_right();
                }
                HOME => self.engine.move_cursor_home(),
                END => self.engine.move_cursor_end(),
                _ => return self.character(unicode, selected),
            }
            self.refresh();
            return true;
        }
        if sym == BACKSPACE {
            self.engine.note_backspace();
        }
        self.character(unicode, selected)
    }

    fn character(&mut self, unicode: u32, selected: usize) -> bool {
        let Some(c) = char::from_u32(unicode).filter(|c| !c.is_control()) else {
            return false;
        };
        let composing = !self.engine.composition().is_empty();
        let special =
            self.engine.expression_mode() || self.engine.question_mode() || self.engine.raw_mode();
        if c.is_ascii_alphabetic()
            || (composing && (c == '\'' || (special && c != ' ')))
            || (c == ';' && self.engine.takes_semicolon())
        {
            // 上限防止无限按键积累拖慢全局输入服务。
            if self.engine.composition().text().len() >= 128 {
                self.committed = cstring(self.engine.take_raw());
            }
            self.engine.push(c);
            self.refresh();
            return true;
        }
        if c == ' ' && composing {
            if !self.select(selected, None) {
                self.committed = cstring(self.engine.take_raw());
                self.refresh();
            }
            return true;
        }
        if composing && (c.is_ascii_punctuation() || c.is_ascii_digit()) {
            let mut text = String::new();
            if self.select(selected, None) {
                text.push_str(self.committed.to_str().unwrap_or_default());
            }
            // 候选可能只消费拼音前缀；先保留尾部再上标点，不能遗失键串。
            text.push_str(&self.engine.take_raw());
            if let Some(punctuation) = self.engine.punctuate(c) {
                text.push_str(punctuation);
            } else {
                text.push(c);
                self.engine.note_passthrough(c);
            }
            self.committed = cstring(text);
            self.refresh();
            return true;
        }
        if !composing {
            if let Some(punctuation) = self.engine.punctuate(c) {
                self.committed = cstring(punctuation);
                return true;
            }
            self.engine.note_passthrough(c);
        }
        false
    }
}
