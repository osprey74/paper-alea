//! Yes / No（DESIGN.md §10.6）。振り終わりで YES か NO を大きく表示する。

use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::{GrayColor, Point};
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::layout::SCREEN_W;
use crate::ui::widgets::{self, Lines};

/// Yes / No。
pub struct YesNoApp {
    answer: Option<bool>,
}

impl YesNoApp {
    /// 作る。
    pub const fn new() -> Self {
        Self { answer: None }
    }

    fn draw(&self, ctx: &mut Ctx) {
        ctx.clear();
        let mut canvas = ctx.canvas();
        widgets::header(&mut canvas, "Yes / No", true);
        let mut lines = Lines::below_header();
        lines.put(&mut canvas, "Shake to decide");
        let center = Point::new(SCREEN_W / 2, 430);
        match self.answer {
            Some(true) => widgets::big_text(&mut canvas, "YES", center, 12, Gray2::BLACK),
            Some(false) => widgets::big_text(&mut canvas, "NO", center, 12, Gray2::BLACK),
            None => widgets::big_text(&mut canvas, "?", center, 12, Gray2::BLACK),
        }
    }
}

impl App for YesNoApp {
    fn id(&self) -> &'static str {
        "yesno"
    }

    fn title(&self) -> &'static str {
        "Yes / No"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.answer = None;
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if e == Event::ShakeEnd {
            let yes = ctx.rng().bernoulli();
            self.answer = Some(yes);
            println!("[yesno] {}", if yes { "YES" } else { "NO" });
            self.draw(ctx);
            ctx.present(Refresh::Partial).await;
        }
        Action::None
    }
}
