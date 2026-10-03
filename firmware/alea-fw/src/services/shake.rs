//! Shake — 本体を振ったことの検出（DESIGN.md §6.3）。
//!
//! 判定ロジックは `alea_core::shake`（ホストでテスト済み）。ここでは BMI270 を
//! 約 100Hz（メインループの周期）で読み、検出器に渡す。描画中はサンプリングが止まるため、
//! 描画後に [`Shake::resync`] で状態を捨てる。

use alea_core::config;
use alea_core::shake::{magnitude_mg, ShakeDetector, ShakeEvent, ShakeParams};
use embassy_time::Instant;
use esp_println::println;

use crate::board::imu;
use crate::board::ioe::SysI2c;
use crate::services::storage::Storage;

/// 設定ファイル（microSD）。
const CONFIG_PATH: &str = "alea/config.json";
/// 設定ファイルの最大の大きさ [byte]。
const CONFIG_MAX: usize = 1024;

/// シェイク検出。
pub struct Shake {
    ok: bool,
    detector: ShakeDetector,
    /// 振っている間の最大の揺れ量 [mg]（ログ用）。
    peak_mg: u32,
}

impl Shake {
    /// BMI270 を初期化する。失敗してもシェイクが来ないだけで動作は続ける。
    /// 検出パラメータは microSD の `config.json` があれば上書きする（DESIGN.md §6.3）。
    pub async fn begin(i2c: &mut SysI2c, storage: &mut Storage) -> Self {
        let ok = imu::begin(i2c).await;
        let params = load_params(storage).await;
        Self {
            ok,
            detector: ShakeDetector::new(params),
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

/// `config.json` を読んでパラメータを作る。無ければ・読めなければ既定値。
async fn load_params(storage: &mut Storage) -> ShakeParams {
    let mut buf = [0u8; CONFIG_MAX];
    let Some(n) = storage.read_all(CONFIG_PATH, &mut buf).await else {
        println!("[Shake] config.json not found, defaults");
        return ShakeParams::DEFAULT;
    };
    let text = core::str::from_utf8(&buf[..n]).unwrap_or("");
    let (p, applied) = config::shake_params(text);
    println!(
        "[Shake] config.json {} key(s): threshold={}mg peaks={} window={}ms quiet={}ms",
        applied, p.threshold_mg, p.start_peaks, p.start_window_ms, p.end_quiet_ms
    );
    p
}
