//! HAL 層。PaperMono 固有の初期化とドライバ（Nostos・papermono-rs 由来）。
//!
//! アプリはここを直接呼ばない（`core/` の公開 API だけを使う）。

pub mod ioe;
pub mod panel;
