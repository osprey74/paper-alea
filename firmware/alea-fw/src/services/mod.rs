//! Core Services（DESIGN.md §6）。標準の `core` クレートと衝突しないよう `services` と呼ぶ。アプリはここの公開 API だけを使う。

pub mod app;
pub mod display;
pub mod input;
pub mod rng;
pub mod shake;
pub mod storage;
