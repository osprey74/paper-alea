// tools/render_apps.py が生成。手で編集しない。

/// ダイスの台紙（チップはすべて未選択の姿）。
pub const DICE_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 0 };
/// コインの台紙。
pub const COIN_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 44004 };
/// 是か非かの台紙（結果の下の罫を含む）。
pub const YESNO_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 88008 };
/// 選択チップの矩形 (x, y, w, h)。`DiceKind::ALL` の順。
pub const DICE_CHIPS: [(i32, i32, i32, i32); 10] = [
    (40, 134, 75, 64), // 1D3
    (121, 134, 75, 64), // 1D4
    (202, 134, 76, 64), // 1D6
    (284, 134, 75, 64), // 2D6
    (365, 134, 75, 64), // 3D6
    (40, 204, 75, 64), // 1D8
    (121, 204, 75, 64), // 1D10
    (202, 204, 76, 64), // 1D12
    (284, 204, 75, 64), // 1D20
    (365, 204, 75, 64), // 1D100
];
/// 2 個・3 個のダイスと 1D100 の中心の高さ。
pub const DICE_ROW_Y: i32 = 414;
/// 1 個のダイスの中心の高さ。
pub const DICE_SINGLE_Y: i32 = 492;
/// 「合計」の中心の高さ（2D6・3D6）。
pub const TOTAL_LABEL_Y_D6: i32 = 524;
/// 「合計」の中心の高さ（1D100）。
pub const TOTAL_LABEL_Y_D100: i32 = 534;
/// 「合計」から合計値の中心までの距離。
pub const TOTAL_VALUE_DY: i32 = 73;
/// 「合計」（中心からの相対）。
pub const TOTAL_LABEL: Sprite = Sprite { x: -27, y: -10, w: 55, h: 23, off: 132012 };
/// D6（210px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。
pub const D6_L: [Sprite; 7] = [
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 132173 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 137425 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 142677 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 147929 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 153181 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 158433 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 163685 },
];
/// D6（150px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。
pub const D6_M: [Sprite; 7] = [
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 168937 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 171529 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 174121 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 176713 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 179305 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 181897 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 184489 },
];
/// D6（120px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。
pub const D6_S: [Sprite; 7] = [
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 187081 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 188806 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 190531 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 192256 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 193981 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 195706 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 197431 },
];
/// triangle の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_TRIANGLE: Die = Die { outline: Sprite { x: -109, y: -107, w: 218, h: 190, off: 199156 }, text_dy: 18 };
/// diamond の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_DIAMOND: Die = Die { outline: Sprite { x: -109, y: -116, w: 218, h: 232, off: 204476 }, text_dy: 0 };
/// kite の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_KITE: Die = Die { outline: Sprite { x: -106, y: -116, w: 212, h: 232, off: 210972 }, text_dy: -6 };
/// pentagon の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_PENTAGON: Die = Die { outline: Sprite { x: -114, y: -113, w: 229, h: 218, off: 217236 }, text_dy: 7 };
/// hexagon の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_HEXAGON: Die = Die { outline: Sprite { x: -105, y: -120, w: 210, h: 240, off: 223558 }, text_dy: 0 };
/// kite の輪郭（倍率 1.0・中心からの相対）と数字の中心のずれ。
pub const DIE_M_KITE: Die = Die { outline: Sprite { x: -76, y: -83, w: 152, h: 166, off: 230038 }, text_dy: -4 };
/// 数字 120px（Cormorant Garamond Bold）。
pub const FONT_TOTAL: [Glyph; 11] = [
    Glyph { ch: b'0', adv: 916, sprite: Sprite { x: 3, y: -10, w: 51, h: 50, off: 233192 } },
    Glyph { ch: b'1', adv: 636, sprite: Sprite { x: 5, y: -8, w: 30, h: 46, off: 233542 } },
    Glyph { ch: b'2', adv: 772, sprite: Sprite { x: 4, y: -11, w: 41, h: 49, off: 233726 } },
    Glyph { ch: b'3', adv: 756, sprite: Sprite { x: 3, y: -11, w: 40, h: 82, off: 234020 } },
    Glyph { ch: b'4', adv: 876, sprite: Sprite { x: 3, y: -9, w: 48, h: 70, off: 234430 } },
    Glyph { ch: b'5', adv: 784, sprite: Sprite { x: 7, y: -13, w: 36, h: 84, off: 234850 } },
    Glyph { ch: b'6', adv: 892, sprite: Sprite { x: 5, y: -41, w: 48, h: 81, off: 235270 } },
    Glyph { ch: b'7', adv: 824, sprite: Sprite { x: 2, y: -12, w: 45, h: 83, off: 235756 } },
    Glyph { ch: b'8', adv: 936, sprite: Sprite { x: 5, y: -31, w: 50, h: 71, off: 236254 } },
    Glyph { ch: b'9', adv: 892, sprite: Sprite { x: 3, y: -10, w: 48, h: 81, off: 236751 } },
    Glyph { ch: b'?', adv: 652, sprite: Sprite { x: 4, y: -37, w: 34, h: 76, off: 237237 } },
];
/// 数字 58px（Cormorant Garamond Bold）。
pub const FONT_DIE_M: [Glyph; 11] = [
    Glyph { ch: b'0', adv: 444, sprite: Sprite { x: 2, y: -5, w: 24, h: 24, off: 237617 } },
    Glyph { ch: b'1', adv: 312, sprite: Sprite { x: 3, y: -4, w: 14, h: 23, off: 237689 } },
    Glyph { ch: b'2', adv: 376, sprite: Sprite { x: 2, y: -5, w: 20, h: 23, off: 237735 } },
    Glyph { ch: b'3', adv: 364, sprite: Sprite { x: 2, y: -5, w: 19, h: 40, off: 237804 } },
    Glyph { ch: b'4', adv: 420, sprite: Sprite { x: 2, y: -4, w: 22, h: 33, off: 237924 } },
    Glyph { ch: b'5', adv: 380, sprite: Sprite { x: 4, y: -6, w: 17, h: 40, off: 238023 } },
    Glyph { ch: b'6', adv: 436, sprite: Sprite { x: 2, y: -20, w: 24, h: 39, off: 238143 } },
    Glyph { ch: b'7', adv: 400, sprite: Sprite { x: 1, y: -5, w: 22, h: 39, off: 238260 } },
    Glyph { ch: b'8', adv: 456, sprite: Sprite { x: 2, y: -15, w: 25, h: 34, off: 238377 } },
    Glyph { ch: b'9', adv: 432, sprite: Sprite { x: 2, y: -5, w: 23, h: 39, off: 238513 } },
    Glyph { ch: b'?', adv: 312, sprite: Sprite { x: 2, y: -18, w: 16, h: 37, off: 238630 } },
];
/// 数字 84px（Cormorant Garamond Bold）。
pub const FONT_DIE_L: [Glyph; 11] = [
    Glyph { ch: b'0', adv: 644, sprite: Sprite { x: 2, y: -7, w: 36, h: 35, off: 238704 } },
    Glyph { ch: b'1', adv: 448, sprite: Sprite { x: 4, y: -6, w: 21, h: 33, off: 238879 } },
    Glyph { ch: b'2', adv: 540, sprite: Sprite { x: 3, y: -7, w: 29, h: 34, off: 238978 } },
    Glyph { ch: b'3', adv: 528, sprite: Sprite { x: 2, y: -7, w: 28, h: 57, off: 239114 } },
    Glyph { ch: b'4', adv: 612, sprite: Sprite { x: 2, y: -7, w: 34, h: 49, off: 239342 } },
    Glyph { ch: b'5', adv: 548, sprite: Sprite { x: 5, y: -9, w: 25, h: 59, off: 239587 } },
    Glyph { ch: b'6', adv: 620, sprite: Sprite { x: 3, y: -29, w: 34, h: 57, off: 239823 } },
    Glyph { ch: b'7', adv: 576, sprite: Sprite { x: 1, y: -8, w: 32, h: 58, off: 240108 } },
    Glyph { ch: b'8', adv: 656, sprite: Sprite { x: 3, y: -22, w: 36, h: 50, off: 240340 } },
    Glyph { ch: b'9', adv: 624, sprite: Sprite { x: 2, y: -7, w: 34, h: 57, off: 240590 } },
    Glyph { ch: b'?', adv: 456, sprite: Sprite { x: 3, y: -26, w: 24, h: 54, off: 240875 } },
];
/// コインの面（heads）。
pub const COIN_HEADS: Sprite = Sprite { x: 90, y: 221, w: 300, h: 300, off: 241037 };
/// コインの面（tails）。
pub const COIN_TAILS: Sprite = Sprite { x: 90, y: 221, w: 300, h: 300, off: 252437 };
/// コインの面（wait）。
pub const COIN_WAIT: Sprite = Sprite { x: 90, y: 221, w: 300, h: 300, off: 263837 };
/// HEADS。
pub const COIN_WORD_HEADS: Sprite = Sprite { x: 126, y: 561, w: 227, h: 33, off: 275237 };
/// TAILS。
pub const COIN_WORD_TAILS: Sprite = Sprite { x: 139, y: 560, w: 201, h: 34, off: 276194 };
/// 表。
pub const COIN_KANJI_HEADS: Sprite = Sprite { x: 230, y: 617, w: 21, h: 22, off: 277078 };
/// 裏。
pub const COIN_KANJI_TAILS: Sprite = Sprite { x: 230, y: 617, w: 20, h: 22, off: 277144 };
/// YES。
pub const YESNO_YES: Sprite = Sprite { x: 102, y: 322, w: 269, h: 97, off: 277210 };
/// NO。
pub const YESNO_NO: Sprite = Sprite { x: 124, y: 322, w: 227, h: 98, off: 280508 };
/// ?。
pub const YESNO_WAIT: Sprite = Sprite { x: 219, y: 324, w: 43, h: 95, off: 283350 };
/// 是。
pub const YESNO_KANJI_YES: Sprite = Sprite { x: 220, y: 518, w: 39, h: 41, off: 283920 };
/// 非。
pub const YESNO_KANJI_NO: Sprite = Sprite { x: 220, y: 518, w: 40, h: 41, off: 284125 };
