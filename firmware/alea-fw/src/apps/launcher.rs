//! ランチャー（DESIGN.md §9、HANDOFF T6）。
//!
//! 書物調のデザイン（Claude Design「Alea ランチャー」B 案）を `tools/render_launcher.py` で
//! 4 階調の画像にしたものを表示する（`Refresh::Gray`）。ローマ数字 I〜IX の 9 枚のタイルに収録アプリを割り当て、
//! 未実装・SD が無いと使えないものは薄く表示してタップを無効にする。
//! ボタン B で設定画面（`apps/settings.rs`）を開く。開発用の画面は設定画面から開く。

use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::ui::layout::SCREEN_W;

include!("../../assets/launcher_tiles.rs");

/// ランチャーの画像（480×800・2bit/画素）。
static LAUNCHER_IMAGE: &[u8] = include_bytes!("../../assets/launcher.2bpp");

/// タイルに出すアプリの情報。
#[derive(Clone, Copy)]
pub struct TileInfo {
    /// 名称（ログ用）。
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

/// ランチャー。
pub struct LauncherApp {
    /// タイル I〜IX（`None` = 未実装）。
    main: [Option<Slot>; 9],
    sd_available: bool,
}

impl LauncherApp {
    /// タイルの割り当てから作る。
    pub fn new(main: [Option<Slot>; 9]) -> Self {
        Self {
            main,
            sd_available: false,
        }
    }

    fn enabled(&self, slot: &Option<Slot>) -> bool {
        slot.is_some_and(|s| !s.info.requires_sd || self.sd_available)
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        canvas.blit_2bpp(0, 0, SCREEN_W, 800, LAUNCHER_IMAGE);
        for (slot, &(x, y, w, h)) in self.main.iter().zip(LAUNCHER_TILES.iter()) {
            if !self.enabled(slot) {
                // 枠線は残し、内側だけ薄くする。
                canvas.fade_rect(x + 2, y + 2, w - 4, h - 4);
            }
        }
    }
}

/// タップ座標にあるタイルの番号（I〜IX → 0〜8）。
fn tile_at(x: i32, y: i32) -> Option<usize> {
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
        self.sd_available = ctx.storage().available();
        self.draw(ctx);
    }

    async fn on_event(&mut self, _ctx: &mut Ctx, e: Event) -> Action {
        match e {
            Event::ButtonB => Action::OpenId("settings"),
            Event::Tap { x, y } => {
                let Some(tile) = tile_at(i32::from(x), i32::from(y)) else {
                    return Action::None;
                };
                let slot = self.main[tile];
                match slot {
                    Some(s) if self.enabled(&slot) => {
                        println!("[launcher] open tile #{} {}", tile, s.info.title);
                        Action::Open(s.app)
                    }
                    Some(s) => {
                        println!(
                            "[launcher] tile #{} {} disabled (no SD)",
                            tile, s.info.title
                        );
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
