//! ダイス（DESIGN.md §10.1）。
//!
//! - 確定デザイン（書物調）の台紙に、10 種の選択チップ（5 列×2 段）。タップで切替（初期値 1D6、アプリ終了まで保持）。
//!   選択中のチップは内側を反転して黒地に白抜きにする。
//! - 振り終わりで振る。D6 は目、それ以外は多角形の輪郭と数字。2 個以上なら下に「合計」を出す。
//!   1D100 は十の位と一の位の D10 を並べる。待機中は出目の欄を「?」にする。
//! 抽選は `alea_core::dice`（ホストで分布テスト済み）。画像部品は `tools/render_apps.py` が生成する。

use core::fmt::Write as _;

use alea_core::dice::{self, DiceKind, Roll};
use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::{Canvas, Refresh};
use crate::ui::art::{self, Die, Sprite};
use crate::ui::layout::SCREEN_W;
use crate::ui::FmtBuf;

/// 画面の左右中央。
const CX: i32 = SCREEN_W / 2;

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
        art::draw(&mut canvas, &art::DICE_FRAME, Point::zero());
        if let Some(i) = DiceKind::ALL.iter().position(|&k| k == self.kind) {
            let (x, y, w, h) = art::DICE_CHIPS[i];
            canvas.invert_rect(x + 1, y + 1, w - 2, h - 2);
        }
        // 出目（待機中は None）。
        let face = |i: usize| self.roll.map(|r| r.faces[i]);
        let total = self.roll.map(|r| r.total);
        match self.kind {
            DiceKind::D6 => d6(
                &mut canvas,
                &art::D6_L,
                Point::new(CX, art::DICE_SINGLE_Y),
                face(0),
            ),
            DiceKind::TwoD6 => {
                for (i, x) in [145, 335].into_iter().enumerate() {
                    d6(
                        &mut canvas,
                        &art::D6_M,
                        Point::new(x, art::DICE_ROW_Y),
                        face(i),
                    );
                }
                total_block(&mut canvas, art::TOTAL_LABEL_Y_D6, total);
            }
            DiceKind::ThreeD6 => {
                for (i, x) in [100, 240, 380].into_iter().enumerate() {
                    d6(
                        &mut canvas,
                        &art::D6_S,
                        Point::new(x, art::DICE_ROW_Y),
                        face(i),
                    );
                }
                total_block(&mut canvas, art::TOTAL_LABEL_Y_D6, total);
            }
            DiceKind::D100 => {
                // 十の位は「00」〜「90」、一の位は「0」〜「9」。
                for (i, x) in [146, 334].into_iter().enumerate() {
                    let mut label = FmtBuf::<4>::new();
                    match face(i) {
                        Some(v) if i == 0 => {
                            let _ = write!(label, "{:02}", v);
                        }
                        Some(v) => {
                            let _ = write!(label, "{}", v);
                        }
                        None => {
                            let _ = write!(label, "?");
                        }
                    }
                    polygon(
                        &mut canvas,
                        &art::DIE_M_KITE,
                        &art::FONT_DIE_M,
                        Point::new(x, art::DICE_ROW_Y),
                        label.as_str(),
                    );
                }
                total_block(&mut canvas, art::TOTAL_LABEL_Y_D100, total);
            }
            kind => {
                let die = match kind {
                    DiceKind::D3 | DiceKind::D4 => &art::DIE_L_TRIANGLE,
                    DiceKind::D8 => &art::DIE_L_DIAMOND,
                    DiceKind::D10 => &art::DIE_L_KITE,
                    DiceKind::D12 => &art::DIE_L_PENTAGON,
                    _ => &art::DIE_L_HEXAGON,
                };
                let mut label = FmtBuf::<4>::new();
                match face(0) {
                    Some(v) => {
                        let _ = write!(label, "{}", v);
                    }
                    None => {
                        let _ = write!(label, "?");
                    }
                }
                polygon(
                    &mut canvas,
                    die,
                    &art::FONT_DIE_L,
                    Point::new(CX, art::DICE_SINGLE_Y),
                    label.as_str(),
                );
            }
        }
    }
}

/// D6 の面（`None` は「?」）。
fn d6(canvas: &mut Canvas<'_>, faces: &[Sprite; 7], center: Point, face: Option<u32>) {
    let i = face.map_or(0, |f| f.clamp(1, 6) as usize);
    art::draw(canvas, &faces[i], center);
}

/// D6 以外：輪郭と数字。
fn polygon(canvas: &mut Canvas<'_>, die: &Die, font: &[art::Glyph], center: Point, label: &str) {
    art::draw(canvas, &die.outline, center);
    art::text_ink_centered(
        canvas,
        font,
        label,
        center + Point::new(0, i32::from(die.text_dy)),
    );
}

/// 「合計」と合計値（`None` は「?」）。
fn total_block(canvas: &mut Canvas<'_>, label_y: i32, total: Option<u32>) {
    art::draw(canvas, &art::TOTAL_LABEL, Point::new(CX, label_y));
    let mut value = FmtBuf::<4>::new();
    match total {
        Some(t) => {
            let _ = write!(value, "{}", t);
        }
        None => {
            let _ = write!(value, "?");
        }
    }
    art::text(
        canvas,
        &art::FONT_TOTAL,
        value.as_str(),
        Point::new(CX, label_y + art::TOTAL_VALUE_DY),
    );
}

/// タップ座標にあるチップ。
fn chip_at(x: i32, y: i32) -> Option<DiceKind> {
    art::DICE_CHIPS
        .iter()
        .position(|&(cx, cy, w, h)| x >= cx && x < cx + w && y >= cy && y < cy + h)
        .map(|i| DiceKind::ALL[i])
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
