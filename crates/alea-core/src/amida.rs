//! あみだくじ（DESIGN.md §10.4）。
//!
//! 縦線 `n` 本（2〜6）の間に、`LEVELS` 段の高さのどこかに横線を引く。
//! - 同じ高さで隣り合う横線は引かない（どちらに曲がるか決まらなくなるため）。
//! - どの隣り合う縦線の間にも横線を最低 1 本引く（素通りの線を作らない）。

use crate::{bernoulli, uniform, RandomSource};

/// 縦線の最小本数。
pub const MIN_LINES: usize = 2;
/// 縦線の最大本数（上の札の幅を 64px 以上に保つ・DESIGN.md §9）。
pub const MAX_LINES: usize = 6;
/// 横線を引く高さの段数。
pub const LEVELS: usize = 10;

/// あみだの横線。`rungs[段][間]` が true なら、その段で縦線 `間` と `間 + 1` をつなぐ。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ladder {
    /// 縦線の本数。
    pub lines: usize,
    /// 横線（使うのは各段の先頭 `lines - 1` 個）。
    pub rungs: [[bool; MAX_LINES - 1]; LEVELS],
}

impl Ladder {
    /// 縦線 `start` から下りて着く縦線と、通った道筋（段ごとの位置の変化）を返す。
    /// `path[k]` は段 `k` を通り過ぎた後にいる縦線。
    pub fn trace(&self, start: usize) -> (usize, [usize; LEVELS]) {
        let mut pos = start;
        let mut path = [0; LEVELS];
        for (k, row) in self.rungs.iter().enumerate() {
            if pos + 1 < self.lines && row[pos] {
                pos += 1;
            } else if pos > 0 && row[pos - 1] {
                pos -= 1;
            }
            path[k] = pos;
        }
        (pos, path)
    }
}

/// `lines` 本（`MIN_LINES`〜`MAX_LINES` に丸める）のあみだを作る。
pub fn generate<R: RandomSource>(rng: &mut R, lines: usize) -> Ladder {
    let lines = lines.clamp(MIN_LINES, MAX_LINES);
    let gaps = lines - 1;
    let mut rungs = [[false; MAX_LINES - 1]; LEVELS];
    for row in rungs.iter_mut() {
        for g in 0..gaps {
            let left_taken = g > 0 && row[g - 1];
            row[g] = !left_taken && bernoulli(rng);
        }
    }
    // 横線の無い間には、隣と重ならない段を選んで 1 本足す。
    for g in 0..gaps {
        if rungs.iter().any(|row| row[g]) {
            continue;
        }
        let free = |row: &[bool; MAX_LINES - 1]| {
            !(g > 0 && row[g - 1]) && !(g + 1 < gaps && row[g + 1])
        };
        let candidates = rungs.iter().filter(|row| free(row)).count() as u32;
        if candidates == 0 {
            // 両隣が全段埋まっている（確率はごく小さい）。隣の 1 本を外して空ける。
            let k = uniform(rng, LEVELS as u32) as usize;
            if g > 0 {
                rungs[k][g - 1] = false;
            }
            if g + 1 < gaps {
                rungs[k][g + 1] = false;
            }
            rungs[k][g] = true;
            continue;
        }
        let mut nth = uniform(rng, candidates);
        for row in rungs.iter_mut() {
            if free(row) {
                if nth == 0 {
                    row[g] = true;
                    break;
                }
                nth -= 1;
            }
        }
    }
    Ladder { lines, rungs }
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
    fn rules_hold_for_every_size() {
        let mut rng = XorShift(0xA41D_A001);
        for lines in MIN_LINES..=MAX_LINES {
            for _ in 0..20_000 {
                let l = generate(&mut rng, lines);
                assert_eq!(l.lines, lines);
                let gaps = lines - 1;
                for row in &l.rungs {
                    // 隣り合う横線が同じ高さに無い。使わない間には線が無い。
                    for g in 1..gaps {
                        assert!(!(row[g - 1] && row[g]), "adjacent rungs: {:?}", l);
                    }
                    assert!(row[gaps..].iter().all(|&r| !r));
                }
                for g in 0..gaps {
                    assert!(l.rungs.iter().any(|row| row[g]), "empty gap {g}: {:?}", l);
                }
            }
        }
    }

    #[test]
    fn trace_is_a_permutation() {
        let mut rng = XorShift(0x1357_2468);
        for lines in MIN_LINES..=MAX_LINES {
            for _ in 0..10_000 {
                let l = generate(&mut rng, lines);
                let mut seen = [false; MAX_LINES];
                for s in 0..lines {
                    let (end, path) = l.trace(s);
                    assert!(end < lines);
                    assert_eq!(path[LEVELS - 1], end);
                    assert!(!seen[end], "two starts reach {end}: {:?}", l);
                    seen[end] = true;
                }
            }
        }
    }

    #[test]
    fn every_goal_is_reachable() {
        // 一様ではない（あみだの性質）が、どの始点からもすべての行き先に着くことがある。
        let mut rng = XorShift(0x0F0F_A5A5);
        for lines in MIN_LINES..=MAX_LINES {
            let mut hit = [[0u32; MAX_LINES]; MAX_LINES];
            for _ in 0..20_000 {
                let l = generate(&mut rng, lines);
                for s in 0..lines {
                    hit[s][l.trace(s).0] += 1;
                }
            }
            for row in hit.iter().take(lines) {
                assert!(row[..lines].iter().all(|&c| c > 0), "lines={lines} hit={hit:?}");
            }
        }
    }
}
