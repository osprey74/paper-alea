//! 易・三枚硬貨法（DESIGN.md §10.7）。
//!
//! 確定デザイン（キャンバス 5 段目）：
//! - 途中：振るたびに硬貨 3 枚で 1 本の爻を立て、下から積む（毎回モノクロの部分更新）。
//!   爻の横に硬貨の表裏、変爻には〇（老陽）か×（老陰）。
//! - 結果：6 本目を積んだ画面を部分更新で見せ、[`RESULT_DELAY_MS`] 後に全面更新で結果へ移る。本卦（変爻の印付き）→ 之卦、卦名・番号と読み・変爻の位置・解釈。
//!   変爻が無ければ本卦だけを中央に置く。結果の画面で振ると、最初からやり直す。
//! 卦画（爻の棒）は実機で描き、文字は `tools/render_m5.py` の画像部品。抽選は `alea_core::iching`。

use alea_core::iching::{self, Line, LINES};
use embassy_time::Timer;
use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Line as Segment, PrimitiveStyle, Rectangle};
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::{Canvas, Refresh};
use crate::ui::art;
use crate::ui::layout::SCREEN_W;

/// 6 本目を積んだ画面を見せておく時間 [ms]（6 本目の表裏が分かるように・2026-10-03 実機）。
const RESULT_DELAY_MS: u64 = 2500;
/// 陰爻の片側の長さ（爻の幅に対する割合 [%]）。
const YIN_SEGMENT_PCT: i32 = 41;
/// 結果の卦画の変爻の印の、卦画の右端からの距離 [px]。
const FIG_MARK_DX: i32 = 18;

/// 易。
pub struct IchingApp {
    lines: [Option<Line>; LINES],
    count: usize,
}

impl IchingApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            lines: [None; LINES],
            count: 0,
        }
    }

    fn reset(&mut self) {
        self.lines = [None; LINES];
        self.count = 0;
    }

    fn all(&self) -> Option<[Line; LINES]> {
        let mut out = [Line { coins: [false; 3] }; LINES];
        for (o, l) in out.iter_mut().zip(self.lines) {
            *o = l?;
        }
        Some(out)
    }

    /// 途中の画面。
    fn draw_cast(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        art::draw(&mut canvas, &art::IC_CAST_FRAME, Point::zero());
        art::draw(&mut canvas, &art::IC_NOTE[self.count], Point::zero());
        for i in 0..LINES {
            let cy = art::IC_ROW_BOTTOM_Y - i as i32 * art::IC_ROW_PITCH;
            let Some(line) = self.lines[i] else {
                dashed(
                    &mut canvas,
                    art::IC_BAR_X,
                    art::IC_BAR_X + art::IC_BAR_W,
                    cy,
                );
                continue;
            };
            bar(
                &mut canvas,
                art::IC_BAR_X,
                cy - art::IC_BAR_H / 2,
                art::IC_BAR_W,
                art::IC_BAR_H,
                line.is_yang(),
            );
            mark(&mut canvas, line, Point::new(art::IC_MARK_CX, cy));
            let coins = line
                .coins
                .iter()
                .fold(0, |acc, &h| (acc << 1) | usize::from(h));
            art::draw(
                &mut canvas,
                &art::IC_COINS[coins],
                Point::new(art::IC_COIN_CX, cy),
            );
        }
    }

    /// 結果の画面。
    fn draw_result(&self, ctx: &mut Ctx, lines: &[Line; LINES]) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        art::draw(&mut canvas, &art::IC_RESULT_FRAME, Point::zero());
        let primary = iching::primary(lines);
        let changed = iching::changed(lines);
        let cols: &[i32] = if changed.is_some() {
            &art::IC_COL_CX
        } else {
            &[SCREEN_W / 2]
        };
        for (k, &cx) in cols.iter().enumerate() {
            let (code, label, marks) = match k {
                0 => (primary, &art::IC_LABEL_PRIMARY, Some(lines)),
                _ => (changed.unwrap_or(primary), &art::IC_LABEL_CHANGED, None),
            };
            let no = usize::from(iching::number(code)) - 1;
            let col = Point::new(cx, 0);
            art::draw(&mut canvas, label, col);
            figure(&mut canvas, cx, code, marks);
            art::draw(&mut canvas, &art::IC_NAME[no], col);
            art::draw(&mut canvas, &art::IC_NUMBER_READING[no], col);
            let top = Point::new(0, art::IC_SUM_TOP[k]);
            let heading = if k == 0 {
                &art::IC_SUM_PRIMARY
            } else {
                &art::IC_SUM_CHANGED
            };
            art::draw(&mut canvas, heading, top);
            art::draw(&mut canvas, &art::IC_SUMMARY[no], top);
        }
        if changed.is_some() {
            art::draw(&mut canvas, &art::IC_ARROW, Point::zero());
        }
        changes_row(&mut canvas, lines);
    }
}

/// 爻の棒（陽は 1 本・陰は中央が切れた 2 本）。
fn bar(canvas: &mut Canvas<'_>, x: i32, y: i32, w: i32, h: i32, yang: bool) {
    let fill = PrimitiveStyle::with_fill(Gray2::BLACK);
    let mut rect = |x: i32, w: i32| {
        let _ = Rectangle::new(Point::new(x, y), Size::new(w as u32, h as u32))
            .into_styled(fill)
            .draw(canvas);
    };
    if yang {
        rect(x, w);
    } else {
        let seg = w * YIN_SEGMENT_PCT / 100;
        rect(x, seg);
        rect(x + w - seg, seg);
    }
}

/// 変爻の印（老陽〇・老陰×）。
fn mark(canvas: &mut Canvas<'_>, line: Line, at: Point) {
    if line.is_changing() {
        let m = if line.is_yang() {
            &art::IC_MARK_O
        } else {
            &art::IC_MARK_X
        };
        art::draw(canvas, m, at);
    }
}

/// まだ立てていない爻の欄（点線）。
fn dashed(canvas: &mut Canvas<'_>, x0: i32, x1: i32, y: i32) {
    let style = PrimitiveStyle::with_stroke(Gray2::BLACK, 1);
    for x in (x0..x1).step_by(8) {
        let _ = Segment::new(Point::new(x, y), Point::new(x + 3, y))
            .into_styled(style)
            .draw(canvas);
    }
}

/// 結果の卦画（下の爻が下）。`marks` があれば変爻の印を付ける。
fn figure(canvas: &mut Canvas<'_>, cx: i32, code: u8, marks: Option<&[Line; LINES]>) {
    let x = cx - art::IC_FIG_W / 2;
    for i in 0..LINES {
        let y = art::IC_FIG_TOP + (LINES - 1 - i) as i32 * (art::IC_FIG_BAR + art::IC_FIG_GAP);
        bar(
            canvas,
            x,
            y,
            art::IC_FIG_W,
            art::IC_FIG_BAR,
            code >> i & 1 == 1,
        );
        if let Some(lines) = marks {
            let at = Point::new(x + art::IC_FIG_W + FIG_MARK_DX, y + art::IC_FIG_BAR / 2);
            mark(canvas, lines[i], at);
        }
    }
}

/// 「変爻　三・五」（無ければ「変爻　なし」）を 1 字ずつ並べる（`'　'` は空白）。
fn changes_row(canvas: &mut Canvas<'_>, lines: &[Line; LINES]) {
    const POS: [char; LINES] = ['初', '二', '三', '四', '五', '上'];
    let mut row = ['　'; 2 + 1 + 2 * LINES];
    let mut n = 0;
    let mut push = |c: char| {
        row[n] = c;
        n += 1;
    };
    push('変');
    push('爻');
    push('　');
    let mut any = false;
    for (i, l) in lines.iter().enumerate() {
        if l.is_changing() {
            if any {
                push('・');
            }
            push(POS[i]);
            any = true;
        }
    }
    if !any {
        push('な');
        push('し');
    }
    let adv = art::IC_CHANGES_ADV;
    let x0 = SCREEN_W / 2 - adv * n as i32 / 2 + adv / 2;
    for (k, c) in row[..n].iter().enumerate() {
        if let Some(g) = art::IC_CHANGE_CHARS.iter().position(|x| x == c) {
            let at = Point::new(x0 + k as i32 * adv, art::IC_CHANGES_Y);
            art::draw(canvas, &art::IC_CHANGE_GLYPHS[g], at);
        }
    }
}

impl App for IchingApp {
    fn id(&self) -> &'static str {
        "iching"
    }

    fn title(&self) -> &'static str {
        "I Ching"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.reset();
        self.draw_cast(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if e != Event::ShakeEnd {
            return Action::None;
        }
        if self.count == LINES {
            self.reset();
        }
        let line = iching::cast_line(ctx.rng());
        self.lines[self.count] = Some(line);
        self.count += 1;
        println!("[iching] line {} = {}", self.count, line.value());
        self.draw_cast(ctx);
        ctx.present(Refresh::Partial).await;
        if let Some(lines) = self.all() {
            let p = iching::number(iching::primary(&lines));
            let c = iching::changed(&lines).map(iching::number);
            println!("[iching] primary={} changed={:?}", p, c);
            Timer::after_millis(RESULT_DELAY_MS).await;
            self.draw_result(ctx, &lines);
            ctx.full_refresh().await;
        }
        Action::None
    }
}
