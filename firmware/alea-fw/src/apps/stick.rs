//! 棒倒し（DESIGN.md §10.3）。振り終わりで棒を倒し、倒れた向きを表示する。
//!
//! 確定デザイン（キャンバス 4 段目）：方式の札「左右／八方位」（ボタン B で切替・今の方式を反転）。
//! 八方位は方位盤と倒れた棒・方位名（和英）、左右は地面と倒れた棒・左右（和英）。待機中は「?」。
//! 画像部品は `tools/render_m4.py` が生成する。抽選は `alea_core::stick`。

use alea_core::stick::{self, Fall, Mode};
use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::art;

/// 棒倒し。
pub struct StickApp {
    mode: Mode,
    fall: Option<Fall>,
}

impl StickApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            mode: Mode::Eight,
            fall: None,
        }
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        let at = Point::zero();
        art::draw(&mut canvas, &art::STICK_FRAME, at);
        let (x, y, w, h) = art::STICK_CHIPS[usize::from(self.mode == Mode::Eight)];
        canvas.invert_rect(x + 1, y + 1, w - 2, h - 2);
        match (self.mode, self.fall) {
            (Mode::Eight, Some(Fall::Dir(d))) => {
                let d = usize::from(d % 8);
                art::draw(&mut canvas, &art::STICK_COMPASS, at);
                art::draw(&mut canvas, &art::STICK_DIR[d], at);
                art::draw(&mut canvas, &art::STICK_DIR_JA[d], at);
                art::draw(&mut canvas, &art::STICK_DIR_EN[d], at);
            }
            (Mode::Eight, _) => {
                art::draw(&mut canvas, &art::STICK_COMPASS, at);
                art::draw(&mut canvas, &art::STICK_PIVOT, at);
                art::draw(&mut canvas, &art::STICK_EIGHT_WAIT, at);
            }
            (Mode::LeftRight, Some(Fall::Side(right))) => {
                let i = usize::from(right);
                art::draw(&mut canvas, &art::STICK_GROUND, at);
                art::draw(&mut canvas, &art::STICK_SIDE[i], at);
                art::draw(&mut canvas, &art::STICK_SIDE_JA[i], at);
                art::draw(&mut canvas, &art::STICK_SIDE_EN[i], at);
            }
            (Mode::LeftRight, _) => {
                art::draw(&mut canvas, &art::STICK_GROUND, at);
                art::draw(&mut canvas, &art::STICK_STAND, at);
                art::draw(&mut canvas, &art::STICK_SIDE_WAIT, at);
            }
        }
    }
}

impl App for StickApp {
    fn id(&self) -> &'static str {
        "stick"
    }

    fn title(&self) -> &'static str {
        "Stick"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.mode = Mode::Eight;
        self.fall = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ShakeEnd => {
                let f = stick::fall(ctx.rng(), self.mode);
                println!("[stick] {:?}", f);
                self.fall = Some(f);
            }
            Event::ButtonB => {
                self.mode = match self.mode {
                    Mode::Eight => Mode::LeftRight,
                    Mode::LeftRight => Mode::Eight,
                };
                self.fall = None;
                println!("[stick] mode {:?}", self.mode);
            }
            _ => return Action::None,
        }
        self.draw(ctx);
        ctx.present(Refresh::Partial).await;
        Action::None
    }
}
