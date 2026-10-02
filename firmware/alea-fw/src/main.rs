//! Alea — M5Stack PaperMono 向け「偶然」ミニアプリ集のファーム本体。
//!
//! 現状は M1 の T2（HAL 初期化）＋ T3（Display）の検証用：
//! - 起動時に 4 階調のテスト画面を描く（`Refresh::Gray`）
//! - ボタン B ＝ カウンタを進めて部分更新（`Refresh::Partial`。11 回目は自動で全面）
//! - ボタン A ＝ 明示的な全面更新（`full_refresh`）
//!
//! T6 で AppManager / Launcher に置き換える。bring-up は Nostos の実機実績を流用する。

#![no_std]
#![no_main]

mod board;
mod services;

use core::fmt::Write as _;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::mono_font::ascii::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::Gray2;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use esp_backtrace as _;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use static_cell::ConstStaticCell;

use services::display::{Display, Refresh, PLANE_BYTES, WIDTH};

// ESP-IDF 第二段ブートローダ用アプリ記述子。
esp_bootloader_esp_idf::esp_app_desc!();

/// 描画の最小間隔 [ms]（連打で部分更新を連続させない・R-2）。
const MIN_REDRAW_MS: u64 = 500;

/// ボタンのポーリング周期 [ms]。
const POLL_MS: u64 = 20;

// e-ink 用 1bpp プレーン（480×800 / 8 = 48,000 バイト ×2）。静的確保。
static BW_PLANE: ConstStaticCell<[u8; PLANE_BYTES]> = ConstStaticCell::new([0; PLANE_BYTES]);
static RED_PLANE: ConstStaticCell<[u8; PLANE_BYTES]> = ConstStaticCell::new([0; PLANE_BYTES]);

#[esp_hal::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    println!("[Board] alea-fw {} boot", env!("CARGO_PKG_VERSION"));

    // R-6: ブザー（GPIO42）は LOW 固定で鳴らさない。
    let _buzzer = Output::new(peripherals.GPIO42, Level::Low, OutputConfig::default());

    // 物理ボタン（active-low・内部プルアップ）。
    let btn_a = Input::new(peripherals.GPIO2, InputConfig::default().with_pull(Pull::Up));
    let btn_b = Input::new(peripherals.GPIO3, InputConfig::default().with_pull(Pull::Up));
    // SSD1677 BUSY（データシート準拠プルアップ）。
    let busy = Input::new(peripherals.GPIO18, InputConfig::default().with_pull(Pull::Up));

    // システム I2C（GPIO47 SDA / GPIO48 SCL・100 kHz）→ 電源レール bring-up。
    let mut i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .expect("I2C0")
        .with_sda(peripherals.GPIO47)
        .with_scl(peripherals.GPIO48);
    let up = board::ioe::bring_up(&mut i2c).await;

    // SSD1677 パネル（SPI2: MOSI=14 / SCLK=15 / CS=16 / DC=17）。
    let panel = board::panel::begin(
        &mut i2c,
        peripherals.SPI2,
        peripherals.GPIO14.into(),
        peripherals.GPIO15.into(),
        peripherals.GPIO16.into(),
        peripherals.GPIO17.into(),
        &busy,
    )
    .await;
    println!(
        "[Board] bring-up done: ioe={} power_hold={} panel={} sku={}",
        up.ioe_addr.is_some() as u8,
        up.power_held as u8,
        panel.is_some() as u8,
        if up.is_c153 { "C153" } else { "C153-Lite" }
    );

    let mut display = Display::new(panel, busy, BW_PLANE.take(), RED_PLANE.take());

    draw_gray_test(&mut display);
    display.present(&mut i2c, Refresh::Gray).await;

    let mut draws: u32 = 0;
    let mut prev_a = false;
    let mut prev_b = false;
    let mut last_draw = Instant::now();

    loop {
        Timer::after(Duration::from_millis(POLL_MS)).await;
        let a = btn_a.is_low();
        let b = btn_b.is_low();
        let a_pressed = a && !prev_a;
        let b_pressed = b && !prev_b;
        prev_a = a;
        prev_b = b;
        if !(a_pressed || b_pressed) {
            continue;
        }
        if last_draw.elapsed() < Duration::from_millis(MIN_REDRAW_MS) {
            continue;
        }

        draws += 1;
        if b_pressed {
            println!("[Input] button B");
            draw_counter(&mut display, draws, "B: partial");
            display.present(&mut i2c, Refresh::Partial).await;
        } else {
            println!("[Input] button A");
            draw_counter(&mut display, draws, "A: full refresh");
            display.full_refresh(&mut i2c).await;
        }
        last_draw = Instant::now();
    }
}

/// 起動時のテスト画面：4 階調の帯と見出し。
fn draw_gray_test(display: &mut Display) {
    display.clear();
    let mut canvas = display.canvas();
    let text = MonoTextStyle::new(&FONT_10X20, Gray2::BLACK);
    let _ = Text::new("Alea", Point::new(24, 48), text).draw(&mut canvas);
    let _ = Text::new("4-gray test (Refresh::Gray)", Point::new(24, 84), text).draw(&mut canvas);
    let bar_w = WIDTH / 4;
    for i in 0..4u8 {
        let _ = Rectangle::new(Point::new(i32::from(i) * bar_w, 120), Size::new(bar_w as u32, 480))
            .into_styled(PrimitiveStyle::with_fill(Gray2::new(i)))
            .draw(&mut canvas);
    }
    let _ = Text::new("B: partial  /  A: full refresh", Point::new(24, 680), text)
        .draw(&mut canvas);
}

/// ボタン操作後の画面：描画回数と部分更新回数。
fn draw_counter(display: &mut Display, draws: u32, action: &str) {
    let partials = display.partial_count();
    display.clear();
    let mut canvas = display.canvas();
    let text = MonoTextStyle::new(&FONT_10X20, Gray2::BLACK);
    let mut line = FmtBuf::<48>::new();
    let _ = Text::new("Alea - refresh test", Point::new(24, 48), text).draw(&mut canvas);
    let _ = Text::new(action, Point::new(24, 84), text).draw(&mut canvas);
    let _ = write!(line, "draw #{draws}");
    let _ = Text::new(line.as_str(), Point::new(24, 160), text).draw(&mut canvas);
    line.clear();
    let _ = write!(line, "partials before this draw: {partials}/10");
    let _ = Text::new(line.as_str(), Point::new(24, 196), text).draw(&mut canvas);
    // 部分更新回数を 10 個の枠で示す（塗り = 済み）。
    for i in 0..10i32 {
        let style = if i < i32::from(partials) {
            PrimitiveStyle::with_fill(Gray2::BLACK)
        } else {
            PrimitiveStyle::with_stroke(Gray2::BLACK, 3)
        };
        let _ = Rectangle::new(Point::new(24 + i * 44, 240), Size::new(36, 36))
            .into_styled(style)
            .draw(&mut canvas);
    }
    // 位置の移動で残像を見やすくするマーカー。
    let x = 24 + (draws as i32 % 10) * 44;
    let _ = Rectangle::new(Point::new(x, 320), Size::new(36, 300))
        .into_styled(PrimitiveStyle::with_fill(Gray2::BLACK))
        .draw(&mut canvas);
}

/// 固定長フォーマットバッファ（no-alloc で `write!` を受ける）。
struct FmtBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

impl<const N: usize> FmtBuf<N> {
    fn new() -> Self {
        Self { buf: [0; N], len: 0 }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> core::fmt::Write for FmtBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let n = bytes.len().min(N - self.len);
        self.buf[self.len..self.len + n].copy_from_slice(&bytes[..n]);
        self.len += n;
        Ok(())
    }
}
