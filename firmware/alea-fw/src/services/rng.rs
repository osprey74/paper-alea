//! Rng — 真性乱数（DESIGN.md §6.4・D-4）。
//!
//! ESP32-S3 のハードウェア乱数は、無線も ADC も使っていないと擬似乱数扱いになる
//! （esp-hal `src/rng/mod.rs` の説明）。Alea は無線を使わないため、esp-hal の
//! `TrngSource`（RNG ＋ ADC1 の雑音）を常に生かしておき、`Trng` から値を取る。
//! ADC1 はこのため他に使えない（Alea では未使用）。

use alea_core::RandomSource;
use esp_hal::rng::{Trng, TrngSource};
use esp_println::println;

/// 乱数源。
pub struct Rng {
    trng: Trng,
    _source: TrngSource<'static>,
}

impl Rng {
    /// RNG と ADC1 を占有して真性乱数源を作る。
    pub fn new(
        rng: esp_hal::peripherals::RNG<'static>,
        adc1: esp_hal::peripherals::ADC1<'static>,
    ) -> Self {
        let source = TrngSource::new(rng, adc1);
        let trng = Trng::try_new().expect("TrngSource is alive");
        println!("[Rng] trng ready (rng + adc1 entropy)");
        Self {
            trng,
            _source: source,
        }
    }

    /// 0 か 1 を等確率で。
    pub fn bernoulli(&mut self) -> bool {
        alea_core::bernoulli(self)
    }
}

impl RandomSource for Rng {
    fn next_u32(&mut self) -> u32 {
        self.trng.random()
    }
}
