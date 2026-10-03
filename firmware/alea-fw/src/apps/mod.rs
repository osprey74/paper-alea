//! ミニアプリ。各アプリは [`crate::services::app::App`] を実装し、
//! [`crate::app_manager`] の登録表（`AnyApp`）に加える。

pub mod coin;
pub mod debug_refresh;
pub mod debug_sd;
pub mod dice;
pub mod launcher;
pub mod tarot;
pub mod yesno;
