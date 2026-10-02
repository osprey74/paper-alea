//! Alea — M5Stack PaperMono 向け「偶然」ミニアプリ集のファーム本体。
//!
//! 現状は M1 の T2〜T5（HAL 初期化・Display・Input・Storage）の検証用：
//! - microSD のマウント方式・容量・`/alea` の一覧・読込速度（100KB）を表示する（T5 の受け入れ確認）
//! - ボタン B ＝ 再スキャン（部分更新） / ボタン A ＝ 全面更新
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
use embedded_graphics::text::Text;
use esp_backtrace as _;
use esp_hal::gpio::{Input as GpioInput, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use static_cell::ConstStaticCell;

use services::display::{Display, Refresh, PLANE_BYTES};
use services::input::{Input, InputEvent, POLL_MS};
use services::storage::{Capacity, DirItem, Storage};

// ESP-IDF 第二段ブートローダ用アプリ記述子。
esp_bootloader_esp_idf::esp_app_desc!();

/// 描画の最小間隔 [ms]（連打で部分更新を連続させない・R-2）。
const MIN_REDRAW_MS: u64 = 500;

/// `/alea` の一覧に表示する最大項目数。
const MAX_ITEMS: usize = 20;

/// Alea のデータディレクトリ。
const ALEA_DIR: &str = "alea";

/// 読込速度計測用のファイル（無ければ作る）。
const BENCH_PATH: &str = "alea/bench.bin";

/// 読込速度計測用ファイルの大きさ（100KB）。
const BENCH_BYTES: u64 = 100 * 1024;

const MB: u64 = 1024 * 1024;

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
    let mut storage = Storage::begin(
        &mut i2c,
        peripherals.SDHOST,
        peripherals.GPIO13,
        peripherals.GPIO12,
        peripherals.GPIO11,
    )
    .await;

    let mut report = scan_sd(&mut storage).await;
    draw_sd_check(&mut display, &storage, &report);
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
            InputEvent::ButtonB => {
                println!("[Input] button B (rescan)");
                report = scan_sd(&mut storage).await;
                draw_sd_check(&mut display, &storage, &report);
                display.present(&mut i2c, Refresh::Partial).await;
            }
            InputEvent::ButtonA => {
                println!("[Input] button A (full refresh)");
                display.full_refresh(&mut i2c).await;
            }
            InputEvent::Tap { x, y } => {
                println!("[Input] tap x={} y={} (ignored)", x, y);
                continue;
            }
        }
        input.resync();
        last_draw = Instant::now();
    }
}

/// SD 確認の結果。
struct SdReport {
    capacity: Option<Capacity>,
    /// ベンチ読込（バイト数, 所要 ms）。
    bench: Option<(u64, u64)>,
    items: [DirItem; MAX_ITEMS],
    n_items: Option<usize>,
}

/// 容量・`/alea` の一覧・ベンチ読込を行う（DESIGN.md §6.5、HANDOFF T5）。
async fn scan_sd(storage: &mut Storage) -> SdReport {
    let mut report = SdReport {
        capacity: None,
        bench: None,
        items: [DirItem::EMPTY; MAX_ITEMS],
        n_items: None,
    };
    if !storage.available() {
        println!("[Storage] not available");
        return report;
    }
    let t0 = Instant::now();
    report.capacity = storage.capacity().await;
    match report.capacity {
        Some(c) => println!(
            "[Storage] capacity total={}MB free={}MB (took {}ms)",
            c.total / MB,
            c.free / MB,
            t0.elapsed().as_millis()
        ),
        None => println!("[Storage] capacity FAILED"),
    }
    let made = storage.ensure_file(ALEA_DIR, BENCH_PATH, BENCH_BYTES).await;
    println!("[Storage] {} ready={}", BENCH_PATH, made as u8);
    let t0 = Instant::now();
    if let Some(n) = storage.read_discard(BENCH_PATH).await {
        let ms = t0.elapsed().as_millis();
        report.bench = Some((n, ms));
        println!(
            "[Storage] bench read {} bytes in {}ms ({} KB/s)",
            n,
            ms,
            if ms > 0 { n * 1000 / ms / 1024 } else { 0 }
        );
    } else {
        println!("[Storage] bench read FAILED");
    }
    report.n_items = storage.list(ALEA_DIR, &mut report.items).await;
    if let Some(n) = report.n_items {
        for item in &report.items[..n] {
            println!(
                "[Storage] /alea/{}{} {}",
                item.name(),
                if item.is_dir { "/" } else { "" },
                item.size
            );
        }
    }
    let exists = storage.exists(BENCH_PATH).await;
    let mut head = [0u8; 4];
    let head_n = storage.read_all(BENCH_PATH, &mut head).await;
    println!(
        "[Storage] exists({})={} read_all head={:?} n={:?}",
        BENCH_PATH, exists as u8, head, head_n
    );
    report
}

/// SD 確認画面。
fn draw_sd_check(display: &mut Display, storage: &Storage, r: &SdReport) {
    let partials = display.partial_count();
    display.clear();
    let mut canvas = display.canvas();
    let text = MonoTextStyle::new(&FONT_10X20, Gray2::BLACK);
    let mut line = FmtBuf::<48>::new();
    let mut y = 48;
    let put = |canvas: &mut _, s: &str, y: &mut i32| {
        let _ = Text::new(s, Point::new(24, *y), text).draw(canvas);
        *y += 28;
    };
    put(&mut canvas, "Alea - SD check", &mut y);
    put(&mut canvas, "B: rescan   A: full refresh", &mut y);
    y += 12;
    let det = match storage.detected() {
        Some(true) => "inserted",
        Some(false) => "empty",
        None => "unknown",
    };
    let _ = write!(line, "TF_DET: {det}");
    put(&mut canvas, line.as_str(), &mut y);
    line.clear();
    let _ = write!(
        line,
        "mount: {} {}",
        storage.bus_width(),
        if storage.available() { "OK" } else { "FAILED" }
    );
    put(&mut canvas, line.as_str(), &mut y);
    line.clear();
    match r.capacity {
        Some(c) => {
            let _ = write!(line, "total {} MB / free {} MB", c.total / MB, c.free / MB);
        }
        None => {
            let _ = write!(line, "capacity: -");
        }
    }
    put(&mut canvas, line.as_str(), &mut y);
    line.clear();
    match r.bench {
        Some((n, ms)) => {
            let kbs = if ms > 0 { n * 1000 / ms / 1024 } else { 0 };
            let _ = write!(line, "bench {}KB: {} ms ({} KB/s)", n / 1024, ms, kbs);
        }
        None => {
            let _ = write!(line, "bench: -");
        }
    }
    put(&mut canvas, line.as_str(), &mut y);
    line.clear();
    y += 12;
    put(&mut canvas, "/alea/", &mut y);
    match r.n_items {
        Some(0) => put(&mut canvas, "  (empty)", &mut y),
        Some(n) => {
            for item in &r.items[..n] {
                if item.is_dir {
                    let _ = write!(line, "  {}/", item.name());
                } else {
                    let _ = write!(line, "  {}  {}", item.name(), item.size);
                }
                put(&mut canvas, line.as_str(), &mut y);
                line.clear();
            }
        }
        None => put(&mut canvas, "  (not found)", &mut y),
    }
    let _ = write!(line, "partials {}/10", partials);
    let _ = Text::new(line.as_str(), Point::new(24, 770), text).draw(&mut canvas);
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
