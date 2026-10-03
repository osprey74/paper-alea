//! Storage — microSD の初期化とファイル読み込み（DESIGN.md §6.5）。
//!
//! SD が無い・初期化に失敗しても起動は止めない（`available() == false` で続行する）。
//! 4bit 化は読込速度の実測を見て判断する（現状は Nostos と同じ 1bit）。

use embassy_time::{Duration, Timer};
use esp_println::println;
use m5stack_papermono_lite::ioe1;

use crate::board::ioe::{self, SysI2c};
use crate::board::sd::Sd;
pub use crate::board::sd::{Capacity, DirItem};

/// SD 電源投入からカード初期化までの待ち [ms]。
const POWER_SETTLE_MS: u64 = 100;

/// microSD。
pub struct Storage {
    sd: Option<Sd>,
    /// TF_DET の読み（`Some(true)` = 挿入）。読めなければ `None`。
    detected: Option<bool>,
}

impl Storage {
    /// SD 電源（IOE1 PYG14）を入れ、TF_DET を読み、SDHOST 1bit でカードを初期化する。
    ///
    /// TF_DET は papermono-rs でも未検証（`nyc-tf-det`）のため、ログに残すだけで初期化は常に試す。
    pub async fn begin(
        i2c: &mut SysI2c,
        sdhost: esp_hal::peripherals::SDHOST<'static>,
        clk: esp_hal::peripherals::GPIO13<'static>,
        cmd: esp_hal::peripherals::GPIO12<'static>,
        dat0: esp_hal::peripherals::GPIO11<'static>,
    ) -> Self {
        let powered = ioe::set_output_verified(i2c, ioe1::MICROSD_ENABLE, true);
        Timer::after(Duration::from_millis(POWER_SETTLE_MS)).await;
        // 挿入 = LOW（スロットのスイッチが閉じる）。
        let detected = ioe::input_level(i2c, ioe1::MICROSD_DETECT).map(|high| !high);
        let sd = Sd::init(sdhost, clk, cmd, dat0).await;
        println!(
            "[Storage] power={} tf_det={} mount 1bit {}",
            powered as u8,
            match detected {
                Some(true) => "inserted",
                Some(false) => "empty",
                None => "unknown",
            },
            if sd.is_some() { "OK" } else { "FAILED" }
        );
        Self { sd, detected }
    }

    /// カードが使えるか。
    pub fn available(&self) -> bool {
        self.sd.is_some()
    }

    /// TF_DET の読み（`Some(true)` = 挿入）。
    pub fn detected(&self) -> Option<bool> {
        self.detected
    }

    /// バス幅（現状は常に 1bit）。
    pub fn bus_width(&self) -> &'static str {
        "1bit"
    }

    /// `path` にファイルかディレクトリがあるか。
    pub async fn exists(&mut self, path: &str) -> bool {
        match self.sd.as_mut() {
            Some(sd) => sd.exists(rel(path)).await,
            None => false,
        }
    }

    /// ファイルを `buf` に読み込み、読んだバイト数を返す。
    pub async fn read_all(&mut self, path: &str, buf: &mut [u8]) -> Option<usize> {
        self.sd.as_mut()?.read(rel(path), buf).await
    }

    /// ファイル全体を読み捨て、読めたバイト数を返す（読込速度の計測用）。
    pub async fn read_discard(&mut self, path: &str) -> Option<u64> {
        self.sd.as_mut()?.read_discard(rel(path)).await
    }

    /// 総容量と空き容量。
    pub async fn capacity(&mut self) -> Option<Capacity> {
        self.sd.as_mut()?.capacity().await
    }

    /// ディレクトリの中身を `out` に詰め、項目数を返す。
    pub async fn list(&mut self, dir: &str, out: &mut [DirItem]) -> Option<usize> {
        self.sd.as_mut()?.list(rel(dir), out).await
    }

    /// `dir` を作り、`path` を `size` バイトのファイルにする（ベンチ用）。
    pub async fn ensure_file(&mut self, dir: &str, path: &str, size: u64) -> bool {
        match self.sd.as_mut() {
            Some(sd) => sd.ensure_file(rel(dir), rel(path), size).await,
            None => false,
        }
    }
}

/// FAT のルートからの相対パスにする。embedded-fatfs は先頭の `/` があると開けないため、
/// `"/alea/tarot/..."` と `"alea/tarot/..."` のどちらでも渡せるように取り除く（2026-10-03 実機で判明）。
fn rel(path: &str) -> &str {
    path.trim_start_matches('/')
}
