//! ルーン（DESIGN.md §10.8）。エルダー・フサルク 24 文字から 1 つを引く（ルーン文字を用いた現代の占い）。
//!
//! 逆位置（merkstave）は既定で無効。有効のときも、点対称の字形（どちらに置いても同じ形）は逆位置にしない。
//! どの文字が点対称かは firmware 側の表（`tools/data/runes.json` の字形から生成）が渡す。

use crate::{bernoulli, uniform, RandomSource};

/// 文字の数。
pub const RUNE_COUNT: u32 = 24;

/// 引いた結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    /// 文字の番号（0=Fehu … 23=Othala）。
    pub rune: u8,
    /// 逆位置か。
    pub reversed: bool,
}

/// 1 つ引く。`allow_reversed` が true のとき、`symmetric[文字]` が false の文字だけ逆位置になり得る。
pub fn draw<R: RandomSource>(rng: &mut R, allow_reversed: bool, symmetric: &[bool; 24]) -> Draw {
    let rune = uniform(rng, RUNE_COUNT) as u8;
    let reversed = allow_reversed && !symmetric[usize::from(rune)] && bernoulli(rng);
    Draw { rune, reversed }
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

    /// Gebo・Hagalaz・Naudiz・Isa・Jera・Eihwaz・Sowilo・Ingwaz・Dagaz。
    const SYMMETRIC: [bool; 24] = {
        let mut s = [false; 24];
        let ids = [6, 8, 9, 10, 11, 12, 15, 21, 22];
        let mut i = 0;
        while i < ids.len() {
            s[ids[i]] = true;
            i += 1;
        }
        s
    };

    #[test]
    fn runes_are_uniform() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x2B1E_F00D);
        let mut counts = [0u32; 24];
        for _ in 0..TRIALS {
            counts[usize::from(draw(&mut rng, true, &SYMMETRIC).rune)] += 1;
        }
        let expected = TRIALS / RUNE_COUNT;
        for c in counts {
            // 期待値 約 4,166 に対して ±10%（約 6σ）。
            assert!(c.abs_diff(expected) < expected / 10, "counts={counts:?}");
        }
    }

    #[test]
    fn reversal_off_by_default_and_never_for_symmetric() {
        let mut rng = XorShift(0x0BAD_5EED);
        for _ in 0..50_000 {
            assert!(!draw(&mut rng, false, &SYMMETRIC).reversed);
            let d = draw(&mut rng, true, &SYMMETRIC);
            if SYMMETRIC[usize::from(d.rune)] {
                assert!(!d.reversed, "symmetric rune {} reversed", d.rune);
            }
        }
    }

    #[test]
    fn asymmetric_runes_reverse_about_half() {
        const TRIALS: u32 = 200_000;
        let mut rng = XorShift(0x1234_9876);
        let (mut n, mut rev) = (0u32, 0u32);
        for _ in 0..TRIALS {
            let d = draw(&mut rng, true, &SYMMETRIC);
            if !SYMMETRIC[usize::from(d.rune)] {
                n += 1;
                rev += u32::from(d.reversed);
            }
        }
        assert!(rev.abs_diff(n / 2) < n / 100, "n={n} reversed={rev}");
    }
}
