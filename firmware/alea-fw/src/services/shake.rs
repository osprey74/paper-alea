//! Shake — 本体を振ったことの検出（DESIGN.md §6.3）。
//!
//! 判定ロジックは `alea_core::shake`（ホストでテスト済み）。ここでは BMI270 を
//! 約 100Hz（メインループの周期）で読み、検出器に渡す。描画中はサンプリングが止まるため、
//! 描画後に [`Shake::resync`] で状態を捨てる。

use alea_core::shake::{magnitude_mg, ShakeDetector, ShakeEvent, ShakeParams};
use embassy_time::Instant;
use esp_println::println;

use crate::board::imu;
use crate::board::ioe::SysI2c;

/// シェイク検出。
pub struct Shake {
    ok: bool,
    detector: ShakeDetector,
    /// 振っている間の最大の揺れ量 [mg]（ログ用）。
    peak_mg: u32,
}

impl Shake {
    /// BMI270 を初期化する。失敗してもシェイクが来ないだけで動作は続ける。
    pub async fn begin(i2c: &mut SysI2c) -> Self {
        let ok = imu::begin(i2c).await;
        Self {
            ok,
            detector: ShakeDetector::new(ShakeParams::DEFAULT),
            peak_mg: 0,
        }
    }

    /// 1 サンプル読んで検出器に渡す。
    pub fn poll(&mut self, i2c: &mut SysI2c) -> Option<ShakeEvent> {
        if !self.ok {
            return None;
        }
        let (x, y, z) = imu::read_mg(i2c)?;
        let mag = magnitude_mg(x, y, z);
        if self.detector.is_shaking() {
            self.peak_mg = self.peak_mg.max(mag.abs_diff(1000));
        }
        let now = Instant::now().as_millis() as u32;
        let event = self.detector.feed(mag, now);
        match event {
            Some(ShakeEvent::Start) => {
                self.peak_mg = mag.abs_diff(1000);
                println!("[Shake] start");
            }
            Some(ShakeEvent::End) => println!("[Shake] end (peak dev={}mg)", self.peak_mg),
            None => {}
        }
        event
    }

    /// 描画などで止まった後に、途中の状態を捨てる。
    pub fn resync(&mut self) {
        self.detector.reset();
    }
}
