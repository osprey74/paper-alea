//! シェイク検出（DESIGN.md §6.3）。
//!
//! 加速度の大きさ ‖a‖ と 1g の差 |‖a‖ − 1g| を「揺れ量」とし、
//! - 揺れ量が閾値を上に横切るたびにピークを 1 回数える
//! - [`ShakeParams::start_window_ms`] 以内にピークが [`ShakeParams::start_peaks`] 回以上で振り始め
//! - 振り始めた後、揺れ量が閾値未満の状態が [`ShakeParams::end_quiet_ms`] 続いたら振り終わり
//!
//! 単位は mg（1g = 1000）と ms。浮動小数点を使わない。

/// 検出パラメータ。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShakeParams {
    /// 揺れ閾値 [mg]。
    pub threshold_mg: u32,
    /// 振り始めとみなすピーク数。
    pub start_peaks: u8,
    /// ピークを数える時間窓 [ms]。
    pub start_window_ms: u32,
    /// 振り終わりとみなす静止時間 [ms]。
    pub end_quiet_ms: u32,
}

impl ShakeParams {
    /// DESIGN.md §6.3 の初期値（0.8g・600ms 以内に 2 回・300ms 静止）。
    pub const DEFAULT: Self = Self {
        threshold_mg: 800,
        start_peaks: 2,
        start_window_ms: 600,
        end_quiet_ms: 300,
    };
}

/// 検出結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShakeEvent {
    /// 振り始め。
    Start,
    /// 振り終わり（ここで結果を確定する）。
    End,
}

/// ピーク時刻を覚えておく最大数。
const MAX_PEAKS: usize = 8;

/// シェイク検出器。サンプルを時刻順に [`ShakeDetector::feed`] へ渡す。
pub struct ShakeDetector {
    params: ShakeParams,
    above: bool,
    peaks: [u32; MAX_PEAKS],
    n_peaks: usize,
    shaking: bool,
    /// 振り始めた後、最後に閾値以上だった時刻。
    last_active_ms: u32,
}

impl ShakeDetector {
    /// パラメータを指定して作る。
    pub const fn new(params: ShakeParams) -> Self {
        Self {
            params,
            above: false,
            peaks: [0; MAX_PEAKS],
            n_peaks: 0,
            shaking: false,
            last_active_ms: 0,
        }
    }

    /// 振っている最中か。
    pub fn is_shaking(&self) -> bool {
        self.shaking
    }

    /// 状態を捨てる（描画などで長くサンプリングが止まった後に呼ぶ）。
    pub fn reset(&mut self) {
        self.above = false;
        self.n_peaks = 0;
        self.shaking = false;
    }

    /// 加速度の大きさ [mg] を時刻 [ms] とともに渡す。
    pub fn feed(&mut self, magnitude_mg: u32, now_ms: u32) -> Option<ShakeEvent> {
        let dev = magnitude_mg.abs_diff(1000);
        let above = dev >= self.params.threshold_mg;
        let rising = above && !self.above;
        self.above = above;

        if self.shaking {
            if above {
                self.last_active_ms = now_ms;
                return None;
            }
            if now_ms.wrapping_sub(self.last_active_ms) >= self.params.end_quiet_ms {
                self.shaking = false;
                self.n_peaks = 0;
                return Some(ShakeEvent::End);
            }
            return None;
        }

        if !rising {
            return None;
        }
        // 時間窓より古いピークを捨ててから、今回のピークを足す。
        let window = self.params.start_window_ms;
        let mut kept = 0;
        for i in 0..self.n_peaks {
            if now_ms.wrapping_sub(self.peaks[i]) <= window {
                self.peaks[kept] = self.peaks[i];
                kept += 1;
            }
        }
        self.n_peaks = kept;
        if self.n_peaks < MAX_PEAKS {
            self.peaks[self.n_peaks] = now_ms;
            self.n_peaks += 1;
        }
        if self.n_peaks >= usize::from(self.params.start_peaks) {
            self.shaking = true;
            self.last_active_ms = now_ms;
            return Some(ShakeEvent::Start);
        }
        None
    }
}

/// 3 軸の加速度 [mg] から大きさ [mg] を求める（整数の平方根）。
pub fn magnitude_mg(x: i32, y: i32, z: i32) -> u32 {
    let sq = (x as i64).pow(2) + (y as i64).pow(2) + (z as i64).pow(2);
    isqrt(sq as u64) as u32
}

fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    // ニュートン法。
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 10ms 間隔で大きさの列を流し、出たイベントと時刻を返す。
    fn run(samples: &[u32]) -> ([(u32, ShakeEvent); 8], usize) {
        let mut d = ShakeDetector::new(ShakeParams::DEFAULT);
        let mut out = [(0, ShakeEvent::Start); 8];
        let mut n = 0;
        for (i, &m) in samples.iter().enumerate() {
            let t = i as u32 * 10;
            if let Some(e) = d.feed(m, t) {
                out[n] = (t, e);
                n += 1;
            }
        }
        (out, n)
    }

    #[test]
    fn resting_device_never_shakes() {
        let samples = [1000u32; 300];
        assert_eq!(run(&samples).1, 0);
    }

    #[test]
    fn single_bump_is_ignored() {
        // 机に置いた衝撃のような単発ピーク。
        let mut samples = [1000u32; 200];
        samples[50] = 2500;
        samples[51] = 2500;
        assert_eq!(run(&samples).1, 0);
    }

    #[test]
    fn shake_starts_on_second_peak_and_ends_after_quiet() {
        let mut samples = [1000u32; 200];
        // 100ms・300ms・500ms にピーク（各 30ms）。
        for p in [10, 30, 50] {
            for k in 0..3 {
                samples[p + k] = 2600;
            }
        }
        let (ev, n) = run(&samples);
        assert_eq!(n, 2);
        assert_eq!(ev[0], (300, ShakeEvent::Start));
        // 最後に閾値以上だったのは 520ms。静止 300ms で 820ms に終わる。
        assert_eq!(ev[1], (820, ShakeEvent::End));
    }

    #[test]
    fn peaks_too_far_apart_do_not_start() {
        let mut samples = [1000u32; 300];
        samples[10] = 2600;
        samples[90] = 2600; // 800ms 後
        assert_eq!(run(&samples).1, 0);
    }

    #[test]
    fn free_fall_counts_as_deviation() {
        // ‖a‖ がほぼ 0（投げ上げ・落下）も 1g からの差として数える。
        let mut samples = [1000u32; 200];
        samples[10] = 100;
        samples[30] = 100;
        assert_eq!(run(&samples).0[0], (300, ShakeEvent::Start));
    }

    #[test]
    fn magnitude_is_euclidean() {
        assert_eq!(magnitude_mg(0, 0, 1000), 1000);
        assert_eq!(magnitude_mg(-1000, 0, 0), 1000);
        assert_eq!(magnitude_mg(600, 800, 0), 1000);
        assert_eq!(magnitude_mg(0, 0, 0), 0);
    }
}
