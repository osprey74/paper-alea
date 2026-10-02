//! ランチャー（DESIGN.md §9、HANDOFF T6）。
//!
//! 見出し帯「Alea」＋ 2 列×5 段のタイル。登録済みのアプリだけを並べ、残りの枠は空にする。
//! SD 必須のアプリは、SD が無いと網点で明灰色に見せてタップを無効にする。

use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::ui::layout::{HEADER_H, TILE_COLS, TILE_COUNT, TILE_H, TILE_W};
use crate::ui::widgets;

/// タイルに出すアプリの情報。
#[derive(Clone, Copy)]
pub struct TileInfo {
    /// 名称。
    pub title: &'static str,
    /// SD が必要か。
    pub requires_sd: bool,
}

/// ランチャー。
pub struct LauncherApp {
    tiles: [Option<TileInfo>; TILE_COUNT],
    sd_available: bool,
}

impl LauncherApp {
    /// 登録順のアプリ情報から作る（`TILE_COUNT` を超えた分は出さない）。
    pub fn new(apps: &[TileInfo]) -> Self {
        let mut tiles = [None; TILE_COUNT];
        for (slot, info) in tiles.iter_mut().zip(apps) {
            *slot = Some(*info);
        }
        Self {
            tiles,
            sd_available: false,
        }
    }

    fn enabled(&self, info: &TileInfo) -> bool {
        !info.requires_sd || self.sd_available
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        widgets::header(&mut canvas, "Alea", false);
        for (i, info) in self.tiles.iter().enumerate() {
            let Some(info) = info else { continue };
            widgets::tile(&mut canvas, tile_origin(i), info.title, self.enabled(info));
        }
    }
}

/// タイル `i` の左上座標。
fn tile_origin(i: usize) -> Point {
    let i = i as i32;
    Point::new((i % TILE_COLS) * TILE_W, HEADER_H + (i / TILE_COLS) * TILE_H)
}

/// タップ座標にあるタイルの番号。
fn tile_at(x: i32, y: i32) -> Option<usize> {
    if y < HEADER_H {
        return None;
    }
    let col = x / TILE_W;
    let row = (y - HEADER_H) / TILE_H;
    let i = (row * TILE_COLS + col) as usize;
    (col < TILE_COLS && i < TILE_COUNT).then_some(i)
}

impl App for LauncherApp {
    fn id(&self) -> &'static str {
        "launcher"
    }

    fn title(&self) -> &'static str {
        "Alea"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.sd_available = ctx.storage().available();
        self.draw(ctx);
    }

    async fn on_event(&mut self, _ctx: &mut Ctx, e: Event) -> Action {
        let Event::Tap { x, y } = e else {
            return Action::None;
        };
        let Some(i) = tile_at(i32::from(x), i32::from(y)) else {
            return Action::None;
        };
        match &self.tiles[i] {
            Some(info) if self.enabled(info) => {
                println!("[launcher] open #{} {}", i, info.title);
                Action::Open(i)
            }
            Some(info) => {
                println!("[launcher] #{} {} disabled (no SD)", i, info.title);
                Action::None
            }
            None => Action::None,
        }
    }
}
