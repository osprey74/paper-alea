//! コイントス（DESIGN.md §10.2）。振り終わりで表（HEADS）か裏（TAILS）を決める。
//!
//! 意匠（線画）：表は二重円に「A」（Alea の頭文字）、裏は二重円に放射状の線。

use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle};
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::layout::SCREEN_W;
use crate::ui::widgets::{self, Lines};

/// コインの中心。
const COIN_CENTER: Point = Point::new(SCREEN_W / 2, 400);
/// コインの直径 [px]。
const COIN_D: u32 = 320;

/// 裏面の放射線（中心からの単位ベクトル×1000・30° おき）。
const RAYS: [(i32, i32); 12] = [
    (1000, 0),
    (866, 500),
    (500, 866),
    (0, 1000),
    (-500, 866),
    (-866, 500),
    (-1000, 0),
    (-866, -500),
    (-500, -866),
    (0, -1000),
    (500, -866),
    (866, -500),
];

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
        widgets::header(&mut canvas, "Coin", true);
        let mut lines = Lines::below_header();
        lines.put(&mut canvas, "Shake to toss");

        let Some(heads) = self.heads else {
            widgets::big_text(&mut canvas, "?", COIN_CENTER, 12, Gray2::BLACK);
            return;
        };
        let thick = PrimitiveStyle::with_stroke(Gray2::BLACK, 6);
        let thin = PrimitiveStyle::with_stroke(Gray2::BLACK, 3);
        let _ = Circle::with_center(COIN_CENTER, COIN_D)
            .into_styled(thick)
            .draw(&mut canvas);
        let _ = Circle::with_center(COIN_CENTER, COIN_D - 40)
            .into_styled(thin)
            .draw(&mut canvas);
        if heads {
            widgets::big_text(&mut canvas, "A", COIN_CENTER, 9, Gray2::BLACK);
        } else {
            let (r_in, r_out) = (40, (COIN_D as i32 - 40) / 2 - 16);
            for (dx, dy) in RAYS {
                let from = COIN_CENTER + Point::new(dx * r_in / 1000, dy * r_in / 1000);
                let to = COIN_CENTER + Point::new(dx * r_out / 1000, dy * r_out / 1000);
                let _ = Line::new(from, to).into_styled(thin).draw(&mut canvas);
            }
            let _ = Circle::with_center(COIN_CENTER, 60)
                .into_styled(PrimitiveStyle::with_fill(Gray2::BLACK))
                .draw(&mut canvas);
        }
        let label = if heads { "HEADS" } else { "TAILS" };
        widgets::big_text(&mut canvas, label, Point::new(SCREEN_W / 2, 680), 5, Gray2::BLACK);
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
