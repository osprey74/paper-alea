//! Alea — M5Stack PaperMono 向け「偶然」ミニアプリ集のファーム本体。
//!
//! 現状は M1 の T2〜T4（HAL 初期化・Display・Input）の検証用：
//! - 四隅と中央に的を描き、タップ位置に印を付けて的とのずれを表示する（T4 の受け入れ確認）
//! - タップ ＝ 部分更新（`Refresh::Partial`。11 回目は自動で全面）
//! - ボタン B ＝ 明示的な全面更新（`full_refresh`）
//! - ボタン A ＝ 印を消して描き直す
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
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle};
use embedded_graphics::text::Text;
use esp_backtrace as _;
use esp_hal::gpio::{Input as GpioInput, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use static_cell::ConstStaticCell;

use services::display::{Display, Refresh, PLANE_BYTES};
use services::input::{Input, InputEvent, POLL_MS};

// ESP-IDF 第二段ブートローダ用アプリ記述子。
esp_bootloader_esp_idf::esp_app_desc!();

/// 描画の最小間隔 [ms]（連打で部分更新を連続させない・R-2）。
const MIN_REDRAW_MS: u64 = 500;

/// 的の位置（四隅と中央・ページ座標）。
const TARGETS: [(i32, i32); 5] = [(40, 40), (440, 40), (240, 400), (40, 760), (440, 760)];

/// 画面に残すタップ印の数。
const MAX_TAPS: usize = 20;

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

    let pull_up = InputConfig::default().with_pull(Pull::Up);
    // 物理ボタン（active-low）・FT6336G タッチ割り込み（active-low）。
    let btn_a = GpioInput::new(peripherals.GPIO2, pull_up);
    let btn_b = GpioInput::new(peripherals.GPIO3, pull_up);
    let tp_int = GpioInput::new(peripherals.GPIO4, pull_up);
    // SSD1677 BUSY（データシート準拠プルアップ）。
    let busy = GpioInput::new(peripherals.GPIO18, pull_up);

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
    let mut input = Input::new(btn_a, btn_b, tp_int);

    let mut taps: [(i32, i32); MAX_TAPS] = [(0, 0); MAX_TAPS];
    let mut n_taps: usize = 0;

    draw_touch_test(&mut display, &taps[..n_taps]);
    display.present(&mut i2c, Refresh::Partial).await;
    input.resync();
    let mut last_draw = Instant::now();

    loop {
        Timer::after(Duration::from_millis(POLL_MS)).await;
        let Some(event) = input.poll(&mut i2c) else {
            continue;
        };
        if last_draw.elapsed() < Duration::from_millis(MIN_REDRAW_MS) {
            continue;
        }
        match event {
            InputEvent::Tap { x, y } => {
                let p = (i32::from(x), i32::from(y));
                let (t, dx, dy) = nearest_target(p);
                println!(
                    "[Input] tap x={} y={} nearest=({},{}) dx={} dy={}",
                    p.0, p.1, t.0, t.1, dx, dy
                );
                if n_taps == MAX_TAPS {
                    taps.copy_within(1.., 0);
                    n_taps -= 1;
                }
                taps[n_taps] = p;
                n_taps += 1;
                draw_touch_test(&mut display, &taps[..n_taps]);
                display.present(&mut i2c, Refresh::Partial).await;
            }
            InputEvent::ButtonB => {
                println!("[Input] button B");
                draw_touch_test(&mut display, &taps[..n_taps]);
                display.full_refresh(&mut i2c).await;
            }
            InputEvent::ButtonA => {
                println!("[Input] button A (clear)");
                n_taps = 0;
                draw_touch_test(&mut display, &taps[..n_taps]);
                display.present(&mut i2c, Refresh::Partial).await;
            }
        }
        input.resync();
        last_draw = Instant::now();
    }
}

/// 最も近い的と、そこからのずれ。
fn nearest_target(p: (i32, i32)) -> ((i32, i32), i32, i32) {
    let mut best = TARGETS[0];
    let mut best_d = i32::MAX;
    for t in TARGETS {
        let d = (p.0 - t.0).pow(2) + (p.1 - t.1).pow(2);
        if d < best_d {
            best_d = d;
            best = t;
        }
    }
    (best, p.0 - best.0, p.1 - best.1)
}

/// タッチ検証画面：的・タップ印・最新タップのずれ。
fn draw_touch_test(display: &mut Display, taps: &[(i32, i32)]) {
    let partials = display.partial_count();
    display.clear();
    let mut canvas = display.canvas();
    let text = MonoTextStyle::new(&FONT_10X20, Gray2::BLACK);
    let thin = PrimitiveStyle::with_stroke(Gray2::BLACK, 2);
    let thick = PrimitiveStyle::with_stroke(Gray2::BLACK, 4);

    let _ = Text::new("Alea - touch test", Point::new(110, 120), text).draw(&mut canvas);
    let _ = Text::new("Tap the 5 targets", Point::new(110, 150), text).draw(&mut canvas);
    let _ = Text::new("A: clear   B: full refresh", Point::new(110, 180), text)
        .draw(&mut canvas);

    // 的：半径 20px の円と十字（中心が目標座標）。
    for (x, y) in TARGETS {
        let _ = Circle::with_center(Point::new(x, y), 40)
            .into_styled(thin)
            .draw(&mut canvas);
        let _ = Line::new(Point::new(x - 28, y), Point::new(x + 28, y))
            .into_styled(thin)
            .draw(&mut canvas);
        let _ = Line::new(Point::new(x, y - 28), Point::new(x, y + 28))
            .into_styled(thin)
            .draw(&mut canvas);
    }

    // タップ印：太い ×。
    for &(x, y) in taps {
        let _ = Line::new(Point::new(x - 10, y - 10), Point::new(x + 10, y + 10))
            .into_styled(thick)
            .draw(&mut canvas);
        let _ = Line::new(Point::new(x - 10, y + 10), Point::new(x + 10, y - 10))
            .into_styled(thick)
            .draw(&mut canvas);
    }

    let mut line = FmtBuf::<48>::new();
    if let Some(&p) = taps.last() {
        let (_, dx, dy) = nearest_target(p);
        let _ = write!(line, "last ({},{}) d=({},{})", p.0, p.1, dx, dy);
        let _ = Text::new(line.as_str(), Point::new(110, 470), text).draw(&mut canvas);
        line.clear();
    }

    // 的ごとの平均ずれ（d の平均と回数）。
    let _ = Text::new("avg d per target:", Point::new(110, 510), text).draw(&mut canvas);
    for (i, t) in TARGETS.iter().enumerate() {
        let (mut sx, mut sy, mut n) = (0i32, 0i32, 0i32);
        for &p in taps {
            let (nt, dx, dy) = nearest_target(p);
            if nt == *t {
                sx += dx;
                sy += dy;
                n += 1;
            }
        }
        if n > 0 {
            let _ = write!(line, "({},{}) n={} ({},{})", t.0, t.1, n, sx / n, sy / n);
        } else {
            let _ = write!(line, "({},{}) n=0", t.0, t.1);
        }
        let y = 540 + i as i32 * 26;
        let _ = Text::new(line.as_str(), Point::new(110, y), text).draw(&mut canvas);
        line.clear();
    }
    let _ = write!(line, "taps {}  partials {}/10", taps.len(), partials);
    let _ = Text::new(line.as_str(), Point::new(110, 690), text).draw(&mut canvas);
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
