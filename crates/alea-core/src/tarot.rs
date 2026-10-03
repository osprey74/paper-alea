//! タロット（ワンオラクル・DESIGN.md §10）。
//!
//! 78 枚（大アルカナ 0〜21・小アルカナ 22〜77）から 1 枚を引き、正位置か逆位置かを決める。
//! 番号はカード画像のファイル名（`NN.a2b`）と同じ。

use crate::{bernoulli, uniform, RandomSource};

/// カードの枚数。
pub const CARD_COUNT: u32 = 78;

/// 引いたカード。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Draw {
    /// カード番号（0〜77）。
    pub card: u8,
    /// 逆位置か。
    pub reversed: bool,
}

/// 1 枚引く。
pub fn draw<R: RandomSource>(rng: &mut R) -> Draw {
    let card = uniform(rng, CARD_COUNT) as u8;
    let reversed = bernoulli(rng);
    Draw { card, reversed }
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

    #[test]
    fn cards_are_uniform_and_in_range() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x7A20_7A20);
        let mut counts = [0u32; CARD_COUNT as usize];
        for _ in 0..TRIALS {
            let d = draw(&mut rng);
            assert!(u32::from(d.card) < CARD_COUNT);
            counts[usize::from(d.card)] += 1;
        }
        // 期待値 約 1282 に対して ±15%（約 5σ）。
        let expected = TRIALS / CARD_COUNT;
        for c in counts {
            assert!(c.abs_diff(expected) < expected * 15 / 100, "counts={counts:?}");
        }
    }

    #[test]
    fn reversed_is_about_half() {
        // M3 の完了条件：正逆が 50% 前後に分布する。
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x0BAD_CAFE);
        let reversed = (0..TRIALS).filter(|_| draw(&mut rng).reversed).count() as u32;
        // 期待値 50,000 に対して ±1%（約 6σ）。
        assert!(reversed.abs_diff(TRIALS / 2) < TRIALS / 100, "reversed={reversed}");
    }

    #[test]
    fn reversal_does_not_depend_on_card() {
        // カードごとの逆位置の割合が極端に偏らない（同じ乱数から番号と向きを取る相関の確認）。
        const TRIALS: u32 = 400_000;
        let mut rng = XorShift(0x1234_ABCD);
        let mut total = [0u32; CARD_COUNT as usize];
        let mut rev = [0u32; CARD_COUNT as usize];
        for _ in 0..TRIALS {
            let d = draw(&mut rng);
            total[usize::from(d.card)] += 1;
            rev[usize::from(d.card)] += u32::from(d.reversed);
        }
        for (t, r) in total.iter().zip(rev) {
            // 1 枚あたり約 5,128 回。逆位置の割合は 50% ±5%（約 7σ）。
            assert!(r * 100 / t >= 45 && r * 100 / t <= 55, "card total={t} reversed={r}");
        }
    }
}
