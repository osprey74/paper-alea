//! タロット（ワンオラクル・DESIGN.md §9・§10）。
//!
//! 確定デザイン B 案（額装）：
//! - 待機：二重枠・見出し「I タロット」・裏面（360×540）・「振って一枚を引く」（4 階調）。
//! - 結果：カード（逆位置は 180° 回転）・英名・和名・正逆の札（4 階調）。
//! - キーワード：ボタン B で切替。正位置・逆位置のキーワード各 4 語、今の向きの札を反転（白黒の部分更新）。
//!
//! カード画像（`img/NN.a2b`）と文字画面（`cap/NN.a1b`・`word/NN.a1b`）は microSD から読む
//! （`tools/convert_cards.py`・`tools/render_tarot.py` が生成）。抽選は `alea_core::tarot`。

use core::fmt::Write as _;

use alea_core::tarot::{self, Draw};
use embedded_graphics::prelude::Point;
use esp_println::println;
use static_cell::ConstStaticCell;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::widgets::Lines;
use crate::ui::{art, image, FmtBuf};

/// 読み込み用のバッファ（カード画像 360×540・2bit が最大：16 + 48,600 バイト）。
const BUF_BYTES: usize = image::HEADER + 360 * 540 / 4;
static BUF: ConstStaticCell<[u8; BUF_BYTES]> = ConstStaticCell::new([0; BUF_BYTES]);

/// microSD 上の置き場所。
const DIR: &str = "/alea/tarot";

#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    /// 待機（裏面）。
    Wait,
    /// 結果（カード）。
    Card(Draw),
    /// キーワード。
    Words(Draw),
}

/// タロット。
pub struct TarotApp {
    view: View,
    buf: &'static mut [u8; BUF_BYTES],
}

impl TarotApp {
    /// 作る（1 回だけ呼べる）。
    pub fn new() -> Self {
        Self {
            view: View::Wait,
            buf: BUF.take(),
        }
    }

    /// 今の画面を描く。読めなかったファイルは画面に書き出す。
    async fn draw(&mut self, ctx: &mut Ctx) {
        ctx.clear();
        let mut missing = FmtBuf::<40>::new();
        match self.view {
            View::Wait => {
                art::draw(&mut ctx.canvas(), &art::TAROT_WAIT_FRAME, Point::zero());
                let (x, y) = art::TAROT_WAIT_CARD;
                self.load_a2b(ctx, "img", "back", x, y, false, &mut missing)
                    .await;
            }
            View::Card(d) => {
                let n = number(d.card);
                self.load_a1b(ctx, "cap", n.as_str(), &mut missing).await;
                let (x, y) = art::TAROT_RESULT_CARD;
                self.load_a2b(ctx, "img", n.as_str(), x, y, d.reversed, &mut missing)
                    .await;
                let chip = if d.reversed {
                    &art::TAROT_CHIP_REVERSED
                } else {
                    &art::TAROT_CHIP_UPRIGHT
                };
                art::draw(&mut ctx.canvas(), chip, Point::zero());
            }
            View::Words(d) => {
                self.load_a1b(ctx, "word", number(d.card).as_str(), &mut missing)
                    .await;
                let (x, y, w, h) = art::TAROT_WORD_CHIPS[usize::from(d.reversed)];
                ctx.canvas().invert_rect(x + 1, y + 1, w - 2, h - 2);
            }
        }
        if !missing.as_str().is_empty() {
            let mut lines = Lines::below_header();
            lines.put(&mut ctx.canvas(), "microSD: not found");
            lines.put(&mut ctx.canvas(), missing.as_str());
        }
    }

    /// `DIR/sub/name.a2b` を読んで `(x, y)` に描く。
    #[allow(clippy::too_many_arguments)]
    async fn load_a2b(
        &mut self,
        ctx: &mut Ctx,
        sub: &str,
        name: &str,
        x: i32,
        y: i32,
        rot180: bool,
        missing: &mut FmtBuf<40>,
    ) {
        let path = path(sub, name, "a2b");
        let ok = match ctx
            .storage()
            .read_all(path.as_str(), &mut self.buf[..])
            .await
        {
            Some(n) => image::draw_a2b(&mut ctx.canvas(), &self.buf[..n], x, y, rot180),
            None => false,
        };
        if !ok {
            println!("[tarot] cannot draw {}", path.as_str());
            let _ = write!(missing, "{}/{}.a2b", sub, name);
        }
    }

    /// `DIR/sub/name.a1b` を読んで描く。
    async fn load_a1b(&mut self, ctx: &mut Ctx, sub: &str, name: &str, missing: &mut FmtBuf<40>) {
        let path = path(sub, name, "a1b");
        let ok = match ctx
            .storage()
            .read_all(path.as_str(), &mut self.buf[..])
            .await
        {
            Some(n) => image::draw_a1b(&mut ctx.canvas(), &self.buf[..n]),
            None => false,
        };
        if !ok {
            println!("[tarot] cannot draw {}", path.as_str());
            let _ = write!(missing, "{}/{}.a1b ", sub, name);
        }
    }

    async fn show(&mut self, ctx: &mut Ctx) {
        self.draw(ctx).await;
        let mode = match self.view {
            View::Words(_) => Refresh::Partial,
            _ => Refresh::Gray,
        };
        ctx.present(mode).await;
    }
}

/// カード番号の 2 桁表記（`00`〜`77`）。
fn number(card: u8) -> FmtBuf<4> {
    let mut n = FmtBuf::<4>::new();
    let _ = write!(n, "{:02}", card);
    n
}

fn path(sub: &str, name: &str, ext: &str) -> FmtBuf<40> {
    let mut p = FmtBuf::<40>::new();
    let _ = write!(p, "{}/{}/{}.{}", DIR, sub, name, ext);
    p
}

impl App for TarotApp {
    fn id(&self) -> &'static str {
        "tarot"
    }

    fn title(&self) -> &'static str {
        "Tarot"
    }

    fn requires_sd(&self) -> bool {
        true
    }

    fn enter_gray(&self) -> bool {
        true
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.view = View::Wait;
        self.draw(ctx).await;
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ShakeEnd => {
                let d = tarot::draw(ctx.rng());
                println!(
                    "[tarot] card={:02} {}",
                    d.card,
                    if d.reversed { "reversed" } else { "upright" }
                );
                self.view = View::Card(d);
                self.show(ctx).await;
            }
            Event::ButtonB => {
                let next = match self.view {
                    View::Card(d) => View::Words(d),
                    View::Words(d) => View::Card(d),
                    View::Wait => return Action::None,
                };
                self.view = next;
                self.show(ctx).await;
            }
            _ => {}
        }
        Action::None
    }
}
