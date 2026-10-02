//! ダイス（DESIGN.md §10.1）。
//!
//! 種類：1D3 / 1D4 / 1D6 / 2D6 / 3D6 / 1D8 / 1D10 / 1D12 / 1D20 / 1D100。
//! 1D100 は十の位 D10（00〜90）と一の位 D10（0〜9）を振り、「00＋0」を 100 とする。

use crate::{uniform, RandomSource};

/// ダイスの種類。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiceKind {
    /// 1D3。
    D3,
    /// 1D4。
    D4,
    /// 1D6。
    D6,
    /// 2D6。
    TwoD6,
    /// 3D6。
    ThreeD6,
    /// 1D8。
    D8,
    /// 1D10。
    D10,
    /// 1D12。
    D12,
    /// 1D20。
    D20,
    /// 1D100（十の位と一の位の D10 二つ）。
    D100,
}

impl DiceKind {
    /// 選択チップの並び順。
    pub const ALL: [DiceKind; 10] = [
        DiceKind::D3,
        DiceKind::D4,
        DiceKind::D6,
        DiceKind::TwoD6,
        DiceKind::ThreeD6,
        DiceKind::D8,
        DiceKind::D10,
        DiceKind::D12,
        DiceKind::D20,
        DiceKind::D100,
    ];

    /// 表示名。
    pub const fn label(self) -> &'static str {
        match self {
            DiceKind::D3 => "1D3",
            DiceKind::D4 => "1D4",
            DiceKind::D6 => "1D6",
            DiceKind::TwoD6 => "2D6",
            DiceKind::ThreeD6 => "3D6",
            DiceKind::D8 => "1D8",
            DiceKind::D10 => "1D10",
            DiceKind::D12 => "1D12",
            DiceKind::D20 => "1D20",
            DiceKind::D100 => "1D100",
        }
    }

    /// 振るダイスの個数と面数（1D100 は別扱い）。
    pub const fn count_and_sides(self) -> (usize, u32) {
        match self {
            DiceKind::D3 => (1, 3),
            DiceKind::D4 => (1, 4),
            DiceKind::D6 => (1, 6),
            DiceKind::TwoD6 => (2, 6),
            DiceKind::ThreeD6 => (3, 6),
            DiceKind::D8 => (1, 8),
            DiceKind::D10 => (1, 10),
            DiceKind::D12 => (1, 12),
            DiceKind::D20 => (1, 20),
            DiceKind::D100 => (2, 10),
        }
    }
}

/// 振った結果。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Roll {
    /// 種類。
    pub kind: DiceKind,
    /// 各ダイスの出目（1D100 は `[十の位 0〜90, 一の位 0〜9]`）。
    pub faces: [u32; 3],
    /// ダイスの個数。
    pub count: usize,
    /// 合計（1D100 は 1〜100）。
    pub total: u32,
}

/// `kind` を振る。
pub fn roll<R: RandomSource>(rng: &mut R, kind: DiceKind) -> Roll {
    let mut faces = [0u32; 3];
    if kind == DiceKind::D100 {
        let tens = uniform(rng, 10) * 10;
        let ones = uniform(rng, 10);
        faces[0] = tens;
        faces[1] = ones;
        let total = if tens + ones == 0 { 100 } else { tens + ones };
        return Roll {
            kind,
            faces,
            count: 2,
            total,
        };
    }
    let (count, sides) = kind.count_and_sides();
    let mut total = 0;
    for face in faces.iter_mut().take(count) {
        *face = uniform(rng, sides) + 1;
        total += *face;
    }
    Roll {
        kind,
        faces,
        count,
        total,
    }
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

    /// 決まった値を順に返す乱数源。
    struct Seq<'a>(&'a [u32], usize);

    impl RandomSource for Seq<'_> {
        fn next_u32(&mut self) -> u32 {
            let v = self.0[self.1];
            self.1 += 1;
            v
        }
    }

    #[test]
    fn d100_double_zero_is_100() {
        // 十の位 0（→00）・一の位 0 → 100。棄却されない小さな値を使う。
        let mut rng = Seq(&[10, 20], 0);
        let r = roll(&mut rng, DiceKind::D100);
        assert_eq!(r.faces[..2], [0, 0]);
        assert_eq!(r.total, 100);
    }

    #[test]
    fn d100_is_uniform_over_1_to_100() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x0BAD_F00D);
        let mut counts = [0u32; 101];
        for _ in 0..TRIALS {
            let r = roll(&mut rng, DiceKind::D100);
            assert!((1..=100).contains(&r.total));
            counts[r.total as usize] += 1;
        }
        let expected = TRIALS / 100;
        for c in &counts[1..] {
            // 期待値 1000 に対して ±15%（約 5σ）。
            assert!(c.abs_diff(expected) < expected * 15 / 100, "counts={counts:?}");
        }
    }

    #[test]
    fn every_kind_stays_in_range() {
        let mut rng = XorShift(0x1357_9BDF);
        for kind in DiceKind::ALL {
            let (count, sides) = kind.count_and_sides();
            for _ in 0..10_000 {
                let r = roll(&mut rng, kind);
                assert_eq!(r.count, count);
                if kind == DiceKind::D100 {
                    continue;
                }
                let sum: u32 = r.faces[..count].iter().sum();
                assert_eq!(sum, r.total);
                assert!(r.faces[..count].iter().all(|&f| (1..=sides).contains(&f)));
            }
        }
    }

    #[test]
    fn d3_is_uniform() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0xACE1_2345);
        let mut counts = [0u32; 4];
        for _ in 0..TRIALS {
            counts[roll(&mut rng, DiceKind::D3).total as usize] += 1;
        }
        let expected = TRIALS / 3;
        for c in &counts[1..] {
            assert!(c.abs_diff(expected) < expected / 20, "counts={counts:?}");
        }
    }
}
