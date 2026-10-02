//! Alea — M5Stack PaperMono 向け「偶然」ミニアプリ集のファーム本体。
//!
//! 現状は M1/T1 の雛形（起動ログのみ）。HAL 初期化・Display・Input・Storage・
//! AppManager・Launcher は `HANDOFF.md` の M1 で実装する。
//! bring-up は Nostos（`g:\dev\Nostos\firmware\nostos-fw`）の実機実績を流用する。

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::timer::timg::TimerGroup;
use esp_println::println;

// ESP-IDF 第二段ブートローダ用アプリ記述子。
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
async fn main(_spawner: Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    println!("[Board] alea-fw {} boot", env!("CARGO_PKG_VERSION"));

    loop {
        // R-2: loop 内で定期描画しない。M1 以降はイベント待ちに置き換える。
        Timer::after(Duration::from_secs(60)).await;
    }
}
