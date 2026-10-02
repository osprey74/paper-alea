//! Display — フレームバッファ・描画反映・リフレッシュ回数管理（DESIGN.md §6.1）。
//!
//! - フレームバッファは BW / RED の 1bpp プレーン 2 枚（各 48,000 バイト・静的確保）。
//! - 描画は [`Canvas`]（embedded-graphics の `DrawTarget<Color = Gray2>`）に行う。
//!   `Gray2` の輝度 0=黒 / 1=暗灰 / 2=明灰 / 3=白 がそのまま OTP の 4 階調トーンになる。
//! - 部分更新の回数管理（R-1）は [`crate::board::panel`] が強制し、アプリからは迂回できない。

use embassy_time::Instant;
use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use esp_hal::gpio::Input;
use esp_println::println;
use m5stack_papermono_lite::display::{self, PageRotation};

use crate::board::ioe::SysI2c;
use crate::board::panel::{Painted, Panel, PARTIALS_BEFORE_FULL};

/// 固定ページ回転（USB-C 下・縦持ち 480×800）。
const ROT: PageRotation = PageRotation::Portrait0;

/// ページ幅 [px]。
pub const WIDTH: i32 = display::PAGE_PORTRAIT_W as i32;
/// ページ高 [px]。
pub const HEIGHT: i32 = display::PAGE_PORTRAIT_H as i32;

/// 1 プレーンのバイト数。
pub const PLANE_BYTES: usize = display::PLANE_BYTES;

/// 描画反映の方式（差分高速モードは定義しない・R-4）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refresh {
    /// 4 階調（`0xD7`）。常に全面。
    Gray,
    /// モノクロ部分（`0xFF`）。回数が上限に達していればモノクロ全面に置き換わる。
    /// 灰色の画素は黒として表示される。
    Partial,
}

/// 画面。
pub struct Display {
    panel: Option<Panel>,
    busy: Input<'static>,
    bw: &'static mut [u8; PLANE_BYTES],
    red: &'static mut [u8; PLANE_BYTES],
}

impl Display {
    /// パネル（初期化に失敗していれば `None`）と BUSY 入力、2 枚のプレーンから作る。
    pub fn new(
        panel: Option<Panel>,
        busy: Input<'static>,
        bw: &'static mut [u8; PLANE_BYTES],
        red: &'static mut [u8; PLANE_BYTES],
    ) -> Self {
        Self {
            panel,
            busy,
            bw,
            red,
        }
    }

    /// 前回の全面更新からの部分更新回数。
    pub fn partial_count(&self) -> u8 {
        self.panel.as_ref().map_or(0, Panel::partials)
    }

    /// フレームバッファを白で埋める。
    pub fn clear(&mut self) {
        self.bw.fill(0);
        self.red.fill(0);
    }

    /// 描画先。
    pub fn canvas(&mut self) -> Canvas<'_> {
        Canvas {
            bw: &mut self.bw[..],
            red: &mut self.red[..],
        }
    }

    /// フレームバッファを画面に反映する。部分更新が上限に達していれば自動で全面にする。
    pub async fn present(&mut self, i2c: &mut SysI2c, mode: Refresh) {
        let Some(panel) = self.panel.as_mut() else {
            println!("[Display] present skipped (panel not ready)");
            return;
        };
        let t0 = Instant::now();
        let painted = match mode {
            Refresh::Gray => {
                panel.paint_gray(i2c, &self.bw[..], &self.red[..], &self.busy).await;
                Painted::Gray
            }
            Refresh::Partial => {
                panel.paint_mono(i2c, &self.bw[..], &self.red[..], &self.busy).await
            }
        };
        log_paint(mode, painted, panel.partials(), t0);
    }

    /// 明示的なモノクロ全面更新（部分更新の回数を 0 に戻す）。
    pub async fn full_refresh(&mut self, i2c: &mut SysI2c) {
        let Some(panel) = self.panel.as_mut() else {
            println!("[Display] full_refresh skipped (panel not ready)");
            return;
        };
        let t0 = Instant::now();
        panel
            .paint_mono_full(i2c, &self.bw[..], &self.red[..], &self.busy)
            .await;
        println!(
            "[Display] mode=FullRefresh partial={}/{} took={}ms",
            panel.partials(),
            PARTIALS_BEFORE_FULL,
            t0.elapsed().as_millis()
        );
    }
}

fn log_paint(mode: Refresh, painted: Painted, partials: u8, t0: Instant) {
    let replaced = mode == Refresh::Partial && painted == Painted::MonoFull;
    println!(
        "[Display] mode={:?}{}{:?} partial={}/{} took={}ms",
        mode,
        if replaced { "->" } else { "=" },
        painted,
        partials,
        PARTIALS_BEFORE_FULL,
        t0.elapsed().as_millis()
    );
}

/// 4 階調の描画先（ページ座標・範囲外は無視）。
pub struct Canvas<'a> {
    bw: &'a mut [u8],
    red: &'a mut [u8],
}

impl Canvas<'_> {
    fn set(&mut self, px: i32, py: i32, luma: u8) {
        if px < 0 || py < 0 || px >= WIDTH || py >= HEIGHT {
            return;
        }
        let Some((x, y)) = display::page_to_framebuffer(px as u16, py as u16, ROT) else {
            return;
        };
        let i = usize::from(y) * display::BYTES_PER_ROW + usize::from(x) / 8;
        let mask = 0x80u8 >> (x % 8);
        let (p1, p2) = display::gray_planes(luma);
        if let Some(b) = self.bw.get_mut(i) {
            if p1 {
                *b |= mask;
            } else {
                *b &= !mask;
            }
        }
        if let Some(b) = self.red.get_mut(i) {
            if p2 {
                *b |= mask;
            } else {
                *b &= !mask;
            }
        }
    }
}

impl OriginDimensions for Canvas<'_> {
    fn size(&self) -> Size {
        Size::new(WIDTH as u32, HEIGHT as u32)
    }
}

impl DrawTarget for Canvas<'_> {
    type Color = Gray2;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            self.set(point.x, point.y, color.luma());
        }
        Ok(())
    }
}
