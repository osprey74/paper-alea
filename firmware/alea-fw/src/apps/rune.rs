//! ルーン（DESIGN.md §10.8）。エルダー・フサルク 24 文字から 1 つを引く（ルーン文字を用いた現代の占い）。
//!
//! 確定デザイン（キャンバス 5 段目）：逆位置の札「正位置のみ／逆位置あり」（ボタン B で切替・既定は正位置のみ）。
//! 石に刻んだ字形（逆位置は 180° 回転）・名前（英字とカタカナ）・キーワード・文。待機中は「?」。
//! 点対称の字形は逆位置にしない。部品は `tools/render_m5.py`、抽選は `alea_core::rune`。

use alea_core::rune::{self, Draw};
use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::art;
use crate::ui::layout::SCREEN_W;

/// ルーン。
pub struct RuneApp {
    allow_reversed: bool,
    draw: Option<Draw>,
}

impl RuneApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            allow_reversed: false,
            draw: None,
        }
    }

    fn render(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        let at = Point::zero();
        art::draw(&mut canvas, &art::RUNE_FRAME, at);
        let (x, y, w, h) = art::RUNE_CHIPS[usize::from(self.allow_reversed)];
        canvas.invert_rect(x + 1, y + 1, w - 2, h - 2);
        let Some(d) = self.draw else {
            art::draw(&mut canvas, &art::RUNE_WAIT, at);
            return;
        };
        let i = usize::from(d.rune);
        let (glyph, words, text) = if d.reversed {
            (
                &art::RUNE_GLYPH_REV,
                &art::RUNE_WORDS_REV,
                &art::RUNE_TEXT_REV,
            )
        } else {
            (&art::RUNE_GLYPH, &art::RUNE_WORDS_UP, &art::RUNE_TEXT_UP)
        };
        art::draw(&mut canvas, &glyph[i], at);
        art::draw(&mut canvas, &art::RUNE_NAME[i], at);
        art::draw(&mut canvas, &words[i], at);
        art::draw(&mut canvas, &text[i], at);
        // カタカナ名（逆位置なら右に「逆位置」の札を並べ、組で中央にそろえる）。
        let kana = &art::RUNE_KANA[i];
        let cx = SCREEN_W / 2;
        if d.reversed {
            let chip = &art::RUNE_REV_CHIP;
            let (kw, cw) = (i32::from(kana.w), i32::from(chip.w));
            let total = kw + art::RUNE_KANA_GAP + cw;
            let left = cx - total / 2;
            art::draw(
                &mut canvas,
                kana,
                Point::new(left + kw / 2, art::RUNE_KANA_Y),
            );
            art::draw(
                &mut canvas,
                chip,
                Point::new(left + total - cw / 2, art::RUNE_KANA_Y),
            );
        } else {
            art::draw(&mut canvas, kana, Point::new(cx, art::RUNE_KANA_Y));
        }
    }
}

impl App for RuneApp {
    fn id(&self) -> &'static str {
        "rune"
    }

    fn title(&self) -> &'static str {
        "Rune"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.allow_reversed = false;
        self.draw = None;
        self.render(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ShakeEnd => {
                let d = rune::draw(ctx.rng(), self.allow_reversed, &art::RUNE_SYMMETRIC);
                println!(
                    "[rune] {}{}",
                    art::RUNE_NAMES[usize::from(d.rune)],
                    if d.reversed { " reversed" } else { "" }
                );
                self.draw = Some(d);
            }
            Event::ButtonB => {
                self.allow_reversed = !self.allow_reversed;
                self.draw = None;
                println!("[rune] allow_reversed={}", self.allow_reversed);
            }
            _ => return Action::None,
        }
        self.render(ctx);
        ctx.present(Refresh::Partial).await;
        Action::None
    }
}
