//! C ABI；句柄由 C++ RAII 独占，字符串借用到下一次可变调用为止。

use crate::{runtime::Runtime, session::Session};
use std::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;

fn protect<T>(fallback: T, f: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(f)).unwrap_or_else(|_| {
        eprintln!("青简：内部错误，当前操作已取消");
        fallback
    })
}

unsafe fn path(p: *const c_char) -> Option<PathBuf> {
    if p.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(p) }
        .to_str()
        .ok()
        .map(PathBuf::from)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_runtime_new(
    root: *const c_char,
    config: *const c_char,
    user: *const c_char,
) -> *mut Runtime {
    protect(std::ptr::null_mut(), || {
        let Some((root, config, user)) =
            (unsafe { path(root).zip(path(config)).zip(path(user)) }).map(|((r, c), u)| (r, c, u))
        else {
            return std::ptr::null_mut();
        };
        match Runtime::new(root, config, user) {
            Ok(runtime) => Box::into_raw(Box::new(runtime)),
            Err(e) => {
                eprintln!("青简加载失败：{e}");
                std::ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_runtime_free(p: *mut Runtime) {
    if !p.is_null() {
        protect((), || unsafe { drop(Box::from_raw(p)) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_session_new(p: *const Runtime) -> *mut Session {
    protect(std::ptr::null_mut(), || {
        let Some(runtime) = (unsafe { p.as_ref() }) else {
            return std::ptr::null_mut();
        };
        match runtime.session() {
            Ok(session) => Box::into_raw(Box::new(session)),
            Err(e) => {
                eprintln!("青简会话创建失败：{e}");
                std::ptr::null_mut()
            }
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_session_free(p: *mut Session) {
    if !p.is_null() {
        protect((), || unsafe { drop(Box::from_raw(p)) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_key(
    p: *mut Session,
    sym: u32,
    unicode: u32,
    modifiers: u32,
    selected: usize,
) -> bool {
    protect(false, || {
        unsafe { p.as_mut() }.is_some_and(|s| s.key(sym, unicode, modifiers, selected))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_select(p: *mut Session, index: usize, sense: i32) -> bool {
    protect(false, || {
        unsafe { p.as_mut() }
            .is_some_and(|s| s.select(index, (sense >= 0).then_some(sense as usize)))
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_reset(p: *mut Session) {
    protect((), || {
        if let Some(s) = unsafe { p.as_mut() } {
            s.reset();
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_private(p: *mut Session, private: bool) {
    protect((), || {
        if let Some(s) = unsafe { p.as_mut() } {
            s.engine.set_private(private);
        }
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_annotate(p: *mut Session, start: usize, count: usize) {
    protect((), || {
        if let Some(s) = unsafe { p.as_mut() } {
            s.annotate_range(start, count);
        }
    });
}

// 以下只读查询不分配、不索引越界、不调用 Core。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_text(p: *const Session, field: u32, index: usize) -> *const c_char {
    let Some(s) = (unsafe { p.as_ref() }) else {
        return c"".as_ptr();
    };
    match field {
        0 => s.preedit.as_ptr(),
        1 => s.committed.as_ptr(),
        2 => s.labels.get(index).map_or(c"".as_ptr(), |v| v.as_ptr()),
        3 => s
            .annotations
            .get(index)
            .map_or(c"".as_ptr(), |v| v.as_ptr()),
        _ => c"".as_ptr(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn qj_value(p: *const Session, field: u32) -> usize {
    let Some(s) = (unsafe { p.as_ref() }) else {
        return 0;
    };
    match field {
        0 => s.labels.len(),
        1 => s.cursor,
        2 => s.config.general.page_size(),
        3 => s.config.general.page_keys().0 as usize,
        4 => s.config.general.page_keys().1 as usize,
        5 => usize::from(!s.engine.composition().is_empty()),
        6 | 7 => {
            let (a, b) = s.config.shortcut.translation_keys();
            let m = if field == 6 { a } else { b };
            usize::from(m.shift)
                | (usize::from(m.control) << 1)
                | (usize::from(m.option) << 2)
                | (usize::from(m.command) << 3)
        }
        8 => usize::from(
            s.engine.expression_mode() || s.engine.question_mode() || s.engine.raw_mode(),
        ),
        _ => 0,
    }
}
