//! 設定（DESIGN.md §6.6・§9 設定画面）。ランチャーでボタン B を押すと開く。
//!
//! 確定デザイン（キャンバス 6 段目）：
//! - バックライト「消灯／弱／強」・自動電源オフ「しない／1／3／5／10 分」の札（タップで切替・今の値を反転）。
//!   値は M5PM1 の RTC RAM に保存され、電源を入れ直しても残る（`Ctx::set_backlight`・`Ctx::set_auto_off_min`）。
//! - 電池：電圧と残量の目安（USB 給電中はその旨）。
//! - 開発用：Refresh test・SD check を開く。
//! - ボタン B（または A）でランチャーに戻る。白黒の部分更新で描く。

use core::fmt::Write as _;

use alea_core::settings::{battery_percent, AUTO_OFF_CHOICES};
use embedded_graphics::prelude::Point;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::{Canvas, Refresh};
use crate::ui::art;
use crate::ui::FmtBuf;

/// 電圧と「V」・「約」と残量の間隔 [px]。
const UNIT_GAP: i32 = 6;
/// 電圧と残量の組の間隔 [px]。
const GROUP_GAP: i32 = 28;

/// 開発用のボタンで開くアプリ。
const DEV_APPS: [&str; 2] = ["debug_refresh", "debug_sd"];

/// 設定。
pub struct SettingsApp;

impl SettingsApp {
    /// 作る。
    pub const fn new() -> Self {
        Self
    }

    fn draw(&self, ctx: &mut Ctx) {
        let backlight = ctx.backlight();
        let auto_off = ctx.auto_off_min();
        let vbat = ctx.battery_mv();
        let usb = ctx.usb_present();
        ctx.clear();
        let mut canvas = ctx.canvas();
        art::draw(&mut canvas, &art::SETTINGS_FRAME, Point::zero());
        let invert = |c: &mut Canvas<'_>, (x, y, w, h): (i32, i32, i32, i32)| {
            c.invert_rect(x + 1, y + 1, w - 2, h - 2)
        };
        if let Some(&r) = art::SETTINGS_LIGHT.get(usize::from(backlight)) {
            invert(&mut canvas, r);
        }
        if let Some(i) = AUTO_OFF_CHOICES.iter().position(|&m| m == auto_off) {
            invert(&mut canvas, art::SETTINGS_AUTO[i]);
        }
        battery_row(&mut canvas, vbat, usb);
    }
}

/// 「3.92 V　約 70%」（USB 給電中は「3.92 V　USB 給電中」）を中央にそろえて描く。
fn battery_row(canvas: &mut Canvas<'_>, vbat: Option<u16>, usb: bool) {
    let y = art::SETTINGS_BATTERY_Y;
    let mut volt = FmtBuf::<8>::new();
    if let Some(mv) = vbat {
        let _ = write!(volt, "{}.{:02}", mv / 1000, mv % 1000 / 10);
    }
    let mut pct = FmtBuf::<8>::new();
    if let Some(mv) = vbat {
        let _ = write!(pct, "{}%", battery_percent(mv));
    }
    let left = if vbat.is_some() {
        art::text_width(&art::FONT_BATT_V, volt.as_str()) + UNIT_GAP + i32::from(art::BATT_V.w)
    } else {
        0
    };
    let right = if usb {
        i32::from(art::BATT_USB.w)
    } else if vbat.is_some() {
        i32::from(art::BATT_ABOUT.w) + UNIT_GAP + art::text_width(&art::FONT_BATT_PCT, pct.as_str())
    } else {
        0
    };
    let gap = if left > 0 && right > 0 { GROUP_GAP } else { 0 };
    let mut x = 240 - (left + gap + right) / 2;
    if vbat.is_some() {
        let tw = art::text_width(&art::FONT_BATT_V, volt.as_str());
        art::text(
            canvas,
            &art::FONT_BATT_V,
            volt.as_str(),
            Point::new(x + tw / 2, y),
        );
        x += tw + UNIT_GAP;
        let vw = i32::from(art::BATT_V.w);
        art::draw(canvas, &art::BATT_V, Point::new(x + vw / 2, y));
        x += vw + gap;
    }
    if usb {
        let uw = i32::from(art::BATT_USB.w);
        art::draw(canvas, &art::BATT_USB, Point::new(x + uw / 2, y));
    } else if vbat.is_some() {
        let aw = i32::from(art::BATT_ABOUT.w);
        art::draw(canvas, &art::BATT_ABOUT, Point::new(x + aw / 2, y));
        x += aw + UNIT_GAP;
        let pw = art::text_width(&art::FONT_BATT_PCT, pct.as_str());
        art::text(
            canvas,
            &art::FONT_BATT_PCT,
            pct.as_str(),
            Point::new(x + pw / 2, y),
        );
    }
}

/// タップ座標にある札の番号。
fn hit(rects: &[(i32, i32, i32, i32)], x: i32, y: i32) -> Option<usize> {
    rects
        .iter()
        .position(|&(rx, ry, w, h)| x >= rx && x < rx + w && y >= ry && y < ry + h)
}

impl App for SettingsApp {
    fn id(&self) -> &'static str {
        "settings"
    }

    fn title(&self) -> &'static str {
        "Settings"
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.draw(ctx);
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        let (x, y) = match e {
            Event::ButtonB => return Action::Close,
            Event::Tap { x, y } => (i32::from(x), i32::from(y)),
            _ => return Action::None,
        };
        if let Some(i) = hit(&art::SETTINGS_LIGHT, x, y) {
            if ctx.backlight() == i as u8 {
                return Action::None;
            }
            println!("[settings] backlight {}", i);
            ctx.set_backlight(i as u8);
        } else if let Some(i) = hit(&art::SETTINGS_AUTO, x, y) {
            let m = AUTO_OFF_CHOICES[i];
            if ctx.auto_off_min() == m {
                return Action::None;
            }
            println!("[settings] auto off {} min", m);
            ctx.set_auto_off_min(m);
        } else if let Some(i) = hit(&art::SETTINGS_DEV, x, y) {
            if DEV_APPS[i] == "debug_sd" && !ctx.storage().available() {
                println!("[settings] SD check disabled (no SD)");
                return Action::None;
            }
            return Action::OpenId(DEV_APPS[i]);
        } else {
            return Action::None;
        }
        self.draw(ctx);
        ctx.present(Refresh::Partial).await;
        Action::None
    }
}
