//! あみだくじ（DESIGN.md §10.4）。
//!
//! 確定デザイン（キャンバス 4 段目）：上にローマ数字の札、下に番号 1〜n。上の札（または縦線の列）を
//! タップすると、その道筋を太線で示し、行き着いた番号を反転する（1 回の部分更新・辿るアニメーションなし）。
//! - ボタン B：本数を 2〜6 で循環し、引き直す。振る：同じ本数で引き直す。
//! - 縦線・横線・道筋は実機で描く。案内・ローマ数字・番号は `tools/render_m4.py` の画像部品。
//! 横線の生成と道筋は `alea_core::amida`。

use alea_core::amida::{self, Ladder, LEVELS, MAX_LINES, MIN_LINES};
use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::{Canvas, Refresh};
use crate::ui::art;
use crate::ui::layout::SCREEN_W;

/// 起動時の本数。
const DEFAULT_LINES: usize = 5;
/// 横線の段の上下の余白 [px]。
const LEVEL_MARGIN: i32 = 20;
/// 道筋の太さ [px]。
const PATH_W: u32 = 8;

/// あみだくじ。
pub struct AmidaApp {
    ladder: Option<Ladder>,
    selected: Option<usize>,
}

impl AmidaApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            ladder: None,
            selected: None,
        }
    }

    fn lines(&self) -> usize {
        self.ladder.map_or(DEFAULT_LINES, |l| l.lines)
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        art::draw(&mut canvas, &art::AMIDA_FRAME, Point::zero());
        let Some(ladder) = self.ladder else { return };
        let n = ladder.lines;
        art::draw(&mut canvas, &art::AMIDA_NOTE[n - MIN_LINES], Point::zero());

        let thin = PrimitiveStyle::with_stroke(Gray2::BLACK, 2);
        let half_w = art::AMIDA_BOX_W / 2;
        for i in 0..n {
            let x = line_x(n, i);
            // 上の札。
            let _ = Rectangle::new(
                Point::new(x - half_w, art::AMIDA_BOX_TOP),
                Size::new(art::AMIDA_BOX_W as u32, art::AMIDA_BOX_H as u32),
            )
            .into_styled(PrimitiveStyle::with_stroke(Gray2::BLACK, 1))
            .draw(&mut canvas);
            let box_c = Point::new(x, art::AMIDA_BOX_TOP + art::AMIDA_BOX_H / 2);
            art::draw(&mut canvas, &art::AMIDA_ROMAN[i], box_c);
            // 縦線と下の番号。
            let _ = Line::new(
                Point::new(x, art::AMIDA_LADDER_TOP),
                Point::new(x, art::AMIDA_LADDER_BOTTOM),
            )
            .into_styled(thin)
            .draw(&mut canvas);
            let goal_c = Point::new(x, art::AMIDA_GOAL_TOP + art::AMIDA_GOAL_H / 2);
            art::draw(&mut canvas, &art::AMIDA_GOAL[i], goal_c);
        }
        for (k, row) in ladder.rungs.iter().enumerate() {
            for (g, _) in row.iter().enumerate().take(n - 1).filter(|(_, &r)| r) {
                let y = level_y(k);
                let _ = Line::new(Point::new(line_x(n, g), y), Point::new(line_x(n, g + 1), y))
                    .into_styled(thin)
                    .draw(&mut canvas);
            }
        }

        let Some(start) = self.selected else { return };
        let (end, path) = ladder.trace(start);
        draw_path(&mut canvas, n, start, &path);
        let x = line_x(n, start);
        canvas.invert_rect(
            x - half_w + 1,
            art::AMIDA_BOX_TOP + 1,
            art::AMIDA_BOX_W - 2,
            art::AMIDA_BOX_H - 2,
        );
        let x = line_x(n, end);
        canvas.invert_rect(
            x - half_w,
            art::AMIDA_GOAL_TOP,
            art::AMIDA_BOX_W,
            art::AMIDA_GOAL_H,
        );
    }

    fn regenerate(&mut self, ctx: &mut Ctx, lines: usize) {
        self.ladder = Some(amida::generate(ctx.rng(), lines));
        self.selected = None;
        println!("[amida] new ladder lines={}", lines);
    }
}

/// 縦線 `i` の x 座標（`n` 本を画面中央にそろえる）。
fn line_x(n: usize, i: usize) -> i32 {
    let spacing = spacing(n);
    SCREEN_W / 2 + (2 * i as i32 - (n as i32 - 1)) * spacing / 2
}

fn spacing(n: usize) -> i32 {
    (art::AMIDA_SPAN / (n as i32 - 1)).min(art::AMIDA_MAX_SPACING)
}

/// 段 `k` の y 座標。
fn level_y(k: usize) -> i32 {
    let span = art::AMIDA_LADDER_BOTTOM - art::AMIDA_LADDER_TOP - 2 * LEVEL_MARGIN;
    art::AMIDA_LADDER_TOP + LEVEL_MARGIN + k as i32 * span / (LEVELS as i32 - 1)
}

/// 道筋を太線で描く（角は丸める）。
fn draw_path(canvas: &mut Canvas<'_>, n: usize, start: usize, path: &[usize; LEVELS]) {
    let style = PrimitiveStyle::with_stroke(Gray2::BLACK, PATH_W);
    let joint = PrimitiveStyle::with_fill(Gray2::BLACK);
    let mut points = [Point::zero(); 2 * LEVELS + 2];
    let mut len = 0;
    let mut push = |p: Point| {
        points[len] = p;
        len += 1;
    };
    let mut pos = start;
    push(Point::new(line_x(n, pos), art::AMIDA_LADDER_TOP));
    for (k, &next) in path.iter().enumerate() {
        if next != pos {
            let y = level_y(k);
            push(Point::new(line_x(n, pos), y));
            push(Point::new(line_x(n, next), y));
            pos = next;
        }
    }
    push(Point::new(line_x(n, pos), art::AMIDA_LADDER_BOTTOM));
    for pair in points[..len].windows(2) {
        let _ = Line::new(pair[0], pair[1]).into_styled(style).draw(canvas);
    }
    for &p in &points[..len] {
        let _ = Circle::with_center(p, PATH_W)
            .into_styled(joint)
            .draw(canvas);
    }
}

/// タップ座標にある縦線の列（上の札から縦線の下端まで）。
fn column_at(n: usize, x: i32, y: i32) -> Option<usize> {
    if !(art::AMIDA_BOX_TOP..=art::AMIDA_LADDER_BOTTOM).contains(&y) {
        return None;
    }
    let half = spacing(n) / 2;
    (0..n).find(|&i| (x - line_x(n, i)).abs() <= half)
}

impl App for AmidaApp {
    fn id(&self) -> &'static str {
        "amida"
    }

    fn title(&self) -> &'static str {
        "Amida"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.regenerate(ctx, DEFAULT_LINES);
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ShakeEnd => self.regenerate(ctx, self.lines()),
            Event::ButtonB => {
                let n = self.lines();
                let next = if n >= MAX_LINES { MIN_LINES } else { n + 1 };
                self.regenerate(ctx, next);
            }
            Event::Tap { x, y } => {
                let Some(ladder) = self.ladder else {
                    return Action::None;
                };
                let Some(col) = column_at(ladder.lines, i32::from(x), i32::from(y)) else {
                    return Action::None;
                };
                if self.selected == Some(col) {
                    return Action::None;
                }
                self.selected = Some(col);
                println!(
                    "[amida] start {} -> goal {}",
                    col + 1,
                    ladder.trace(col).0 + 1
                );
            }
            _ => return Action::None,
        }
        self.draw(ctx);
        ctx.present(Refresh::Partial).await;
        Action::None
    }
}
