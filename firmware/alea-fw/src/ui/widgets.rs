//! 見出し帯・本文の行（開発用の画面で使う）。

use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};

use super::layout::{HEADER_H, LINE_H, MARGIN_X, SCREEN_W};
use crate::services::display::Canvas;

/// 見出し帯（黒地に白文字）。`back_hint` が true なら右端に「A: back」を出す。
pub fn header(canvas: &mut Canvas<'_>, title: &str, back_hint: bool) {
    let _ = Rectangle::new(Point::zero(), Size::new(SCREEN_W as u32, HEADER_H as u32))
        .into_styled(PrimitiveStyle::with_fill(Gray2::BLACK))
        .draw(canvas);
    let white = MonoTextStyle::new(&FONT_10X20, Gray2::WHITE);
    let middle = TextStyleBuilder::new().baseline(Baseline::Middle).build();
    let _ = Text::with_text_style(title, Point::new(MARGIN_X, HEADER_H / 2), white, middle)
        .draw(canvas);
    if back_hint {
        let right = TextStyleBuilder::new()
            .baseline(Baseline::Middle)
            .alignment(Alignment::Right)
            .build();
        let _ = Text::with_text_style(
            "A: back",
            Point::new(SCREEN_W - MARGIN_X, HEADER_H / 2),
            white,
            right,
        )
        .draw(canvas);
    }
}

/// 本文の行カーソル。見出し帯の下から 1 行ずつ書く。
pub struct Lines {
    y: i32,
}

impl Lines {
    /// 見出し帯の下から始める。
    pub fn below_header() -> Self {
        Self {
            y: HEADER_H + LINE_H + 8,
        }
    }

    /// 1 行書いて次の行へ進む。
    pub fn put(&mut self, canvas: &mut Canvas<'_>, s: &str) {
        let style = MonoTextStyle::new(&FONT_10X20, Gray2::BLACK);
        let _ = Text::new(s, Point::new(MARGIN_X, self.y), style).draw(canvas);
        self.y += LINE_H;
    }

    /// 空行を入れる。
    pub fn gap(&mut self, px: i32) {
        self.y += px;
    }
}
