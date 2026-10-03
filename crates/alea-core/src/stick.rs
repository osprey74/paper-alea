//! 棒倒し（DESIGN.md §10.3）。左右（2 方向）か 8 方位のどちらかに倒す。

use crate::{bernoulli, uniform, RandomSource};

/// 方式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// 左右。
    LeftRight,
    /// 8 方位。
    Eight,
}

/// 倒れた向き。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fall {
    /// 左（`false`）か右（`true`）。
    Side(bool),
    /// 8 方位（0=北、時計回りに 45° ずつ。1=北東 … 7=北西）。
    Dir(u8),
}

/// 倒す。
pub fn fall<R: RandomSource>(rng: &mut R, mode: Mode) -> Fall {
    match mode {
        Mode::LeftRight => Fall::Side(bernoulli(rng)),
        Mode::Eight => Fall::Dir(uniform(rng, 8) as u8),
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

    #[test]
    fn eight_directions_are_uniform() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x5717_C0DE);
        let mut counts = [0u32; 8];
        for _ in 0..TRIALS {
            match fall(&mut rng, Mode::Eight) {
                Fall::Dir(d) => counts[usize::from(d)] += 1,
                Fall::Side(_) => panic!("mode mismatch"),
            }
        }
        let expected = TRIALS / 8;
        for c in counts {
            // 期待値 12,500 に対して ±5%（約 6σ）。
            assert!(c.abs_diff(expected) < expected / 20, "counts={counts:?}");
        }
    }

    #[test]
    fn left_right_is_about_half() {
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0x0123_4567);
        let right = (0..TRIALS)
            .filter(|_| fall(&mut rng, Mode::LeftRight) == Fall::Side(true))
            .count() as u32;
        assert!(right.abs_diff(TRIALS / 2) < TRIALS / 100, "right={right}");
    }
}
