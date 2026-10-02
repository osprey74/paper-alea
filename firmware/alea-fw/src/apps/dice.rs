//! ダイス（DESIGN.md §10.1）。
//!
//! - 画面上部に 10 種の選択チップ（5 列×2 段・各 96×64px）。タップで切替（初期値 1D6、アプリ終了まで保持）。
//! - 振り終わりで振る。各ダイスの出目を表示し、複数個なら合計も大きく表示する。
//! - D6 は目（ピップ）、それ以外は多角形の輪郭と数字。1D100 は十の位と一の位の D10 を並べる。
//! 抽選は `alea_core::dice`（ホストで分布テスト済み）。

use core::fmt::Write as _;

use alea_core::dice::{self, DiceKind, Roll};
use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{
    Circle, Polyline, PrimitiveStyle, Rectangle, RoundedRectangle,
};
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::{Canvas, Refresh};
use crate::ui::layout::{HEADER_H, SCREEN_W};
use crate::ui::widgets;
use crate::ui::FmtBuf;

/// チップの列数と大きさ（DESIGN.md §9 の最小 64px 角を守る）。
const CHIP_COLS: i32 = 5;
const CHIP_W: i32 = SCREEN_W / CHIP_COLS;
const CHIP_H: i32 = 64;
/// チップ領域の下端。
const CHIPS_BOTTOM: i32 = HEADER_H + 2 * CHIP_H;
/// ダイスを並べる高さ（中心）。
const DICE_Y: i32 = 420;
/// 合計を出す高さ（中心）。
const TOTAL_Y: i32 = 660;

/// 単位多角形（半径 1000）。
const TRIANGLE: [(i32, i32); 3] = [(0, -1000), (866, 500), (-866, 500)];
const DIAMOND: [(i32, i32); 4] = [(0, -1000), (1000, 0), (0, 1000), (-1000, 0)];
const KITE: [(i32, i32); 4] = [(0, -1000), (900, -150), (0, 1000), (-900, -150)];
const PENTAGON: [(i32, i32); 5] = [(0, -1000), (951, -309), (588, 809), (-588, 809), (-951, -309)];
const HEXAGON: [(i32, i32); 6] = [
    (0, -1000),
    (866, -500),
    (866, 500),
    (0, 1000),
    (-866, 500),
    (-866, -500),
];

/// ダイス。
pub struct DiceApp {
    kind: DiceKind,
    roll: Option<Roll>,
}

impl DiceApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            kind: DiceKind::D6,
            roll: None,
        }
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        widgets::header(&mut canvas, "Dice", true);
        draw_chips(&mut canvas, self.kind);
        widgets::big_text(
            &mut canvas,
            "Shake to roll",
            Point::new(SCREEN_W / 2, CHIPS_BOTTOM + 36),
            1,
            Gray2::BLACK,
        );
        let Some(r) = self.roll else {
            widgets::big_text(&mut canvas, "?", Point::new(SCREEN_W / 2, DICE_Y), 12, Gray2::BLACK);
            return;
        };
        let mut label = FmtBuf::<8>::new();
        if r.kind == DiceKind::D100 {
            // 十の位（00〜90）と一の位（0〜9）。
            let _ = write!(label, "{:02}", r.faces[0]);
            draw_die(&mut canvas, Point::new(130, DICE_Y), 200, 10, label.as_str());
            label.clear();
            let _ = write!(label, "{}", r.faces[1]);
            draw_die(&mut canvas, Point::new(350, DICE_Y), 200, 10, label.as_str());
        } else {
            let (count, sides) = r.kind.count_and_sides();
            let size = match count {
                1 => 240,
                2 => 180,
                _ => 130,
            };
            let step = SCREEN_W / count as i32;
            for i in 0..count {
                let cx = step / 2 + i as i32 * step;
                let center = Point::new(cx, DICE_Y);
                if sides == 6 {
                    draw_d6(&mut canvas, center, size, r.faces[i]);
                } else {
                    label.clear();
                    let _ = write!(label, "{}", r.faces[i]);
                    draw_die(&mut canvas, center, size, sides, label.as_str());
                }
            }
        }
        if r.count > 1 {
            label.clear();
            let _ = write!(label, "{}", r.total);
            widgets::big_text(&mut canvas, "total", Point::new(SCREEN_W / 2, TOTAL_Y - 70), 2, Gray2::BLACK);
            widgets::big_text(&mut canvas, label.as_str(), Point::new(SCREEN_W / 2, TOTAL_Y + 20), 5, Gray2::BLACK);
        }
    }
}

/// 選択チップ（選択中は黒地に白文字）。
fn draw_chips(canvas: &mut Canvas<'_>, selected: DiceKind) {
    for (i, kind) in DiceKind::ALL.iter().enumerate() {
        let i = i as i32;
        let tl = Point::new((i % CHIP_COLS) * CHIP_W + 4, HEADER_H + (i / CHIP_COLS) * CHIP_H + 4);
        let rect = Rectangle::new(tl, Size::new((CHIP_W - 8) as u32, (CHIP_H - 8) as u32));
        let on = *kind == selected;
        let style = if on {
            PrimitiveStyle::with_fill(Gray2::BLACK)
        } else {
            PrimitiveStyle::with_stroke(Gray2::BLACK, 2)
        };
        let _ = rect.into_styled(style).draw(canvas);
        let color = if on { Gray2::WHITE } else { Gray2::BLACK };
        widgets::big_text(canvas, kind.label(), rect.center(), 1, color);
    }
}

/// タップ座標にあるチップ。
fn chip_at(x: i32, y: i32) -> Option<DiceKind> {
    if !(HEADER_H..CHIPS_BOTTOM).contains(&y) {
        return None;
    }
    let i = ((y - HEADER_H) / CHIP_H * CHIP_COLS + x / CHIP_W) as usize;
    DiceKind::ALL.get(i).copied()
}

/// D6：角丸の正方形と目。
fn draw_d6(canvas: &mut Canvas<'_>, center: Point, size: i32, face: u32) {
    let half = size / 2;
    let body = Rectangle::new(center - Point::new(half, half), Size::new(size as u32, size as u32));
    let _ = RoundedRectangle::with_equal_corners(body, Size::new(size as u32 / 8, size as u32 / 8))
        .into_styled(PrimitiveStyle::with_stroke(Gray2::BLACK, 5))
        .draw(canvas);
    let q = size / 4;
    let pip = |c: &mut Canvas<'_>, dx: i32, dy: i32| {
        let _ = Circle::with_center(center + Point::new(dx * q, dy * q), (size / 6) as u32)
            .into_styled(PrimitiveStyle::with_fill(Gray2::BLACK))
            .draw(c);
    };
    let pips: &[(i32, i32)] = match face {
        1 => &[(0, 0)],
        2 => &[(-1, -1), (1, 1)],
        3 => &[(-1, -1), (0, 0), (1, 1)],
        4 => &[(-1, -1), (1, -1), (-1, 1), (1, 1)],
        5 => &[(-1, -1), (1, -1), (0, 0), (-1, 1), (1, 1)],
        _ => &[(-1, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (1, 1)],
    };
    for &(dx, dy) in pips {
        pip(canvas, dx, dy);
    }
}

/// D6 以外：面数に応じた多角形の輪郭と数字。
fn draw_die(canvas: &mut Canvas<'_>, center: Point, size: i32, sides: u32, label: &str) {
    let shape: &[(i32, i32)] = match sides {
        3 | 4 => &TRIANGLE,
        8 => &DIAMOND,
        10 => &KITE,
        12 => &PENTAGON,
        _ => &HEXAGON,
    };
    let r = size / 2;
    let mut points = [Point::zero(); 7];
    for (p, &(ux, uy)) in points.iter_mut().zip(shape) {
        *p = center + Point::new(ux * r / 1000, uy * r / 1000);
    }
    points[shape.len()] = points[0];
    let _ = Polyline::new(&points[..=shape.len()])
        .into_styled(PrimitiveStyle::with_stroke(Gray2::BLACK, 5))
        .draw(canvas);
    // 三角形は重心が下寄りなので数字も下げる。
    let text_center = if shape.len() == 3 {
        center + Point::new(0, r / 6)
    } else {
        center
    };
    let scale = if size >= 200 { 5 } else { 3 };
    widgets::big_text(canvas, label, text_center, scale, Gray2::BLACK);
}

impl App for DiceApp {
    fn id(&self) -> &'static str {
        "dice"
    }

    fn title(&self) -> &'static str {
        "Dice"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.kind = DiceKind::D6;
        self.roll = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ShakeEnd => {
                let r = dice::roll(ctx.rng(), self.kind);
                println!(
                    "[dice] {} faces={:?} total={}",
                    r.kind.label(),
                    &r.faces[..r.count],
                    r.total
                );
                self.roll = Some(r);
                self.draw(ctx);
                ctx.present(Refresh::Partial).await;
            }
            Event::Tap { x, y } => {
                if let Some(kind) = chip_at(i32::from(x), i32::from(y)) {
                    if kind != self.kind {
                        println!("[dice] select {}", kind.label());
                        self.kind = kind;
                        self.roll = None;
                        self.draw(ctx);
                        ctx.present(Refresh::Partial).await;
                    }
                }
            }
            _ => {}
        }
        Action::None
    }
}
