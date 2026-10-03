//! アプリ画面の画像部品（ダイス・コイン・是か非か）。
//!
//! `tools/render_apps.py` が確定デザイン（Claude Design「Alea ランチャー」2 段目）から生成した
//! 1bit の部品と配置の定数を使う。台紙を貼り、結果の部品と数字の字形を黒で重ねる。

use embedded_graphics::prelude::Point;

use crate::services::display::Canvas;

/// 1bit の部品（行ごとにバイト境界・MSB から・1=黒）。
#[derive(Clone, Copy)]
pub struct Sprite {
    /// 基準点からの左上の位置。
    pub x: i16,
    /// 同上。
    pub y: i16,
    /// 幅 [px]。
    pub w: u16,
    /// 高さ [px]。
    pub h: u16,
    /// [`DATA`] の中の位置 [byte]。
    pub off: u32,
}

/// 数字の字形。位置はペン位置（左端）と文字の中心線からの相対。
#[derive(Clone, Copy)]
pub struct Glyph {
    /// 文字。
    pub ch: u8,
    /// 送り幅 [1/16 px]。
    pub adv: u16,
    /// 字形。
    pub sprite: Sprite,
}

/// D6 以外のダイスの輪郭（中心からの相対）と、数字の中心の縦のずれ。
#[derive(Clone, Copy)]
pub struct Die {
    /// 輪郭。
    pub outline: Sprite,
    /// 数字の中心のずれ [px]。
    pub text_dy: i16,
}

static DATA: &[u8] = include_bytes!("../../assets/app_art.1bpp");

include!("../../assets/app_art.rs");

/// 部品を基準点 `at` に描く（固定位置の部品は `Point::zero()`）。
pub fn draw(canvas: &mut Canvas<'_>, s: &Sprite, at: Point) {
    let stride = (usize::from(s.w) + 7) / 8;
    let off = s.off as usize;
    let Some(data) = DATA.get(off..off + stride * usize::from(s.h)) else {
        return;
    };
    canvas.blit_1bpp(
        at.x + i32::from(s.x),
        at.y + i32::from(s.y),
        i32::from(s.w),
        i32::from(s.h),
        data,
    );
}

/// 数字の文字列を、字の上端と下端の中央が `center` に来るように描く。
/// オールドスタイル数字は字によって高さと位置が違う（0・1・2 は小文字の高さ、3・4・7・9 は下に出る）ため、
/// 図形の中に置く数字は行の中央ではなく、実際の字の範囲で上下を合わせる。
pub fn text_ink_centered(canvas: &mut Canvas<'_>, font: &[Glyph], s: &str, center: Point) {
    let glyphs = s.bytes().filter_map(|c| font.iter().find(|g| g.ch == c));
    let top = glyphs.clone().map(|g| i32::from(g.sprite.y)).min();
    let bottom = glyphs
        .map(|g| i32::from(g.sprite.y) + i32::from(g.sprite.h))
        .max();
    let dy = match (top, bottom) {
        (Some(t), Some(b)) => -(t + b) / 2,
        _ => 0,
    };
    text(canvas, font, s, center + Point::new(0, dy));
}

/// 数字の文字列を `center` を中心に描く（上下は書体の行の中央・字形の無い文字は飛ばす）。
pub fn text(canvas: &mut Canvas<'_>, font: &[Glyph], s: &str, center: Point) {
    let find = |c: u8| font.iter().find(|g| g.ch == c);
    let total: i32 = s.bytes().filter_map(find).map(|g| i32::from(g.adv)).sum();
    let mut pen = center.x * 16 - total / 2;
    for g in s.bytes().filter_map(find) {
        draw(canvas, &g.sprite, Point::new((pen + 8) / 16, center.y));
        pen += i32::from(g.adv);
    }
}
