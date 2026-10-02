//! RefreshTestApp（`debug_refresh`・HANDOFF T7）。
//!
//! - 描画回数と部分更新回数を表示する（11 回目の部分描画が全面になることの確認）。
//! - タップするたびに部分更新で描き直し、タップ位置に十字と座標を描く（T4 の確認を兼ねる）。
//! - ボタン B で明示的な全面更新。

use core::fmt::Write as _;

use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line, PrimitiveStyle, Rectangle};
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::widgets::{self, Lines};
use crate::ui::FmtBuf;

/// 部分更新の上限（表示用）。
const MAX_PARTIAL: i32 = 10;

/// 画面書き換えテスト。
pub struct RefreshTestApp {
    draws: u32,
    last_tap: Option<(i32, i32)>,
}

impl RefreshTestApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            draws: 0,
            last_tap: None,
        }
    }

    fn draw(&self, ctx: &mut Ctx) {
        let partials = i32::from(ctx.partial_count());
        ctx.clear();
        let mut canvas = ctx.canvas();
        widgets::header(&mut canvas, "Refresh test", true);
        let mut lines = Lines::below_header();
        let mut line = FmtBuf::<48>::new();
        lines.put(&mut canvas, "Tap: partial   B: full refresh");
        lines.gap(8);
        let _ = write!(line, "draws: {}", self.draws);
        lines.put(&mut canvas, line.as_str());
        line.clear();
        let _ = write!(line, "partials before this draw: {}/{}", partials, MAX_PARTIAL);
        lines.put(&mut canvas, line.as_str());
        line.clear();
        match self.last_tap {
            Some((x, y)) => {
                let _ = write!(line, "last tap: ({}, {})", x, y);
            }
            None => {
                let _ = write!(line, "last tap: -");
            }
        }
        lines.put(&mut canvas, line.as_str());

        // 部分更新回数を 10 個の枠で示す（塗り = 済み）。
        for i in 0..MAX_PARTIAL {
            let style = if i < partials {
                PrimitiveStyle::with_fill(Gray2::BLACK)
            } else {
                PrimitiveStyle::with_stroke(Gray2::BLACK, 3)
            };
            let _ = Rectangle::new(Point::new(24 + i * 44, 250), Size::new(36, 36))
                .into_styled(style)
                .draw(&mut canvas);
        }

        if let Some((x, y)) = self.last_tap {
            let thick = PrimitiveStyle::with_stroke(Gray2::BLACK, 3);
            let _ = Line::new(Point::new(x - 24, y), Point::new(x + 24, y))
                .into_styled(thick)
                .draw(&mut canvas);
            let _ = Line::new(Point::new(x, y - 24), Point::new(x, y + 24))
                .into_styled(thick)
                .draw(&mut canvas);
        }
    }
}

impl App for RefreshTestApp {
    fn id(&self) -> &'static str {
        "debug_refresh"
    }

    fn title(&self) -> &'static str {
        "Refresh test"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.draws = 0;
        self.last_tap = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::Tap { x, y } => {
                self.draws += 1;
                self.last_tap = Some((i32::from(x), i32::from(y)));
                println!("[debug_refresh] tap x={} y={} draw #{}", x, y, self.draws);
                self.draw(ctx);
                ctx.present(Refresh::Partial).await;
            }
            Event::ButtonB => {
                self.draws += 1;
                println!("[debug_refresh] full refresh draw #{}", self.draws);
                self.draw(ctx);
                ctx.full_refresh().await;
            }
            _ => {}
        }
        Action::None
    }
}
