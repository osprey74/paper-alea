//! 画面の座標定数（縦持ち 480×800・DESIGN.md §9）。

/// 画面幅 [px]。
pub const SCREEN_W: i32 = 480;

/// 見出し帯の高さ [px]。
pub const HEADER_H: i32 = 56;

/// 本文の左右余白 [px]。
pub const MARGIN_X: i32 = 24;

/// 本文の 1 行の高さ（FONT_10X20 の行送り）[px]。
pub const LINE_H: i32 = 28;

/// ランチャーのタイル列数。
pub const TILE_COLS: i32 = 2;
/// ランチャーのタイル段数。
pub const TILE_ROWS: i32 = 5;
/// ランチャーのタイル幅 [px]。
pub const TILE_W: i32 = SCREEN_W / TILE_COLS;
/// ランチャーのタイル高 [px]（56 + 148×5 = 796）。
pub const TILE_H: i32 = 148;
/// ランチャーのタイル数。
pub const TILE_COUNT: usize = (TILE_COLS * TILE_ROWS) as usize;
