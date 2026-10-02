//! デザイン確認（`debug_preview`・開発用）。
//!
//! `tools/render_app_mockups.py` で画像にしたアプリ画面のデザイン案を、実機の見え方
//! （モノクロの部分更新）で順に表示する。ボタン B かタップで次の画面へ。

use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::layout::SCREEN_W;

/// 表示する画像（480×800・2bit/画素）と名前。
const MOCKS: [(&str, &[u8]); 4] = [
    ("dice 2D6", include_bytes!("../../assets/mock_dice.2bpp")),
    ("dice 1D100", include_bytes!("../../assets/mock_dice100.2bpp")),
    ("coin", include_bytes!("../../assets/mock_coin.2bpp")),
    ("yesno", include_bytes!("../../assets/mock_yesno.2bpp")),
];

/// デザイン確認。
pub struct DesignPreviewApp {
    index: usize,
}

impl DesignPreviewApp {
    /// 作る。
    pub const fn new() -> Self {
        Self { index: 0 }
    }

    fn draw(&self, ctx: &mut Ctx) {
        let (name, data) = MOCKS[self.index];
        println!("[debug_preview] {} ({}/{})", name, self.index + 1, MOCKS.len());
        ctx.clear();
        ctx.canvas().blit_2bpp(0, 0, SCREEN_W, 800, data);
    }
}

impl App for DesignPreviewApp {
    fn id(&self) -> &'static str {
        "debug_preview"
    }

    fn title(&self) -> &'static str {
        "Design preview"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.index = 0;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if matches!(e, Event::ButtonB | Event::Tap { .. }) {
            self.index = (self.index + 1) % MOCKS.len();
            self.draw(ctx);
            ctx.present(Refresh::Partial).await;
        }
        Action::None
    }
}
