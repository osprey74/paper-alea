//! コイントス（DESIGN.md §10.2）。振り終わりで表（HEADS）か裏（TAILS）を決める。
//!
//! 確定デザイン（書物調）の台紙に、コインの面と「HEADS／表」「TAILS／裏」を重ねる。
//! 表は二重円に「A」（Alea の頭文字）、裏は二重円に放射状の線と菱形。待機中は二重円に「?」。
//! 画像部品は `tools/render_apps.py` が生成する。

use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::art;

/// コイントス。
pub struct CoinApp {
    heads: Option<bool>,
}

impl CoinApp {
    /// 作る。
    pub const fn new() -> Self {
        Self { heads: None }
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        let parts: &[&art::Sprite] = match self.heads {
            None => &[&art::COIN_FRAME, &art::COIN_WAIT],
            Some(true) => &[
                &art::COIN_FRAME,
                &art::COIN_HEADS,
                &art::COIN_WORD_HEADS,
                &art::COIN_KANJI_HEADS,
            ],
            Some(false) => &[
                &art::COIN_FRAME,
                &art::COIN_TAILS,
                &art::COIN_WORD_TAILS,
                &art::COIN_KANJI_TAILS,
            ],
        };
        for s in parts {
            art::draw(&mut canvas, s, Point::zero());
        }
    }
}

impl App for CoinApp {
    fn id(&self) -> &'static str {
        "coin"
    }

    fn title(&self) -> &'static str {
        "Coin"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.heads = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if e == Event::ShakeEnd {
            let heads = ctx.rng().bernoulli();
            self.heads = Some(heads);
            println!("[coin] {}", if heads { "HEADS" } else { "TAILS" });
            self.draw(ctx);
            ctx.present(Refresh::Partial).await;
        }
        Action::None
    }
}
