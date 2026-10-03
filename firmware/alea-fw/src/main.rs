//! Alea — M5Stack PaperMono 向け「偶然」ミニアプリ集のファーム本体。
//!
//! 起動：HAL 初期化（Nostos の実機実績を流用）→ Core Services → AppManager（ランチャー表示）。
//! 以後はメインループで入力を一定周期で調べ、確定した操作を AppManager に渡す（R-2：定期描画しない）。

#![no_std]
#![no_main]

mod app_manager;
mod apps;
mod board;
mod services;
mod ui;

use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use esp_backtrace as _;
use esp_hal::gpio::{Input as GpioInput, InputConfig, Level, Output, OutputConfig, Pull};
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;
use static_cell::ConstStaticCell;

use app_manager::AppManager;
use services::app::{Ctx, Event, POLL_MS};
use services::display::{Display, PLANE_BYTES};
use services::input::Input;
use services::rng::Rng;
use services::shake::Shake;
use services::storage::Storage;

// ESP-IDF 第二段ブートローダ用アプリ記述子。
esp_bootloader_esp_idf::esp_app_desc!();

/// 描画の最小間隔 [ms]。直前の描画からこの時間内の操作は捨てる（連打で部分更新を連続させない・R-2）。
const MIN_REDRAW_MS: u64 = 500;

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

    // microSD（SDHOST 1bit: CLK=13 / CMD=12 / DAT0=11）。無くても続行する。
    let mut storage = Storage::begin(
        &mut i2c,
        peripherals.SDHOST,
        peripherals.GPIO13,
        peripherals.GPIO12,
        peripherals.GPIO11,
    )
    .await;

    // BMI270（シェイク検出）と真性乱数源（RNG ＋ ADC1）。
    let shake = Shake::begin(&mut i2c, &mut storage).await;
    let rng = Rng::new(peripherals.RNG, peripherals.ADC1);

    println!(
        "[Board] bring-up done: ioe={} power_hold={} panel={} sd={} sku={}",
        up.ioe_addr.is_some() as u8,
        up.power_held as u8,
        panel.is_some() as u8,
        storage.available() as u8,
        if up.is_c153 { "C153" } else { "C153-Lite" }
    );

    let display = Display::new(panel, busy, BW_PLANE.take(), RED_PLANE.take());
    let input = Input::new(btn_a, btn_b, tp_int);
    let mut ctx = Ctx::new(display, i2c, storage, input, shake, rng);
    let mut manager = AppManager::new();
    manager.start(&mut ctx).await;
    let mut last_draw = Instant::now();

    loop {
        Timer::after(Duration::from_millis(POLL_MS)).await;
        let Some(event) = ctx.poll() else {
            continue;
        };
        // 描画直後の操作は捨てる。電源ボタンは捨てない（押下の記録は読んだ時点で消えるため）。
        if event != Event::PowerButton
            && last_draw.elapsed() < Duration::from_millis(MIN_REDRAW_MS)
        {
            if event == Event::ShakeEnd {
                ctx.set_shake_led(false);
            }
            continue;
        }
        manager.handle(&mut ctx, event).await;
        last_draw = Instant::now();
    }
}
