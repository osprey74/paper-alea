//! Input — ボタン A/B とタッチ（Tap のみ）（DESIGN.md §6.2）。
//!
//! [`Input::poll`] を 20ms ごとに呼ぶと、確定した操作を 1 つずつ返す（下の tick 数はこの周期が前提）。
//! - ボタン：active-low。[`DEBOUNCE_TICKS`] 回連続で同じ状態のときだけ変化とみなし、押下の瞬間を返す。
//! - タッチ：指を離した時点で判定する。押下から離すまで [`TAP_MAX_MS`] 以内・移動 [`TAP_MAX_MOVE_PX`] 以内を
//!   Tap とし、着地点の座標を返す（それ以外の動きは捨てる）。座標は有効範囲にクランプする。
//!   タッチ処理は Nostos `main.rs` のジェスチャ判定（指離れを数 tick の無接触で確定）に倣う。

use embassy_time::{Duration, Instant};
use esp_hal::gpio::Input as GpioInput;
use m5stack_papermono_lite::display::{self, PageRotation};
use m5stack_papermono_lite::touch;

use crate::board::ioe::{self, SysI2c};

/// ボタンの状態変化とみなす連続一致回数（20ms × 2 = 40ms）。
const DEBOUNCE_TICKS: u8 = 2;

/// 指離れを確定する無接触 tick 数（20ms × 3 = 60ms）。
const TOUCH_RELEASE_TICKS: u8 = 3;

/// Tap とみなす最大の押下時間 [ms]。
const TAP_MAX_MS: u64 = 300;

/// Tap とみなす最大の移動量 [px]。
const TAP_MAX_MOVE_PX: i32 = 20;

/// タッチ座標の補正量 [px]（ページ座標に加える）。
///
/// 2026-10-03 C153 実機（机上・人差し指の腹・5 点×3 回）で、全点がほぼ一様に右下へ
/// 平均 (+8.6, +8.7) px ずれた（倍率は x 1.017 / y 0.998 で回転・軸入替なし）。
/// センサー由来か指の接触重心かは切り分けていないが、どちらでも一律の平行移動で補正できる。
const TOUCH_OFFSET: (i32, i32) = (-9, -9);

/// 確定した操作。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputEvent {
    /// ボタン A の押下。
    ButtonA,
    /// ボタン B の押下。
    ButtonB,
    /// タップ（ページ座標・縦持ち 480×800）。
    Tap {
        /// x [px]。
        x: i16,
        /// y [px]。
        y: i16,
    },
}

/// デバウンス付きボタン。
struct Button {
    pin: GpioInput<'static>,
    stable: bool,
    count: u8,
}

impl Button {
    fn new(pin: GpioInput<'static>) -> Self {
        let stable = pin.is_low();
        Self {
            pin,
            stable,
            count: 0,
        }
    }

    /// 押下の瞬間だけ true を返す。
    fn poll(&mut self) -> bool {
        let raw = self.pin.is_low();
        if raw == self.stable {
            self.count = 0;
            return false;
        }
        self.count += 1;
        if self.count < DEBOUNCE_TICKS {
            return false;
        }
        self.count = 0;
        self.stable = raw;
        raw
    }

    /// 現在の状態を取り直す（描画などで長く止まった後に、離した操作を誤検出しないため）。
    fn resync(&mut self) {
        self.stable = self.pin.is_low();
        self.count = 0;
    }
}

/// 進行中のタッチ。
struct Touch {
    start: (i32, i32),
    last: (i32, i32),
    started_at: Instant,
    last_seen_at: Instant,
    idle: u8,
}

/// ボタンとタッチの入力。
pub struct Input {
    btn_a: Button,
    btn_b: Button,
    tp_int: GpioInput<'static>,
    touch: Option<Touch>,
}

impl Input {
    /// ボタン A（GPIO2）・B（GPIO3）とタッチ割り込み（GPIO4）から作る。
    pub fn new(
        btn_a: GpioInput<'static>,
        btn_b: GpioInput<'static>,
        tp_int: GpioInput<'static>,
    ) -> Self {
        Self {
            btn_a: Button::new(btn_a),
            btn_b: Button::new(btn_b),
            tp_int,
            touch: None,
        }
    }

    /// 1 周期分の処理を行い、確定した操作があれば返す。
    pub fn poll(&mut self, i2c: &mut SysI2c) -> Option<InputEvent> {
        if self.btn_a.poll() {
            return Some(InputEvent::ButtonA);
        }
        if self.btn_b.poll() {
            return Some(InputEvent::ButtonB);
        }
        self.poll_touch(i2c)
    }

    /// 描画などで長く止まった後に呼び、その間の押下・タッチを捨てる。
    pub fn resync(&mut self) {
        self.btn_a.resync();
        self.btn_b.resync();
        self.touch = None;
    }

    fn poll_touch(&mut self, i2c: &mut SysI2c) -> Option<InputEvent> {
        // INT が上がっていても、ホールド中は座標を読み続ける。
        let point = if self.tp_int.is_low() || self.touch.is_some() {
            ioe::read_touch(i2c).and_then(|(fx, fy)| to_page(fx, fy))
        } else {
            None
        };
        let now = Instant::now();
        match (point, self.touch.as_mut()) {
            (Some(p), Some(t)) => {
                t.last = p;
                t.last_seen_at = now;
                t.idle = 0;
                None
            }
            (Some(p), None) => {
                self.touch = Some(Touch {
                    start: p,
                    last: p,
                    started_at: now,
                    last_seen_at: now,
                    idle: 0,
                });
                None
            }
            (None, Some(t)) => {
                t.idle += 1;
                if t.idle < TOUCH_RELEASE_TICKS {
                    return None;
                }
                let t = self.touch.take()?;
                let dx = t.last.0 - t.start.0;
                let dy = t.last.1 - t.start.1;
                let moved = dx * dx + dy * dy > TAP_MAX_MOVE_PX * TAP_MAX_MOVE_PX;
                let held = t.last_seen_at - t.started_at;
                if moved || held > Duration::from_millis(TAP_MAX_MS) {
                    esp_println::println!(
                        "[Input] touch ignored (held={}ms move=({},{}))",
                        held.as_millis(),
                        dx,
                        dy
                    );
                    return None;
                }
                Some(InputEvent::Tap {
                    x: t.start.0 as i16,
                    y: t.start.1 as i16,
                })
            }
            (None, None) => None,
        }
    }
}

/// 物理フレームバッファ座標 → ページ座標（縦持ち）。補正を加え、有効範囲にクランプする。
fn to_page(fx: u16, fy: u16) -> Option<(i32, i32)> {
    let (px, py) = display::framebuffer_to_page(fx, fy, PageRotation::Portrait0)?;
    let x = (i32::from(px) + TOUCH_OFFSET.0).clamp(
        i32::from(touch::ACTIVE_MIN_X),
        i32::from(touch::ACTIVE_MAX_X),
    );
    let y = (i32::from(py) + TOUCH_OFFSET.1).clamp(
        i32::from(touch::ACTIVE_MIN_Y),
        i32::from(touch::ACTIVE_MAX_Y),
    );
    Some((x, y))
}
