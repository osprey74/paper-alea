//! おみくじ（DESIGN.md §10.5）。振り終わりで運勢と一言を引き、縦書きの紙片に表示する。
//!
//! 確定デザイン（キャンバス 4 段目）：二重枠の紙片に「第N番」・運勢・一言（縦書き）。待機中は「?」。
//! 運勢・重み・一言は `tools/data/omikuji.json`（Alea のオリジナル文）から `tools/render_m4.py` が
//! 画像部品と表にしてファームに組み込む（microSD 不要）。抽選は `alea_core::omikuji`。

use alea_core::omikuji::{self, Pick};
use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::art;

/// おみくじ。
pub struct OmikujiApp {
    pick: Option<Pick>,
}

impl OmikujiApp {
    /// 作る。
    pub const fn new() -> Self {
        Self { pick: None }
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        let at = Point::zero();
        art::draw(&mut canvas, &art::OMIKUJI_FRAME, at);
        let Some(p) = self.pick else {
            art::draw(&mut canvas, &art::OMIKUJI_WAIT, at);
            return;
        };
        // 一言は運勢の順の通し番号で並んでいる。
        let index = serial(p);
        art::draw(&mut canvas, &art::OMIKUJI_NUMBER[index], at);
        art::draw(&mut canvas, &art::OMIKUJI_FORTUNE[p.fortune], at);
        art::draw(&mut canvas, &art::OMIKUJI_MESSAGE[index], at);
    }
}

/// 一言の通し番号（0 始まり）。
fn serial(p: Pick) -> usize {
    let before: u32 = art::OMIKUJI_COUNTS[..p.fortune].iter().sum();
    before as usize + p.message
}

impl App for OmikujiApp {
    fn id(&self) -> &'static str {
        "omikuji"
    }

    fn title(&self) -> &'static str {
        "Omikuji"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.pick = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if e == Event::ShakeEnd {
            let p = omikuji::pick(ctx.rng(), &art::OMIKUJI_WEIGHTS, &art::OMIKUJI_COUNTS);
            println!(
                "[omikuji] no.{} {}",
                serial(p) + 1,
                art::OMIKUJI_LABELS[p.fortune]
            );
            self.pick = Some(p);
            self.draw(ctx);
            ctx.present(Refresh::Partial).await;
        }
        Action::None
    }
}
