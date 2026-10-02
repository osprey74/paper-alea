//! ランチャー（DESIGN.md §9、HANDOFF T6）。
//!
//! - アプリのページ：書物調のデザイン（Claude Design「Alea ランチャー」B 案）を
//!   `tools/render_launcher.py` で 4 階調の画像にしたものを表示する（`Refresh::Gray`）。
//!   ローマ数字 I〜IX の 9 枚のタイルに収録アプリを割り当て、未実装・SD が無いと使えないものは薄く表示してタップを無効にする。
//! - 開発用ページ：ボタン B で切り替える。Refresh test・SD check を白黒のタイルで並べる。

use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::layout::{HEADER_H, SCREEN_W, TILE_COLS, TILE_COUNT, TILE_H, TILE_W};
use crate::ui::widgets::{self, Lines};

include!("../../assets/launcher_tiles.rs");

/// アプリのページの画像（480×800・2bit/画素）。
static LAUNCHER_IMAGE: &[u8] = include_bytes!("../../assets/launcher.2bpp");

/// タイルに出すアプリの情報。
#[derive(Clone, Copy)]
pub struct TileInfo {
    /// 名称（開発用ページで使う）。
    pub title: &'static str,
    /// SD が必要か。
    pub requires_sd: bool,
}

/// タイルに割り当てたアプリ（AppManager の登録順の番号と情報）。
#[derive(Clone, Copy)]
pub struct Slot {
    /// AppManager の登録順の番号。
    pub app: usize,
    /// 情報。
    pub info: TileInfo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Apps,
    Developer,
}

/// ランチャー。
pub struct LauncherApp {
    /// アプリのページのタイル I〜IX（`None` = 未実装）。
    main: [Option<Slot>; 9],
    /// 開発用ページのタイル。
    dev: [Option<Slot>; TILE_COUNT],
    page: Page,
    sd_available: bool,
}

impl LauncherApp {
    /// タイルの割り当てから作る。
    pub fn new(main: [Option<Slot>; 9], dev: [Option<Slot>; TILE_COUNT]) -> Self {
        Self {
            main,
            dev,
            page: Page::Apps,
            sd_available: false,
        }
    }

    fn enabled(&self, slot: &Option<Slot>) -> bool {
        slot.is_some_and(|s| !s.info.requires_sd || self.sd_available)
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        match self.page {
            Page::Apps => {
                canvas.blit_2bpp(0, 0, SCREEN_W, 800, LAUNCHER_IMAGE);
                for (slot, &(x, y, w, h)) in self.main.iter().zip(LAUNCHER_TILES.iter()) {
                    if !self.enabled(slot) {
                        // 枠線は残し、内側だけ薄くする。
                        canvas.fade_rect(x + 2, y + 2, w - 4, h - 4);
                    }
                }
            }
            Page::Developer => {
                widgets::header(&mut canvas, "Developer", false);
                for (i, slot) in self.dev.iter().enumerate() {
                    let Some(s) = slot else { continue };
                    widgets::tile(&mut canvas, dev_tile_origin(i), s.info.title, self.enabled(slot));
                }
                let mut lines = Lines::below_header();
                lines.gap(TILE_H * 2);
                lines.put(&mut canvas, "B: back to apps");
            }
        }
    }

    async fn present(&self, ctx: &mut Ctx) {
        let mode = match self.page {
            Page::Apps => Refresh::Gray,
            Page::Developer => Refresh::Partial,
        };
        ctx.present(mode).await;
    }
}

/// 開発用ページのタイル `i` の左上座標。
fn dev_tile_origin(i: usize) -> Point {
    let i = i as i32;
    Point::new((i % TILE_COLS) * TILE_W, HEADER_H + (i / TILE_COLS) * TILE_H)
}

/// 開発用ページのタップ座標にあるタイルの番号。
fn dev_tile_at(x: i32, y: i32) -> Option<usize> {
    if y < HEADER_H {
        return None;
    }
    let col = x / TILE_W;
    let row = (y - HEADER_H) / TILE_H;
    let i = (row * TILE_COLS + col) as usize;
    (col < TILE_COLS && i < TILE_COUNT).then_some(i)
}

/// アプリのページのタップ座標にあるタイルの番号（I〜IX → 0〜8）。
fn main_tile_at(x: i32, y: i32) -> Option<usize> {
    LAUNCHER_TILES
        .iter()
        .position(|&(tx, ty, w, h)| x >= tx && x < tx + w && y >= ty && y < ty + h)
}

impl App for LauncherApp {
    fn id(&self) -> &'static str {
        "launcher"
    }

    fn title(&self) -> &'static str {
        "Alea"
    }

    fn enter_gray(&self) -> bool {
        true
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.page = Page::Apps;
        self.sd_available = ctx.storage().available();
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ButtonB => {
                self.page = match self.page {
                    Page::Apps => Page::Developer,
                    Page::Developer => Page::Apps,
                };
                self.draw(ctx);
                self.present(ctx).await;
                Action::None
            }
            Event::Tap { x, y } => {
                let (x, y) = (i32::from(x), i32::from(y));
                let (tile, slot) = match self.page {
                    Page::Apps => match main_tile_at(x, y) {
                        Some(i) => (i, self.main[i]),
                        None => return Action::None,
                    },
                    Page::Developer => match dev_tile_at(x, y) {
                        Some(i) => (i, self.dev[i]),
                        None => return Action::None,
                    },
                };
                match slot {
                    Some(s) if self.enabled(&slot) => {
                        println!("[launcher] open tile #{} {}", tile, s.info.title);
                        Action::Open(s.app)
                    }
                    Some(s) => {
                        println!("[launcher] tile #{} {} disabled (no SD)", tile, s.info.title);
                        Action::None
                    }
                    None => {
                        println!("[launcher] tile #{} not implemented", tile);
                        Action::None
                    }
                }
            }
            _ => Action::None,
        }
    }
}
