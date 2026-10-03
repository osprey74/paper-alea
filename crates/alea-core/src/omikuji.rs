//! おみくじ（DESIGN.md §10.5）。運勢を重みに従って選び、その運勢の一言を等確率で選ぶ。
//!
//! 運勢の種類・重み・一言の数は firmware 側の表（`tools/data/omikuji.json` から生成）が持つ。

use crate::{uniform, RandomSource};

/// 引いた結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pick {
    /// 運勢の番号（`weights` の添字）。
    pub fortune: usize,
    /// その運勢の中の一言の番号。
    pub message: usize,
}

/// 重み `weights` に比例して運勢を選び、`messages[運勢]` 個の一言から 1 つを選ぶ。
///
/// # Panics
/// `weights` と `messages` の長さが違うとき、重みの合計が 0 のとき、一言が 0 個の運勢が選ばれたとき。
pub fn pick<R: RandomSource>(rng: &mut R, weights: &[u32], messages: &[u32]) -> Pick {
    assert_eq!(weights.len(), messages.len(), "omikuji: table size mismatch");
    let total: u32 = weights.iter().sum();
    let mut r = uniform(rng, total);
    let mut fortune = weights.len() - 1;
    for (i, &w) in weights.iter().enumerate() {
        if r < w {
            fortune = i;
            break;
        }
        r -= w;
    }
    let message = uniform(rng, messages[fortune]) as usize;
    Pick { fortune, message }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct XorShift(u32);

    impl RandomSource for XorShift {
        fn next_u32(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.0 = x;
            x
        }
    }

    const WEIGHTS: [u32; 6] = [15, 20, 20, 18, 15, 12];
    const MESSAGES: [u32; 6] = [5, 5, 5, 5, 5, 5];

    #[test]
    fn fortunes_follow_weights() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x0A1C_0B2D);
        let mut counts = [0u32; 6];
        for _ in 0..TRIALS {
            counts[pick(&mut rng, &WEIGHTS, &MESSAGES).fortune] += 1;
        }
        let total: u32 = WEIGHTS.iter().sum();
        for (c, w) in counts.iter().zip(WEIGHTS) {
            let expected = TRIALS / total * w;
            // 期待値の ±5%（最小の 12,000 でも約 5σ）。
            assert!(c.abs_diff(expected) < expected / 20, "counts={counts:?}");
        }
    }

    #[test]
    fn messages_are_uniform_within_fortune() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x7777_1234);
        let mut counts = [[0u32; 5]; 6];
        for _ in 0..TRIALS {
            let p = pick(&mut rng, &WEIGHTS, &MESSAGES);
            assert!(p.message < 5);
            counts[p.fortune][p.message] += 1;
        }
        for row in counts {
            let sum: u32 = row.iter().sum();
            for c in row {
                // 運勢ごとの期待値（最小 約 2,400）の ±15%。
                assert!(c.abs_diff(sum / 5) < sum / 5 * 15 / 100, "row={row:?}");
            }
        }
    }

    #[test]
    fn zero_weight_is_never_picked() {
        let mut rng = XorShift(0x0BEE_F00D);
        for _ in 0..10_000 {
            assert_ne!(pick(&mut rng, &[3, 0, 1], &[1, 1, 1]).fortune, 1);
        }
    }
}
