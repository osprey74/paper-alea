// tools/render_apps.py が生成。手で編集しない。

/// ダイスの台紙（チップはすべて未選択の姿）。
pub const DICE_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 0 };
/// コインの台紙。
pub const COIN_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 44004 };
/// 是か非かの台紙（結果の下の罫を含む）。
pub const YESNO_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 88008 };
/// タロットの待機画面の台紙（裏面はカード画像を重ねる）。
pub const TAROT_WAIT_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 132012 };
/// 結果画面の札「正位置」（黒地に白抜き）。
pub const TAROT_CHIP_UPRIGHT: Sprite = Sprite { x: 194, y: 672, w: 92, h: 32, off: 176016 };
/// 結果画面の札「逆位置」（黒地に白抜き）。
pub const TAROT_CHIP_REVERSED: Sprite = Sprite { x: 194, y: 672, w: 92, h: 32, off: 176400 };
/// カード（360×540）の左上。
pub const TAROT_WAIT_CARD: (i32, i32) = (60, 162);
/// カード（360×540）の左上。
pub const TAROT_RESULT_CARD: (i32, i32) = (60, 38);
/// キーワード画面の札の矩形 (x, y, w, h)。[正位置, 逆位置]。今の向きの札の内側を反転する。
pub const TAROT_WORD_CHIPS: [(i32, i32, i32, i32); 2] = [(194, 199, 92, 32), (194, 479, 92, 32)];
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
pub const TOTAL_LABEL: Sprite = Sprite { x: -27, y: -10, w: 55, h: 23, off: 176784 };
/// D6（210px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。
pub const D6_L: [Sprite; 7] = [
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 176945 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 182197 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 187449 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 192701 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 197953 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 203205 },
    Sprite { x: -101, y: -101, w: 202, h: 202, off: 208457 },
];
/// D6（150px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。
pub const D6_M: [Sprite; 7] = [
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 213709 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 216301 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 218893 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 221485 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 224077 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 226669 },
    Sprite { x: -72, y: -72, w: 144, h: 144, off: 229261 },
];
/// D6（120px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。
pub const D6_S: [Sprite; 7] = [
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 231853 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 233578 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 235303 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 237028 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 238753 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 240478 },
    Sprite { x: -58, y: -57, w: 116, h: 115, off: 242203 },
];
/// triangle の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_TRIANGLE: Die = Die { outline: Sprite { x: -109, y: -107, w: 218, h: 190, off: 243928 }, text_dy: 18 };
/// diamond の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_DIAMOND: Die = Die { outline: Sprite { x: -109, y: -116, w: 218, h: 232, off: 249248 }, text_dy: 0 };
/// kite の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_KITE: Die = Die { outline: Sprite { x: -106, y: -116, w: 212, h: 232, off: 255744 }, text_dy: -6 };
/// pentagon の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_PENTAGON: Die = Die { outline: Sprite { x: -114, y: -113, w: 229, h: 218, off: 262008 }, text_dy: 7 };
/// hexagon の輪郭（倍率 1.4・中心からの相対）と数字の中心のずれ。
pub const DIE_L_HEXAGON: Die = Die { outline: Sprite { x: -105, y: -120, w: 210, h: 240, off: 268330 }, text_dy: 0 };
/// kite の輪郭（倍率 1.0・中心からの相対）と数字の中心のずれ。
pub const DIE_M_KITE: Die = Die { outline: Sprite { x: -76, y: -83, w: 152, h: 166, off: 274810 }, text_dy: -4 };
/// 数字 120px（Cormorant Garamond Bold）。
pub const FONT_TOTAL: [Glyph; 11] = [
    Glyph { ch: b'0', adv: 916, sprite: Sprite { x: 3, y: -10, w: 51, h: 50, off: 277964 } },
    Glyph { ch: b'1', adv: 636, sprite: Sprite { x: 5, y: -8, w: 30, h: 46, off: 278314 } },
    Glyph { ch: b'2', adv: 772, sprite: Sprite { x: 4, y: -11, w: 41, h: 49, off: 278498 } },
    Glyph { ch: b'3', adv: 756, sprite: Sprite { x: 3, y: -11, w: 40, h: 82, off: 278792 } },
    Glyph { ch: b'4', adv: 876, sprite: Sprite { x: 3, y: -9, w: 48, h: 70, off: 279202 } },
    Glyph { ch: b'5', adv: 784, sprite: Sprite { x: 7, y: -13, w: 36, h: 84, off: 279622 } },
    Glyph { ch: b'6', adv: 892, sprite: Sprite { x: 5, y: -41, w: 48, h: 81, off: 280042 } },
    Glyph { ch: b'7', adv: 824, sprite: Sprite { x: 2, y: -12, w: 45, h: 83, off: 280528 } },
    Glyph { ch: b'8', adv: 936, sprite: Sprite { x: 5, y: -31, w: 50, h: 71, off: 281026 } },
    Glyph { ch: b'9', adv: 892, sprite: Sprite { x: 3, y: -10, w: 48, h: 81, off: 281523 } },
    Glyph { ch: b'?', adv: 652, sprite: Sprite { x: 4, y: -37, w: 34, h: 76, off: 282009 } },
];
/// 数字 58px（Cormorant Garamond Bold）。
pub const FONT_DIE_M: [Glyph; 11] = [
    Glyph { ch: b'0', adv: 444, sprite: Sprite { x: 2, y: -5, w: 24, h: 24, off: 282389 } },
    Glyph { ch: b'1', adv: 312, sprite: Sprite { x: 3, y: -4, w: 14, h: 23, off: 282461 } },
    Glyph { ch: b'2', adv: 376, sprite: Sprite { x: 2, y: -5, w: 20, h: 23, off: 282507 } },
    Glyph { ch: b'3', adv: 364, sprite: Sprite { x: 2, y: -5, w: 19, h: 40, off: 282576 } },
    Glyph { ch: b'4', adv: 420, sprite: Sprite { x: 2, y: -4, w: 22, h: 33, off: 282696 } },
    Glyph { ch: b'5', adv: 380, sprite: Sprite { x: 4, y: -6, w: 17, h: 40, off: 282795 } },
    Glyph { ch: b'6', adv: 436, sprite: Sprite { x: 2, y: -20, w: 24, h: 39, off: 282915 } },
    Glyph { ch: b'7', adv: 400, sprite: Sprite { x: 1, y: -5, w: 22, h: 39, off: 283032 } },
    Glyph { ch: b'8', adv: 456, sprite: Sprite { x: 2, y: -15, w: 25, h: 34, off: 283149 } },
    Glyph { ch: b'9', adv: 432, sprite: Sprite { x: 2, y: -5, w: 23, h: 39, off: 283285 } },
    Glyph { ch: b'?', adv: 312, sprite: Sprite { x: 2, y: -18, w: 16, h: 37, off: 283402 } },
];
/// 数字 84px（Cormorant Garamond Bold）。
pub const FONT_DIE_L: [Glyph; 11] = [
    Glyph { ch: b'0', adv: 644, sprite: Sprite { x: 2, y: -7, w: 36, h: 35, off: 283476 } },
    Glyph { ch: b'1', adv: 448, sprite: Sprite { x: 4, y: -6, w: 21, h: 33, off: 283651 } },
    Glyph { ch: b'2', adv: 540, sprite: Sprite { x: 3, y: -7, w: 29, h: 34, off: 283750 } },
    Glyph { ch: b'3', adv: 528, sprite: Sprite { x: 2, y: -7, w: 28, h: 57, off: 283886 } },
    Glyph { ch: b'4', adv: 612, sprite: Sprite { x: 2, y: -7, w: 34, h: 49, off: 284114 } },
    Glyph { ch: b'5', adv: 548, sprite: Sprite { x: 5, y: -9, w: 25, h: 59, off: 284359 } },
    Glyph { ch: b'6', adv: 620, sprite: Sprite { x: 3, y: -29, w: 34, h: 57, off: 284595 } },
    Glyph { ch: b'7', adv: 576, sprite: Sprite { x: 1, y: -8, w: 32, h: 58, off: 284880 } },
    Glyph { ch: b'8', adv: 656, sprite: Sprite { x: 3, y: -22, w: 36, h: 50, off: 285112 } },
    Glyph { ch: b'9', adv: 624, sprite: Sprite { x: 2, y: -7, w: 34, h: 57, off: 285362 } },
    Glyph { ch: b'?', adv: 456, sprite: Sprite { x: 3, y: -26, w: 24, h: 54, off: 285647 } },
];
/// コインの面（heads）。
pub const COIN_HEADS: Sprite = Sprite { x: 90, y: 221, w: 300, h: 300, off: 285809 };
/// コインの面（tails）。
pub const COIN_TAILS: Sprite = Sprite { x: 90, y: 221, w: 300, h: 300, off: 297209 };
/// コインの面（wait）。
pub const COIN_WAIT: Sprite = Sprite { x: 90, y: 221, w: 300, h: 300, off: 308609 };
/// HEADS。
pub const COIN_WORD_HEADS: Sprite = Sprite { x: 126, y: 561, w: 227, h: 33, off: 320009 };
/// TAILS。
pub const COIN_WORD_TAILS: Sprite = Sprite { x: 139, y: 560, w: 201, h: 34, off: 320966 };
/// 表。
pub const COIN_KANJI_HEADS: Sprite = Sprite { x: 230, y: 617, w: 21, h: 22, off: 321850 };
/// 裏。
pub const COIN_KANJI_TAILS: Sprite = Sprite { x: 230, y: 617, w: 20, h: 22, off: 321916 };
/// YES。
pub const YESNO_YES: Sprite = Sprite { x: 102, y: 322, w: 269, h: 97, off: 321982 };
/// NO。
pub const YESNO_NO: Sprite = Sprite { x: 124, y: 322, w: 227, h: 98, off: 325280 };
/// ?。
pub const YESNO_WAIT: Sprite = Sprite { x: 219, y: 324, w: 43, h: 95, off: 328122 };
/// 是。
pub const YESNO_KANJI_YES: Sprite = Sprite { x: 220, y: 518, w: 39, h: 41, off: 328692 };
/// 非。
pub const YESNO_KANJI_NO: Sprite = Sprite { x: 220, y: 518, w: 40, h: 41, off: 328897 };
