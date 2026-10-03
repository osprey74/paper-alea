//! 是か非か（DESIGN.md §10.6）。振り終わりで YES か NO を大きく表示する。
//!
//! 確定デザイン（書物調）の台紙に「YES／是」「NO／非」を重ねる。待機中は「?」。
//! 画像部品は `tools/render_apps.py` が生成する。

use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::art;

/// 是か非か。
pub struct YesNoApp {
    answer: Option<bool>,
}

impl YesNoApp {
    /// 作る。
    pub const fn new() -> Self {
        Self { answer: None }
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        let parts: &[&art::Sprite] = match self.answer {
            None => &[&art::YESNO_FRAME, &art::YESNO_WAIT],
            Some(true) => &[&art::YESNO_FRAME, &art::YESNO_YES, &art::YESNO_KANJI_YES],
            Some(false) => &[&art::YESNO_FRAME, &art::YESNO_NO, &art::YESNO_KANJI_NO],
        };
        for s in parts {
            art::draw(&mut canvas, s, Point::zero());
        }
    }
}

impl App for YesNoApp {
    fn id(&self) -> &'static str {
        "yesno"
    }

    fn title(&self) -> &'static str {
        "Yes / No"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.answer = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if e == Event::ShakeEnd {
            let yes = ctx.rng().bernoulli();
            self.answer = Some(yes);
            println!("[yesno] {}", if yes { "YES" } else { "NO" });
            self.draw(ctx);
            ctx.present(Refresh::Partial).await;
        }
        Action::None
    }
}
