//! microSD の画像ファイル（DESIGN.md §8）。
//!
//! - `.a2b`（Alea 2-bit）：16 バイトのヘッダ（`"A2B1"`・幅・高さ・予約）＋ 2bit/画素（0=黒〜3=白）。
//! - `.a1b`（Alea 1-bit）：16 バイトのヘッダ（`"A1B1"`・幅・高さ・左上 x・y・予約）＋
//!   1bit/画素（行ごとにバイト境界・MSB から・1=黒）。黒い画素だけを重ねる。

use crate::services::display::Canvas;

/// ヘッダの長さ [byte]。
pub const HEADER: usize = 16;

fn u16_at(data: &[u8], i: usize) -> i32 {
    i32::from(u16::from_le_bytes([data[i], data[i + 1]]))
}

/// `.a2b` を `(x, y)` に描く（`rot180` なら 180° 回す）。形式が合わなければ false。
pub fn draw_a2b(canvas: &mut Canvas<'_>, data: &[u8], x: i32, y: i32, rot180: bool) -> bool {
    if data.len() < HEADER || &data[..4] != b"A2B1" {
        return false;
    }
    let (w, h) = (u16_at(data, 4), u16_at(data, 6));
    let body = &data[HEADER..];
    if body.len() < (w * h / 4) as usize {
        return false;
    }
    canvas.blit_2bpp_rot(x, y, w, h, body, rot180);
    true
}

/// `.a1b` をヘッダの位置に描く。形式が合わなければ false。
pub fn draw_a1b(canvas: &mut Canvas<'_>, data: &[u8]) -> bool {
    if data.len() < HEADER || &data[..4] != b"A1B1" {
        return false;
    }
    let (w, h) = (u16_at(data, 4), u16_at(data, 6));
    let (x, y) = (u16_at(data, 8), u16_at(data, 10));
    let body = &data[HEADER..];
    if body.len() < ((w as usize + 7) / 8) * h as usize {
        return false;
    }
    canvas.blit_1bpp(x, y, w, h, body);
    true
}
