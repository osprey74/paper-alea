//! Alea のハード非依存ロジック。
//!
//! ファーム（`firmware/alea-fw`）とホストの両方でビルドできるよう `no_std` とする。
//! 乱数源は [`RandomSource`] で抽象化し、実機では `esp_hal::rng`、テストでは決定的な
//! 擬似乱数を差し込む（DESIGN.md §6.4）。

#![no_std]

pub mod dice;
pub mod shake;
pub mod tarot;

/// 32 ビット一様乱数の供給源。
pub trait RandomSource {
    /// 0〜`u32::MAX` の一様乱数を 1 つ返す。
    fn next_u32(&mut self) -> u32;
}

/// 0〜`n - 1` の一様乱数を返す。
///
/// 剰余による偏りを避けるため、`u32` の値域を `n` の倍数に切り詰め、
/// はみ出した値は捨てて引き直す（棄却サンプリング）。
///
/// # Panics
/// `n == 0` のとき。
pub fn uniform<R: RandomSource>(rng: &mut R, n: u32) -> u32 {
    assert!(n > 0, "uniform: n must be > 0");
    // 2^32 を n で割った余り。これ未満の値を捨てると残りが n の倍数個になる。
    let reject_below = n.wrapping_neg() % n;
    loop {
        let x = rng.next_u32();
        if x >= reject_below {
            return x % n;
        }
    }
}

/// 0 か 1 を等確率で返す（コイン・正逆位置用）。
pub fn bernoulli<R: RandomSource>(rng: &mut R) -> bool {
    rng.next_u32() & 1 == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// テスト用の決定的な擬似乱数（xorshift32）。
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

    /// 常に同じ値を返す乱数源（棄却の確認用）。
    struct Seq<'a>(&'a [u32], usize);

    impl RandomSource for Seq<'_> {
        fn next_u32(&mut self) -> u32 {
            let v = self.0[self.1];
            self.1 += 1;
            v
        }
    }

    #[test]
    fn uniform_stays_in_range() {
        let mut rng = XorShift(0x1234_5678);
        for n in [1, 2, 3, 6, 10, 20, 24, 64, 78, 100] {
            for _ in 0..10_000 {
                assert!(uniform(&mut rng, n) < n);
            }
        }
    }

    #[test]
    fn uniform_rejects_biased_tail() {
        // n=3: 2^32 mod 3 = 1 なので 0 は捨てて次の値を使う。
        let mut rng = Seq(&[0, 5], 0);
        assert_eq!(uniform(&mut rng, 3), 5 % 3);
        assert_eq!(rng.1, 2);
    }

    #[test]
    fn uniform_distribution_is_flat() {
        // 10 万回試行で各値の出現数が期待値の ±5% 以内（DESIGN.md §12 テスト方針）。
        const N: u32 = 6;
        const TRIALS: u32 = 100_000;
        let mut rng = XorShift(0xDEAD_BEEF);
        let mut counts = [0u32; N as usize];
        for _ in 0..TRIALS {
            counts[uniform(&mut rng, N) as usize] += 1;
        }
        let expected = TRIALS / N;
        for c in counts {
            assert!(c.abs_diff(expected) < expected / 20, "counts={counts:?}");
        }
    }
}
