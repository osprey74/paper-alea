//! `config.json`（microSD の `/alea/config.json`・DESIGN.md §6.3・§7）。
//!
//! 実機のチューニング用にシェイク検出のパラメータを上書きする。JSON の汎用パーサは持たず、
//! キー名を探してその直後の整数だけを読む（キーは全体で一意な名前にする）。
//! 無いキー・数でない値・範囲外の値は無視して既定値のままにする。
//!
//! ```json
//! { "shake": { "threshold_mg": 800, "start_peaks": 2, "start_window_ms": 600, "end_quiet_ms": 300 } }
//! ```

use crate::shake::ShakeParams;

/// `"key"` の直後（`:` の後）にある整数を読む。
pub fn int_value(text: &str, key: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    let pat_len = key.len() + 2;
    let mut i = 0;
    while i + pat_len <= bytes.len() {
        if bytes[i] == b'"'
            && text.get(i + 1..i + 1 + key.len()) == Some(key)
            && bytes[i + 1 + key.len()] == b'"'
        {
            let rest = text[i + pat_len..].trim_start();
            let rest = rest.strip_prefix(':')?.trim_start();
            let (neg, digits) = match rest.strip_prefix('-') {
                Some(r) => (true, r),
                None => (false, rest),
            };
            let end = digits
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(digits.len());
            let v: i64 = digits[..end].parse().ok()?;
            return Some(if neg { -v } else { v });
        }
        i += 1;
    }
    None
}

/// 許す範囲。
pub const THRESHOLD_MG: (i64, i64) = (200, 4000);
/// 許す範囲（検出器が覚えるピークは 8 個まで）。
pub const START_PEAKS: (i64, i64) = (1, 8);
/// 許す範囲 [ms]。
pub const START_WINDOW_MS: (i64, i64) = (100, 3000);
/// 許す範囲 [ms]。
pub const END_QUIET_MS: (i64, i64) = (100, 3000);

/// `config.json` の中身からシェイクのパラメータを作る。戻り値の 2 つ目は上書きしたキーの数。
pub fn shake_params(text: &str) -> (ShakeParams, u8) {
    let mut p = ShakeParams::DEFAULT;
    let mut n = 0;
    let mut get = |key: &str, (lo, hi): (i64, i64)| {
        let v = int_value(text, key).filter(|v| (lo..=hi).contains(v))?;
        n += 1;
        Some(v)
    };
    if let Some(v) = get("threshold_mg", THRESHOLD_MG) {
        p.threshold_mg = v as u32;
    }
    if let Some(v) = get("start_peaks", START_PEAKS) {
        p.start_peaks = v as u8;
    }
    if let Some(v) = get("start_window_ms", START_WINDOW_MS) {
        p.start_window_ms = v as u32;
    }
    if let Some(v) = get("end_quiet_ms", END_QUIET_MS) {
        p.end_quiet_ms = v as u32;
    }
    (p, n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_nested_values() {
        let text = r#"{
          "shake": { "threshold_mg": 1200, "start_peaks" : 3,
                     "start_window_ms":800, "end_quiet_ms": 250 }
        }"#;
        let (p, n) = shake_params(text);
        assert_eq!(n, 4);
        assert_eq!(
            p,
            ShakeParams {
                threshold_mg: 1200,
                start_peaks: 3,
                start_window_ms: 800,
                end_quiet_ms: 250,
            }
        );
    }

    #[test]
    fn missing_or_bad_values_keep_defaults() {
        let text = r#"{ "shake": { "threshold_mg": "800", "start_peaks": 99, "end_quiet_ms": -5 } }"#;
        let (p, n) = shake_params(text);
        assert_eq!(n, 0);
        assert_eq!(p, ShakeParams::DEFAULT);
        assert_eq!(shake_params(""), (ShakeParams::DEFAULT, 0));
        assert_eq!(shake_params("not json"), (ShakeParams::DEFAULT, 0));
    }

    #[test]
    fn partial_override() {
        let (p, n) = shake_params(r#"{"shake":{"threshold_mg":600}}"#);
        assert_eq!(n, 1);
        assert_eq!(p.threshold_mg, 600);
        assert_eq!(p.start_peaks, ShakeParams::DEFAULT.start_peaks);
    }

    #[test]
    fn key_must_match_whole_name() {
        // "x_threshold_mg" は "threshold_mg" ではない。
        assert_eq!(int_value(r#"{"x_threshold_mg": 5}"#, "threshold_mg"), None);
        assert_eq!(int_value(r#"{"threshold_mg": 5}"#, "threshold_mg"), Some(5));
    }
}
