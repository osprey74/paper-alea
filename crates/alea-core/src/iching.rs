//! 易・三枚硬貨法（DESIGN.md §10.7）。
//!
//! 1 回振るごとに硬貨 3 枚を投げ、表=3・裏=2 の合計で爻を決める（D-6）。
//! - 6＝老陰（陰・変爻）、7＝少陽（陽）、8＝少陰（陰）、9＝老陽（陽・変爻）
//! - 確率は 6 と 9 が 1/8、7 と 8 が 3/8（硬貨 3 枚の表の数が 0〜3 枚になる確率）。
//!
//! 爻は下から上へ積む。卦は 6 爻を「下の爻を bit0」とする 6 ビットの値で表し、
//! [`KING_WEN`] で卦の番号（文王の順・1〜64）に引く。

use crate::{bernoulli, RandomSource};

/// 爻の数。
pub const LINES: usize = 6;

/// 6 ビットの卦の値 → 文王の順の番号（1〜64）。
///
/// `tools/data/iching.json` の卦名（1 字目が上卦・2 字目が下卦）から求めた表。
/// 八卦のビット（下の爻が bit0）：天 111・沢 011・火 101・雷 001・風 110・水 010・山 100・地 000。
pub const KING_WEN: [u8; 64] = [
    2, 24, 7, 19, 15, 36, 46, 11, 16, 51, 40, 54, 62, 55, 32, 34, 8, 3, 29, 60, 39, 63, 48, 5, 45, 17,
    47, 58, 31, 49, 28, 43, 23, 27, 4, 41, 52, 22, 18, 26, 35, 21, 64, 38, 56, 30, 50, 14, 20, 42, 59,
    61, 53, 37, 57, 9, 12, 25, 6, 10, 33, 13, 44, 1,
];

/// 1 本の爻（硬貨 3 枚の合計 6〜9）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    /// 硬貨 3 枚の表裏（true=表）。
    pub coins: [bool; 3],
}

impl Line {
    /// 合計（表=3・裏=2）。
    pub fn value(self) -> u8 {
        self.coins.iter().map(|&h| if h { 3 } else { 2 }).sum()
    }

    /// 陽か（7・9）。
    pub fn is_yang(self) -> bool {
        self.value() % 2 == 1
    }

    /// 変爻か（6・9）。
    pub fn is_changing(self) -> bool {
        matches!(self.value(), 6 | 9)
    }
}

/// 硬貨 3 枚を投げて 1 本の爻を立てる。
pub fn cast_line<R: RandomSource>(rng: &mut R) -> Line {
    Line {
        coins: [bernoulli(rng), bernoulli(rng), bernoulli(rng)],
    }
}

/// 6 本の爻（下から上）から本卦の 6 ビット値を作る。
pub fn primary(lines: &[Line; LINES]) -> u8 {
    lines
        .iter()
        .enumerate()
        .fold(0, |acc, (i, l)| acc | (u8::from(l.is_yang()) << i))
}

/// 之卦（変爻の陰陽を反転した卦）の 6 ビット値。変爻が無ければ `None`。
pub fn changed(lines: &[Line; LINES]) -> Option<u8> {
    let mask = lines
        .iter()
        .enumerate()
        .fold(0u8, |acc, (i, l)| acc | (u8::from(l.is_changing()) << i));
    (mask != 0).then(|| primary(lines) ^ mask)
}

/// 6 ビット値の卦の番号（1〜64）。
pub fn number(code: u8) -> u8 {
    KING_WEN[usize::from(code & 0x3F)]
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

    fn line(v: u8) -> Line {
        // v = 6〜9 になる表裏（表の枚数 = v - 6）。
        let heads = v - 6;
        Line {
            coins: [heads > 0, heads > 1, heads > 2],
        }
    }

    #[test]
    fn values_follow_three_coin_odds() {
        // D-6：6 と 9 が 1/8、7 と 8 が 3/8。
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x1C41_0006);
        let mut counts = [0u32; 10];
        for _ in 0..TRIALS {
            counts[usize::from(cast_line(&mut rng).value())] += 1;
        }
        for (v, eighths) in [(6, 1), (7, 3), (8, 3), (9, 1)] {
            let expected = TRIALS / 8 * eighths;
            assert!(counts[v].abs_diff(expected) < expected / 20, "counts={counts:?}");
        }
        assert_eq!(counts[..6].iter().sum::<u32>(), 0);
    }

    #[test]
    fn line_kinds() {
        assert!(!line(6).is_yang() && line(6).is_changing()); // 老陰
        assert!(line(7).is_yang() && !line(7).is_changing()); // 少陽
        assert!(!line(8).is_yang() && !line(8).is_changing()); // 少陰
        assert!(line(9).is_yang() && line(9).is_changing()); // 老陽
    }

    #[test]
    fn king_wen_is_a_permutation() {
        let mut seen = [false; 65];
        for &n in &KING_WEN {
            assert!((1..=64).contains(&n));
            assert!(!seen[usize::from(n)], "duplicate {n}");
            seen[usize::from(n)] = true;
        }
    }

    #[test]
    fn known_hexagrams() {
        assert_eq!(number(0b111_111), 1); // 乾為天
        assert_eq!(number(0b000_000), 2); // 坤為地
        assert_eq!(number(0b000_111), 11); // 地天泰（下 乾・上 坤）
        assert_eq!(number(0b111_000), 12); // 天地否
        assert_eq!(number(0b010_101), 63); // 水火既済（下 離・上 坎）
        assert_eq!(number(0b101_010), 64); // 火水未済
        assert_eq!(number(0b010_001), 3); // 水雷屯（下 震・上 坎）
    }

    #[test]
    fn changing_lines_make_the_second_hexagram() {
        // 地天泰で三爻が老陽・五爻が老陰 → 水沢節（60）。
        let lines = [line(7), line(7), line(9), line(8), line(6), line(8)];
        assert_eq!(number(primary(&lines)), 11);
        assert_eq!(changed(&lines).map(number), Some(60));
        // 変爻が無ければ之卦は無い。
        let still = [line(7), line(8), line(7), line(8), line(7), line(8)];
        assert_eq!(changed(&still), None);
    }
}
