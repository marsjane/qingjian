//! Rust/C++ ABI：指针由创建者释放，文本借用到下一次写操作为止。
#pragma once
#include <cstddef>
#include <cstdint>
extern "C" {
struct QjRuntime;
struct QjSession;
QjRuntime *qj_runtime_new(const char *, const char *, const char *);
void qj_runtime_free(QjRuntime *);
QjSession *qj_session_new(const QjRuntime *);
void qj_session_free(QjSession *);
bool qj_key(QjSession *, uint32_t, uint32_t, uint32_t, size_t);
bool qj_select(QjSession *, size_t, int32_t);
void qj_reset(QjSession *);
void qj_private(QjSession *, bool);
void qj_annotate(QjSession *, size_t, size_t);
const char *qj_text(const QjSession *, uint32_t, size_t);
size_t qj_value(const QjSession *, uint32_t);
}
