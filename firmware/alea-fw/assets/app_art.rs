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
/// 棒倒しの台紙（方式の札はどちらも未選択の姿）。
pub const STICK_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 329102 };
/// 方式の札の矩形 (x, y, w, h)。[左右, 八方位]。
pub const STICK_CHIPS: [(i32, i32, i32, i32); 2] = [(127, 134, 110, 40), (243, 134, 110, 40)];
/// 方位盤。
pub const STICK_COMPASS: Sprite = Sprite { x: 59, y: 202, w: 362, h: 362, off: 373106 };
/// 8 方位に倒れた棒（0=北から時計回り）。
pub const STICK_DIR: [Sprite; 8] = [
    Sprite { x: 229, y: 249, w: 22, h: 142, off: 389758 },
    Sprite { x: 231, y: 285, w: 107, h: 106, off: 390184 },
    Sprite { x: 231, y: 371, w: 142, h: 22, off: 391668 },
    Sprite { x: 231, y: 374, w: 107, h: 105, off: 392064 },
    Sprite { x: 229, y: 374, w: 22, h: 141, off: 393534 },
    Sprite { x: 143, y: 374, w: 106, h: 105, off: 393957 },
    Sprite { x: 107, y: 371, w: 142, h: 22, off: 395427 },
    Sprite { x: 143, y: 285, w: 106, h: 106, off: 395823 },
];
/// 方位盤の中心の支点（待機中）。
pub const STICK_PIVOT: Sprite = Sprite { x: 231, y: 374, w: 18, h: 17, off: 397307 };
/// 方位名（和）。
pub const STICK_DIR_JA: [Sprite; 8] = [
    Sprite { x: 217, y: 586, w: 47, h: 47, off: 397358 },
    Sprite { x: 185, y: 584, w: 109, h: 49, off: 397640 },
    Sprite { x: 217, y: 584, w: 46, h: 49, off: 398326 },
    Sprite { x: 185, y: 584, w: 109, h: 49, off: 398620 },
    Sprite { x: 217, y: 585, w: 46, h: 48, off: 399306 },
    Sprite { x: 185, y: 585, w: 109, h: 48, off: 399594 },
    Sprite { x: 217, y: 585, w: 46, h: 48, off: 400266 },
    Sprite { x: 185, y: 585, w: 109, h: 48, off: 400554 },
];
/// 方位名（英）。
pub const STICK_DIR_EN: [Sprite; 8] = [
    Sprite { x: 188, y: 649, w: 103, h: 14, off: 401226 },
    Sprite { x: 142, y: 649, w: 197, h: 14, off: 401408 },
    Sprite { x: 205, y: 649, w: 71, h: 14, off: 401758 },
    Sprite { x: 144, y: 649, w: 193, h: 14, off: 401884 },
    Sprite { x: 191, y: 649, w: 98, h: 14, off: 402234 },
    Sprite { x: 142, y: 649, w: 197, h: 14, off: 402416 },
    Sprite { x: 202, y: 649, w: 76, h: 14, off: 402766 },
    Sprite { x: 139, y: 649, w: 202, h: 14, off: 402906 },
];
/// 8 方位の待機中の「?」。
pub const STICK_EIGHT_WAIT: Sprite = Sprite { x: 232, y: 588, w: 17, h: 38, off: 403270 };
/// 地面・立っていた位置の破線・支点。
pub const STICK_GROUND: Sprite = Sprite { x: 60, y: 300, w: 360, h: 124, off: 403384 };
/// 左右に倒れた棒。[左, 右]。
pub const STICK_SIDE: [Sprite; 2] = [
    Sprite { x: 87, y: 329, w: 158, h: 83, off: 408964 },
    Sprite { x: 235, y: 329, w: 158, h: 83, off: 410624 },
];
/// 立っている棒（左右の待機中）。
pub const STICK_STAND: Sprite = Sprite { x: 229, y: 269, w: 22, h: 142, off: 412284 };
/// 左・右（和）。
pub const STICK_SIDE_JA: [Sprite; 2] = [
    Sprite { x: 196, y: 500, w: 87, h: 88, off: 412710 },
    Sprite { x: 196, y: 500, w: 87, h: 90, off: 413678 },
];
/// LEFT・RIGHT。
pub const STICK_SIDE_EN: [Sprite; 2] = [
    Sprite { x: 207, y: 613, w: 67, h: 14, off: 414668 },
    Sprite { x: 194, y: 613, w: 93, h: 14, off: 414794 },
];
/// 左右の待機中の「?」。
pub const STICK_SIDE_WAIT: Sprite = Sprite { x: 227, y: 510, w: 27, h: 61, off: 414962 };
/// あみだくじの台紙。
pub const AMIDA_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 415206 };
/// 本数の案内（2〜6 本）。
pub const AMIDA_NOTE: [Sprite; 5] = [
    Sprite { x: 72, y: 143, w: 337, h: 21, off: 459210 },
    Sprite { x: 72, y: 143, w: 337, h: 21, off: 460113 },
    Sprite { x: 73, y: 143, w: 336, h: 21, off: 461016 },
    Sprite { x: 72, y: 143, w: 337, h: 21, off: 461898 },
    Sprite { x: 72, y: 143, w: 337, h: 21, off: 462801 },
];
/// 上の札のローマ数字（中心からの相対）。
pub const AMIDA_ROMAN: [Sprite; 6] = [
    Sprite { x: -2, y: -7, w: 4, h: 15, off: 463704 },
    Sprite { x: -6, y: -7, w: 12, h: 15, off: 463719 },
    Sprite { x: -10, y: -7, w: 20, h: 15, off: 463749 },
    Sprite { x: -10, y: -7, w: 21, h: 15, off: 463794 },
    Sprite { x: -7, y: -7, w: 14, h: 15, off: 463839 },
    Sprite { x: -11, y: -7, w: 21, h: 15, off: 463869 },
];
/// 下の番号（中心からの相対）。
pub const AMIDA_GOAL: [Sprite; 6] = [
    Sprite { x: -3, y: -2, w: 6, h: 13, off: 463914 },
    Sprite { x: -5, y: -3, w: 11, h: 14, off: 463927 },
    Sprite { x: -5, y: -3, w: 11, h: 23, off: 463955 },
    Sprite { x: -6, y: -3, w: 13, h: 20, off: 464001 },
    Sprite { x: -5, y: -3, w: 10, h: 23, off: 464041 },
    Sprite { x: -6, y: -11, w: 13, h: 22, off: 464087 },
];
/// 上の札の上端。
pub const AMIDA_BOX_TOP: i32 = 176;
/// 上の札・下の番号の枠の幅。
pub const AMIDA_BOX_W: i32 = 52;
/// 上の札の高さ。
pub const AMIDA_BOX_H: i32 = 44;
/// 縦線の上端。
pub const AMIDA_LADDER_TOP: i32 = 228;
/// 縦線の下端。
pub const AMIDA_LADDER_BOTTOM: i32 = 642;
/// 下の番号の枠の上端。
pub const AMIDA_GOAL_TOP: i32 = 652;
/// 下の番号の枠の高さ。
pub const AMIDA_GOAL_H: i32 = 48;
/// 両端の縦線の間隔の上限。
pub const AMIDA_SPAN: i32 = 340;
/// 縦線の間隔の上限（本数が少ないとき）。
pub const AMIDA_MAX_SPACING: i32 = 110;
/// おみくじの台紙（紙片と 2 本の罫を含む）。
pub const OMIKUJI_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 464131 };
/// 運勢（縦書き）。
pub const OMIKUJI_FORTUNE: [Sprite; 6] = [
    Sprite { x: 210, y: 302, w: 61, h: 132, off: 508135 },
    Sprite { x: 210, y: 336, w: 60, h: 64, off: 509191 },
    Sprite { x: 210, y: 302, w: 60, h: 132, off: 509703 },
    Sprite { x: 209, y: 303, w: 61, h: 131, off: 510759 },
    Sprite { x: 209, y: 302, w: 62, h: 132, off: 511807 },
    Sprite { x: 212, y: 338, w: 57, h: 61, off: 512863 },
];
/// 待機中の「?」。
pub const OMIKUJI_WAIT: Sprite = Sprite { x: 230, y: 342, w: 21, h: 46, off: 513351 };
/// 番号（第一番〜・縦書き）。運勢の順に一言を通し番号にしたもの。
pub const OMIKUJI_NUMBER: [Sprite; 30] = [
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 513489 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 513696 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 513903 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 514110 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 514317 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 514524 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 514731 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 514938 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 515145 },
    Sprite { x: 231, y: 199, w: 18, h: 69, off: 515352 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 515559 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 515904 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 516249 },
    Sprite { x: 221, y: 199, w: 37, h: 69, off: 516594 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 516939 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 517284 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 517629 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 517974 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 518319 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 518664 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 519009 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 519561 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 520113 },
    Sprite { x: 211, y: 199, w: 57, h: 69, off: 520665 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 521217 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 521769 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 522321 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 522873 },
    Sprite { x: 211, y: 199, w: 58, h: 69, off: 523425 },
    Sprite { x: 221, y: 199, w: 38, h: 69, off: 523977 },
];
/// 一言（縦書き・通し番号順）。
pub const OMIKUJI_MESSAGE: [Sprite; 30] = [
    Sprite { x: 192, y: 456, w: 96, h: 197, off: 524322 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 526686 },
    Sprite { x: 193, y: 456, w: 95, h: 195, off: 529026 },
    Sprite { x: 192, y: 456, w: 96, h: 197, off: 531366 },
    Sprite { x: 192, y: 456, w: 96, h: 197, off: 533730 },
    Sprite { x: 193, y: 456, w: 96, h: 195, off: 536094 },
    Sprite { x: 192, y: 456, w: 96, h: 173, off: 538434 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 540510 },
    Sprite { x: 193, y: 456, w: 96, h: 195, off: 542850 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 545190 },
    Sprite { x: 192, y: 456, w: 97, h: 174, off: 547530 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 549792 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 552132 },
    Sprite { x: 192, y: 456, w: 96, h: 153, off: 554472 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 556308 },
    Sprite { x: 192, y: 456, w: 96, h: 195, off: 558648 },
    Sprite { x: 193, y: 456, w: 95, h: 173, off: 560988 },
    Sprite { x: 192, y: 456, w: 96, h: 217, off: 563064 },
    Sprite { x: 193, y: 456, w: 95, h: 195, off: 565668 },
    Sprite { x: 192, y: 456, w: 97, h: 217, off: 568008 },
    Sprite { x: 192, y: 456, w: 96, h: 196, off: 570829 },
    Sprite { x: 192, y: 456, w: 96, h: 196, off: 573181 },
    Sprite { x: 192, y: 456, w: 96, h: 173, off: 575533 },
    Sprite { x: 192, y: 456, w: 97, h: 195, off: 577609 },
    Sprite { x: 192, y: 456, w: 96, h: 196, off: 580144 },
    Sprite { x: 192, y: 456, w: 96, h: 173, off: 582496 },
    Sprite { x: 193, y: 456, w: 95, h: 195, off: 584572 },
    Sprite { x: 192, y: 456, w: 96, h: 173, off: 586912 },
    Sprite { x: 192, y: 456, w: 96, h: 130, off: 588988 },
    Sprite { x: 192, y: 456, w: 96, h: 174, off: 590548 },
];
/// 運勢の重み。
pub const OMIKUJI_WEIGHTS: [u32; 6] = [15, 20, 20, 18, 15, 12];
/// 運勢ごとの一言の数。
pub const OMIKUJI_COUNTS: [u32; 6] = [5, 5, 5, 5, 5, 5];
/// 運勢の名前（ログ用）。
pub const OMIKUJI_LABELS: [&str; 6] = ["大吉", "吉", "中吉", "小吉", "末吉", "凶"];
/// 易の途中の台紙（爻の名前・凡例を含む）。
pub const IC_CAST_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 592636 };
/// 案内と進み具合（〇／六〜六／六・2 行）。
pub const IC_NOTE: [Sprite; 7] = [
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 636640 },
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 638208 },
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 639776 },
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 641344 },
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 642912 },
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 644480 },
    Sprite { x: 128, y: 135, w: 220, h: 56, off: 646048 },
];
/// 硬貨 3 枚の表裏（添字の bit2 が 1 枚目・1=表。中心からの相対）。
pub const IC_COINS: [Sprite; 8] = [
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 647616 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 647721 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 647826 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 647931 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 648036 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 648141 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 648246 },
    Sprite { x: -27, y: -7, w: 54, h: 15, off: 648351 },
];
/// 老陽の印〇（中心からの相対）。
pub const IC_MARK_O: Sprite = Sprite { x: -8, y: -8, w: 16, h: 16, off: 648456 };
/// 老陰の印×（中心からの相対）。
pub const IC_MARK_X: Sprite = Sprite { x: -7, y: -8, w: 15, h: 16, off: 648488 };
/// 初爻の欄の中心の高さ。
pub const IC_ROW_BOTTOM_Y: i32 = 560;
/// 爻の欄の行送り（上へ）。
pub const IC_ROW_PITCH: i32 = 52;
/// 途中の爻の棒の左端。
pub const IC_BAR_X: i32 = 96;
/// 途中の爻の棒の幅。
pub const IC_BAR_W: i32 = 220;
/// 途中の爻の棒の太さ。
pub const IC_BAR_H: i32 = 18;
/// 変爻の印の中心の x。
pub const IC_MARK_CX: i32 = 336;
/// 硬貨の表裏の中心の x。
pub const IC_COIN_CX: i32 = 400;
/// 易の結果の台紙。
pub const IC_RESULT_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 648520 };
/// 「本卦」（列の中心 x からの相対）。
pub const IC_LABEL_PRIMARY: Sprite = Sprite { x: -20, y: 153, w: 40, h: 17, off: 692524 };
/// 「之卦」（列の中心 x からの相対）。
pub const IC_LABEL_CHANGED: Sprite = Sprite { x: -20, y: 153, w: 40, h: 17, off: 692609 };
/// 本卦から之卦への矢印。
pub const IC_ARROW: Sprite = Sprite { x: 231, y: 349, w: 17, h: 10, off: 692694 };
/// 卦名（文王の順・添字 0 が第一卦。列の中心 x からの相対）。
pub const IC_NAME: [Sprite; 64] = [
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 692724 },
    Sprite { x: -47, y: 340, w: 94, h: 28, off: 693060 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 693396 },
    Sprite { x: -45, y: 340, w: 91, h: 28, off: 693732 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 694068 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 694404 },
    Sprite { x: -47, y: 340, w: 93, h: 28, off: 694740 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 695076 },
    Sprite { x: -63, y: 340, w: 125, h: 28, off: 695412 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 695860 },
    Sprite { x: -47, y: 340, w: 94, h: 28, off: 696196 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 696532 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 696868 },
    Sprite { x: -63, y: 340, w: 125, h: 28, off: 697316 },
    Sprite { x: -47, y: 340, w: 94, h: 28, off: 697764 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 698100 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 698436 },
    Sprite { x: -45, y: 340, w: 92, h: 28, off: 698772 },
    Sprite { x: -47, y: 340, w: 93, h: 28, off: 699108 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 699444 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 699780 },
    Sprite { x: -45, y: 340, w: 91, h: 28, off: 700228 },
    Sprite { x: -45, y: 340, w: 91, h: 28, off: 700564 },
    Sprite { x: -47, y: 340, w: 94, h: 28, off: 700900 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 701236 },
    Sprite { x: -61, y: 340, w: 123, h: 28, off: 701684 },
    Sprite { x: -45, y: 340, w: 92, h: 28, off: 702132 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 702468 },
    Sprite { x: -47, y: 340, w: 93, h: 28, off: 702916 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 703252 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 703588 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 703924 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 704260 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 704596 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 705044 },
    Sprite { x: -63, y: 340, w: 125, h: 28, off: 705380 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 705828 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 706276 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 706612 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 706948 },
    Sprite { x: -45, y: 340, w: 91, h: 28, off: 707284 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 707620 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 707956 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 708292 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 708628 },
    Sprite { x: -47, y: 340, w: 93, h: 28, off: 708964 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 709300 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 709636 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 709972 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 710308 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 710644 },
    Sprite { x: -45, y: 340, w: 90, h: 28, off: 710980 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 711316 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 711652 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 712100 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 712436 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 712772 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 713108 },
    Sprite { x: -46, y: 340, w: 93, h: 28, off: 713444 },
    Sprite { x: -46, y: 340, w: 92, h: 28, off: 713780 },
    Sprite { x: -63, y: 339, w: 126, h: 29, off: 714116 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 714580 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 715028 },
    Sprite { x: -63, y: 340, w: 126, h: 28, off: 715476 },
];
/// 番号と読み（列の中心 x からの相対）。
pub const IC_NUMBER_READING: [Sprite; 64] = [
    Sprite { x: -39, y: 379, w: 79, h: 39, off: 715924 },
    Sprite { x: -30, y: 379, w: 60, h: 39, off: 716314 },
    Sprite { x: -57, y: 379, w: 114, h: 39, off: 716626 },
    Sprite { x: -47, y: 379, w: 93, h: 39, off: 717211 },
    Sprite { x: -48, y: 379, w: 95, h: 39, off: 717679 },
    Sprite { x: -57, y: 379, w: 112, h: 39, off: 718147 },
    Sprite { x: -31, y: 379, w: 62, h: 39, off: 718693 },
    Sprite { x: -32, y: 379, w: 64, h: 39, off: 719005 },
    Sprite { x: -74, y: 379, w: 143, h: 39, off: 719317 },
    Sprite { x: -40, y: 379, w: 77, h: 39, off: 720019 },
    Sprite { x: -39, y: 379, w: 79, h: 39, off: 720409 },
    Sprite { x: -33, y: 379, w: 67, h: 39, off: 720799 },
    Sprite { x: -57, y: 379, w: 114, h: 39, off: 721150 },
    Sprite { x: -57, y: 379, w: 112, h: 39, off: 721735 },
    Sprite { x: -39, y: 379, w: 79, h: 39, off: 722281 },
    Sprite { x: -33, y: 379, w: 67, h: 39, off: 722671 },
    Sprite { x: -49, y: 379, w: 97, h: 39, off: 723022 },
    Sprite { x: -39, y: 379, w: 77, h: 39, off: 723529 },
    Sprite { x: -39, y: 379, w: 79, h: 39, off: 723919 },
    Sprite { x: -41, y: 379, w: 81, h: 39, off: 724309 },
    Sprite { x: -57, y: 379, w: 112, h: 39, off: 724738 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 725284 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 725713 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 726142 },
    Sprite { x: -57, y: 379, w: 112, h: 39, off: 726571 },
    Sprite { x: -64, y: 379, w: 125, h: 39, off: 727117 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 727741 },
    Sprite { x: -57, y: 379, w: 115, h: 39, off: 728170 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 728755 },
    Sprite { x: -33, y: 379, w: 67, h: 39, off: 729184 },
    Sprite { x: -49, y: 379, w: 98, h: 39, off: 729535 },
    Sprite { x: -46, y: 379, w: 92, h: 39, off: 730042 },
    Sprite { x: -49, y: 379, w: 98, h: 39, off: 730510 },
    Sprite { x: -63, y: 379, w: 126, h: 39, off: 731017 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 731641 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 732070 },
    Sprite { x: -49, y: 379, w: 98, h: 39, off: 732499 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 733006 },
    Sprite { x: -48, y: 379, w: 97, h: 39, off: 733435 },
    Sprite { x: -46, y: 379, w: 94, h: 39, off: 733942 },
    Sprite { x: -47, y: 379, w: 96, h: 39, off: 734410 },
    Sprite { x: -49, y: 379, w: 96, h: 39, off: 734878 },
    Sprite { x: -49, y: 379, w: 97, h: 39, off: 735346 },
    Sprite { x: -49, y: 379, w: 95, h: 39, off: 735853 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 736321 },
    Sprite { x: -47, y: 379, w: 93, h: 39, off: 736750 },
    Sprite { x: -49, y: 379, w: 98, h: 39, off: 737218 },
    Sprite { x: -48, y: 379, w: 96, h: 39, off: 737725 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 738193 },
    Sprite { x: -40, y: 379, w: 80, h: 39, off: 738622 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 739012 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 739441 },
    Sprite { x: -49, y: 379, w: 98, h: 39, off: 739870 },
    Sprite { x: -55, y: 379, w: 112, h: 39, off: 740377 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 740923 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 741352 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 741781 },
    Sprite { x: -42, y: 379, w: 85, h: 39, off: 742210 },
    Sprite { x: -49, y: 379, w: 98, h: 39, off: 742639 },
    Sprite { x: -48, y: 379, w: 96, h: 39, off: 743146 },
    Sprite { x: -66, y: 379, w: 131, h: 39, off: 743614 },
    Sprite { x: -63, y: 379, w: 129, h: 39, off: 744277 },
    Sprite { x: -48, y: 379, w: 96, h: 39, off: 744940 },
    Sprite { x: -48, y: 379, w: 96, h: 39, off: 745408 },
];
/// 解釈（2 行・欄の上端からの相対）。
pub const IC_SUMMARY: [Sprite; 64] = [
    Sprite { x: 101, y: 2, w: 235, h: 54, off: 745876 },
    Sprite { x: 102, y: 2, w: 261, h: 54, off: 747496 },
    Sprite { x: 101, y: 2, w: 213, h: 54, off: 749278 },
    Sprite { x: 102, y: 2, w: 235, h: 54, off: 750736 },
    Sprite { x: 101, y: 2, w: 235, h: 54, off: 752356 },
    Sprite { x: 101, y: 2, w: 240, h: 54, off: 753976 },
    Sprite { x: 101, y: 2, w: 258, h: 54, off: 755596 },
    Sprite { x: 101, y: 2, w: 238, h: 54, off: 757378 },
    Sprite { x: 102, y: 2, w: 217, h: 53, off: 758998 },
    Sprite { x: 101, y: 2, w: 258, h: 54, off: 760482 },
    Sprite { x: 101, y: 2, w: 236, h: 54, off: 762264 },
    Sprite { x: 101, y: 2, w: 235, h: 54, off: 763884 },
    Sprite { x: 101, y: 2, w: 258, h: 54, off: 765504 },
    Sprite { x: 102, y: 2, w: 234, h: 54, off: 767286 },
    Sprite { x: 101, y: 2, w: 214, h: 54, off: 768906 },
    Sprite { x: 101, y: 2, w: 213, h: 54, off: 770364 },
    Sprite { x: 101, y: 2, w: 195, h: 54, off: 771822 },
    Sprite { x: 101, y: 2, w: 191, h: 54, off: 773172 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 774468 },
    Sprite { x: 104, y: 2, w: 232, h: 54, off: 775818 },
    Sprite { x: 102, y: 2, w: 234, h: 54, off: 777384 },
    Sprite { x: 101, y: 2, w: 218, h: 54, off: 779004 },
    Sprite { x: 101, y: 2, w: 218, h: 54, off: 780516 },
    Sprite { x: 101, y: 2, w: 240, h: 54, off: 782028 },
    Sprite { x: 102, y: 2, w: 239, h: 54, off: 783648 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 785268 },
    Sprite { x: 102, y: 2, w: 212, h: 54, off: 786618 },
    Sprite { x: 101, y: 2, w: 213, h: 54, off: 788076 },
    Sprite { x: 101, y: 2, w: 214, h: 54, off: 789534 },
    Sprite { x: 101, y: 2, w: 235, h: 54, off: 790992 },
    Sprite { x: 101, y: 2, w: 214, h: 54, off: 792612 },
    Sprite { x: 101, y: 2, w: 218, h: 54, off: 794070 },
    Sprite { x: 101, y: 2, w: 195, h: 54, off: 795582 },
    Sprite { x: 101, y: 2, w: 235, h: 54, off: 796932 },
    Sprite { x: 101, y: 2, w: 218, h: 54, off: 798552 },
    Sprite { x: 102, y: 2, w: 234, h: 54, off: 800064 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 801684 },
    Sprite { x: 102, y: 2, w: 234, h: 54, off: 803034 },
    Sprite { x: 101, y: 2, w: 235, h: 54, off: 804654 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 806274 },
    Sprite { x: 101, y: 2, w: 214, h: 54, off: 807624 },
    Sprite { x: 101, y: 2, w: 195, h: 54, off: 809082 },
    Sprite { x: 101, y: 2, w: 237, h: 54, off: 810432 },
    Sprite { x: 101, y: 2, w: 191, h: 54, off: 812052 },
    Sprite { x: 102, y: 2, w: 168, h: 54, off: 813348 },
    Sprite { x: 101, y: 2, w: 218, h: 54, off: 814482 },
    Sprite { x: 101, y: 2, w: 213, h: 54, off: 815994 },
    Sprite { x: 102, y: 2, w: 214, h: 54, off: 817452 },
    Sprite { x: 101, y: 2, w: 213, h: 54, off: 818910 },
    Sprite { x: 101, y: 2, w: 191, h: 54, off: 820368 },
    Sprite { x: 101, y: 2, w: 214, h: 54, off: 821664 },
    Sprite { x: 101, y: 2, w: 191, h: 53, off: 823122 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 824394 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 825744 },
    Sprite { x: 101, y: 2, w: 171, h: 54, off: 827094 },
    Sprite { x: 101, y: 2, w: 174, h: 54, off: 828282 },
    Sprite { x: 101, y: 2, w: 218, h: 54, off: 829470 },
    Sprite { x: 101, y: 2, w: 196, h: 54, off: 830982 },
    Sprite { x: 101, y: 2, w: 196, h: 53, off: 832332 },
    Sprite { x: 102, y: 2, w: 173, h: 54, off: 833657 },
    Sprite { x: 101, y: 2, w: 195, h: 54, off: 834845 },
    Sprite { x: 102, y: 2, w: 239, h: 54, off: 836195 },
    Sprite { x: 101, y: 2, w: 260, h: 54, off: 837815 },
    Sprite { x: 101, y: 2, w: 213, h: 54, off: 839597 },
];
/// 解釈の欄の見出し「本卦」（欄の上端からの相対）。
pub const IC_SUM_PRIMARY: Sprite = Sprite { x: 40, y: 4, w: 44, h: 24, off: 841055 };
/// 解釈の欄の見出し「之卦」（欄の上端からの相対）。
pub const IC_SUM_CHANGED: Sprite = Sprite { x: 40, y: 4, w: 44, h: 24, off: 841199 };
/// 変爻の行に使う字（`IC_CHANGE_GLYPHS` と同じ順）。
pub const IC_CHANGE_CHARS: [char; 11] = ['変', '爻', '初', '二', '三', '四', '五', '上', '・', 'な', 'し'];
/// 変爻の行の字（1 字ずつ・中心からの相対）。
pub const IC_CHANGE_GLYPHS: [Sprite; 11] = [
    Sprite { x: -8, y: -7, w: 16, h: 17, off: 841343 },
    Sprite { x: -7, y: -7, w: 15, h: 17, off: 841377 },
    Sprite { x: -8, y: -7, w: 16, h: 17, off: 841411 },
    Sprite { x: -8, y: -5, w: 16, h: 12, off: 841445 },
    Sprite { x: -8, y: -7, w: 16, h: 15, off: 841469 },
    Sprite { x: -7, y: -6, w: 15, h: 15, off: 841499 },
    Sprite { x: -8, y: -7, w: 16, h: 16, off: 841529 },
    Sprite { x: -8, y: -7, w: 16, h: 16, off: 841561 },
    Sprite { x: -2, y: -1, w: 4, h: 4, off: 841593 },
    Sprite { x: -7, y: -6, w: 13, h: 15, off: 841597 },
    Sprite { x: -5, y: -6, w: 11, h: 15, off: 841627 },
];
/// 本卦・之卦の列の中心 x。
pub const IC_COL_CX: [i32; 2] = [130, 350];
/// 卦画の上端。
pub const IC_FIG_TOP: i32 = 182;
/// 卦画の幅。
pub const IC_FIG_W: i32 = 120;
/// 卦画の爻の太さ。
pub const IC_FIG_BAR: i32 = 14;
/// 卦画の爻の間隔。
pub const IC_FIG_GAP: i32 = 11;
/// 変爻の行の中心の高さ。
pub const IC_CHANGES_Y: i32 = 452;
/// 変爻の行の字送り。
pub const IC_CHANGES_ADV: i32 = 22;
/// 解釈の欄の上端 [本卦, 之卦]。
pub const IC_SUM_TOP: [i32; 2] = [500, 596];
/// ルーンの台紙（逆位置の札は未選択の姿・石の輪郭を含む）。
pub const RUNE_FRAME: Sprite = Sprite { x: 14, y: 14, w: 452, h: 772, off: 841657 };
/// 逆位置の札の矩形 (x, y, w, h)。[正位置のみ, 逆位置あり]。
pub const RUNE_CHIPS: [(i32, i32, i32, i32); 2] = [(87, 134, 150, 40), (243, 134, 150, 40)];
/// 点対称の字形か（逆位置にならない）。
pub const RUNE_SYMMETRIC: [bool; 24] = [false, false, false, false, false, false, true, false, true, true, true, true, true, false, false, true, false, false, false, false, false, true, true, false];
/// 字形（正位置）。
pub const RUNE_GLYPH: [Sprite; 24] = [
    Sprite { x: 209, y: 241, w: 72, h: 171, off: 885661 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 887200 },
    Sprite { x: 214, y: 241, w: 62, h: 171, off: 888739 },
    Sprite { x: 209, y: 241, w: 67, h: 171, off: 890107 },
    Sprite { x: 209, y: 241, w: 67, h: 171, off: 891646 },
    Sprite { x: 209, y: 271, w: 62, h: 111, off: 893185 },
    Sprite { x: 194, y: 261, w: 92, h: 131, off: 894073 },
    Sprite { x: 209, y: 241, w: 67, h: 171, off: 895645 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 897184 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 898723 },
    Sprite { x: 234, y: 241, w: 12, h: 171, off: 900262 },
    Sprite { x: 194, y: 256, w: 92, h: 141, off: 900604 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 902296 },
    Sprite { x: 204, y: 241, w: 77, h: 171, off: 903835 },
    Sprite { x: 194, y: 241, w: 92, h: 171, off: 905545 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 907597 },
    Sprite { x: 194, y: 241, w: 92, h: 171, off: 909136 },
    Sprite { x: 209, y: 241, w: 62, h: 171, off: 911188 },
    Sprite { x: 199, y: 241, w: 82, h: 171, off: 912556 },
    Sprite { x: 199, y: 241, w: 82, h: 171, off: 914437 },
    Sprite { x: 214, y: 241, w: 62, h: 171, off: 916318 },
    Sprite { x: 199, y: 271, w: 82, h: 111, off: 917686 },
    Sprite { x: 194, y: 261, w: 92, h: 131, off: 918907 },
    Sprite { x: 194, y: 246, w: 92, h: 161, off: 920479 },
];
/// 字形（逆位置・180° 回転）。
pub const RUNE_GLYPH_REV: [Sprite; 24] = [
    Sprite { x: 199, y: 241, w: 72, h: 171, off: 922411 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 923950 },
    Sprite { x: 204, y: 241, w: 62, h: 171, off: 925489 },
    Sprite { x: 204, y: 241, w: 67, h: 171, off: 926857 },
    Sprite { x: 204, y: 241, w: 67, h: 171, off: 928396 },
    Sprite { x: 209, y: 271, w: 62, h: 111, off: 929935 },
    Sprite { x: 194, y: 261, w: 92, h: 131, off: 930823 },
    Sprite { x: 204, y: 241, w: 67, h: 171, off: 932395 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 933934 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 935473 },
    Sprite { x: 234, y: 241, w: 12, h: 171, off: 937012 },
    Sprite { x: 194, y: 256, w: 92, h: 141, off: 937354 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 939046 },
    Sprite { x: 199, y: 241, w: 77, h: 171, off: 940585 },
    Sprite { x: 194, y: 241, w: 92, h: 171, off: 942295 },
    Sprite { x: 204, y: 241, w: 72, h: 171, off: 944347 },
    Sprite { x: 194, y: 241, w: 92, h: 171, off: 945886 },
    Sprite { x: 209, y: 241, w: 62, h: 171, off: 947938 },
    Sprite { x: 199, y: 241, w: 82, h: 171, off: 949306 },
    Sprite { x: 199, y: 241, w: 82, h: 171, off: 951187 },
    Sprite { x: 204, y: 241, w: 62, h: 171, off: 953068 },
    Sprite { x: 199, y: 271, w: 82, h: 111, off: 954436 },
    Sprite { x: 194, y: 261, w: 92, h: 131, off: 955657 },
    Sprite { x: 194, y: 246, w: 92, h: 161, off: 957229 },
];
/// 待機中の「?」。
pub const RUNE_WAIT: Sprite = Sprite { x: 229, y: 301, w: 23, h: 51, off: 959161 };
/// 名前（英字）。
pub const RUNE_NAME: [Sprite; 24] = [
    Sprite { x: 184, y: 481, w: 113, h: 22, off: 959314 },
    Sprite { x: 182, y: 481, w: 118, h: 22, off: 959644 },
    Sprite { x: 121, y: 481, w: 238, h: 22, off: 959974 },
    Sprite { x: 165, y: 481, w: 149, h: 22, off: 960634 },
    Sprite { x: 149, y: 481, w: 183, h: 22, off: 961052 },
    Sprite { x: 166, y: 481, w: 148, h: 22, off: 961558 },
    Sprite { x: 181, y: 481, w: 118, h: 22, off: 961976 },
    Sprite { x: 163, y: 481, w: 154, h: 30, off: 962306 },
    Sprite { x: 130, y: 481, w: 221, h: 22, off: 962906 },
    Sprite { x: 152, y: 481, w: 177, h: 22, off: 963522 },
    Sprite { x: 206, y: 481, w: 70, h: 22, off: 964028 },
    Sprite { x: 184, y: 481, w: 109, h: 30, off: 964226 },
    Sprite { x: 151, y: 481, w: 179, h: 22, off: 964646 },
    Sprite { x: 132, y: 481, w: 216, h: 22, off: 965152 },
    Sprite { x: 170, y: 481, w: 139, h: 22, off: 965746 },
    Sprite { x: 151, y: 481, w: 178, h: 22, off: 966142 },
    Sprite { x: 166, y: 481, w: 148, h: 22, off: 966648 },
    Sprite { x: 132, y: 481, w: 217, h: 22, off: 967066 },
    Sprite { x: 162, y: 481, w: 157, h: 22, off: 967682 },
    Sprite { x: 142, y: 481, w: 196, h: 22, off: 968122 },
    Sprite { x: 166, y: 481, w: 149, h: 22, off: 968672 },
    Sprite { x: 148, y: 481, w: 185, h: 22, off: 969090 },
    Sprite { x: 162, y: 481, w: 156, h: 22, off: 969618 },
    Sprite { x: 146, y: 481, w: 189, h: 22, off: 970058 },
];
/// 名前（カタカナ・中心からの相対）。
pub const RUNE_KANA: [Sprite; 24] = [
    Sprite { x: -30, y: -5, w: 61, h: 13, off: 970586 },
    Sprite { x: -30, y: -8, w: 63, h: 17, off: 970690 },
    Sprite { x: -43, y: -8, w: 88, h: 18, off: 970826 },
    Sprite { x: -43, y: -8, w: 88, h: 17, off: 971024 },
    Sprite { x: -30, y: -7, w: 62, h: 16, off: 971211 },
    Sprite { x: -31, y: -8, w: 64, h: 17, off: 971339 },
    Sprite { x: -31, y: -8, w: 63, h: 17, off: 971475 },
    Sprite { x: -54, y: -8, w: 110, h: 17, off: 971611 },
    Sprite { x: -45, y: -8, w: 90, h: 17, off: 971849 },
    Sprite { x: -44, y: -8, w: 89, h: 17, off: 972053 },
    Sprite { x: -19, y: -7, w: 40, h: 17, off: 972257 },
    Sprite { x: -31, y: -8, w: 61, h: 17, off: 972342 },
    Sprite { x: -44, y: -8, w: 89, h: 17, off: 972478 },
    Sprite { x: -45, y: -5, w: 87, h: 13, off: 972682 },
    Sprite { x: -43, y: -8, w: 88, h: 18, off: 972825 },
    Sprite { x: -42, y: -7, w: 84, h: 17, off: 973023 },
    Sprite { x: -42, y: -8, w: 87, h: 18, off: 973210 },
    Sprite { x: -45, y: -7, w: 89, h: 16, off: 973408 },
    Sprite { x: -32, y: -8, w: 65, h: 16, off: 973600 },
    Sprite { x: -44, y: -8, w: 89, h: 17, off: 973744 },
    Sprite { x: -30, y: -8, w: 63, h: 17, off: 973948 },
    Sprite { x: -55, y: -8, w: 112, h: 17, off: 974084 },
    Sprite { x: -31, y: -8, w: 64, h: 17, off: 974322 },
    Sprite { x: -31, y: -7, w: 61, h: 16, off: 974458 },
];
/// 「逆位置」の札（中心からの相対）。
pub const RUNE_REV_CHIP: Sprite = Sprite { x: -46, y: -16, w: 92, h: 32, off: 974586 };
/// キーワード（正位置）。
pub const RUNE_WORDS_UP: [Sprite; 24] = [
    Sprite { x: 129, y: 563, w: 219, h: 25, off: 974970 },
    Sprite { x: 101, y: 564, w: 275, h: 24, off: 975670 },
    Sprite { x: 129, y: 563, w: 223, h: 25, off: 976510 },
    Sprite { x: 86, y: 563, w: 304, h: 25, off: 977210 },
    Sprite { x: 143, y: 563, w: 195, h: 25, off: 978160 },
    Sprite { x: 128, y: 563, w: 224, h: 25, off: 978785 },
    Sprite { x: 129, y: 563, w: 223, h: 25, off: 979485 },
    Sprite { x: 129, y: 563, w: 222, h: 25, off: 980185 },
    Sprite { x: 143, y: 563, w: 194, h: 25, off: 980885 },
    Sprite { x: 128, y: 563, w: 223, h: 25, off: 981510 },
    Sprite { x: 128, y: 563, w: 224, h: 25, off: 982210 },
    Sprite { x: 128, y: 564, w: 223, h: 24, off: 982910 },
    Sprite { x: 100, y: 564, w: 274, h: 24, off: 983582 },
    Sprite { x: 114, y: 563, w: 253, h: 25, off: 984422 },
    Sprite { x: 129, y: 563, w: 223, h: 25, off: 985222 },
    Sprite { x: 129, y: 564, w: 221, h: 24, off: 985922 },
    Sprite { x: 128, y: 563, w: 224, h: 25, off: 986594 },
    Sprite { x: 114, y: 563, w: 246, h: 25, off: 987294 },
    Sprite { x: 128, y: 563, w: 223, h: 25, off: 988069 },
    Sprite { x: 114, y: 563, w: 250, h: 25, off: 988769 },
    Sprite { x: 129, y: 563, w: 223, h: 25, off: 989569 },
    Sprite { x: 114, y: 563, w: 251, h: 25, off: 990269 },
    Sprite { x: 100, y: 563, w: 278, h: 25, off: 991069 },
    Sprite { x: 100, y: 563, w: 280, h: 25, off: 991944 },
];
/// 文（正位置・2 行）。
pub const RUNE_TEXT_UP: [Sprite; 24] = [
    Sprite { x: 101, y: 610, w: 274, h: 50, off: 992819 },
    Sprite { x: 142, y: 610, w: 196, h: 50, off: 994569 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 995819 },
    Sprite { x: 151, y: 610, w: 178, h: 50, off: 996969 },
    Sprite { x: 131, y: 610, w: 217, h: 51, off: 998119 },
    Sprite { x: 121, y: 610, w: 234, h: 50, off: 999547 },
    Sprite { x: 121, y: 610, w: 238, h: 50, off: 1001047 },
    Sprite { x: 112, y: 610, w: 253, h: 50, off: 1002547 },
    Sprite { x: 141, y: 610, w: 197, h: 50, off: 1004147 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 1005397 },
    Sprite { x: 144, y: 610, w: 194, h: 50, off: 1006547 },
    Sprite { x: 141, y: 610, w: 197, h: 51, off: 1007797 },
    Sprite { x: 131, y: 610, w: 217, h: 50, off: 1009072 },
    Sprite { x: 121, y: 610, w: 236, h: 50, off: 1010472 },
    Sprite { x: 143, y: 610, w: 194, h: 50, off: 1011972 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 1013222 },
    Sprite { x: 145, y: 610, w: 190, h: 50, off: 1014372 },
    Sprite { x: 142, y: 610, w: 196, h: 50, off: 1015572 },
    Sprite { x: 132, y: 610, w: 213, h: 50, off: 1016822 },
    Sprite { x: 144, y: 610, w: 194, h: 50, off: 1018172 },
    Sprite { x: 131, y: 610, w: 214, h: 50, off: 1019422 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 1020772 },
    Sprite { x: 141, y: 610, w: 197, h: 50, off: 1021922 },
    Sprite { x: 141, y: 610, w: 197, h: 50, off: 1023172 },
];
/// キーワード（逆位置・点対称の文字は正位置と同じ）。
pub const RUNE_WORDS_REV: [Sprite; 24] = [
    Sprite { x: 129, y: 563, w: 220, h: 25, off: 1024422 },
    Sprite { x: 129, y: 563, w: 216, h: 25, off: 1025122 },
    Sprite { x: 171, y: 563, w: 138, h: 25, off: 1025797 },
    Sprite { x: 143, y: 563, w: 193, h: 25, off: 1026247 },
    Sprite { x: 157, y: 563, w: 160, h: 25, off: 1026872 },
    Sprite { x: 129, y: 563, w: 222, h: 25, off: 1027372 },
    Sprite { x: 129, y: 563, w: 223, h: 25, off: 1028072 },
    Sprite { x: 143, y: 563, w: 193, h: 25, off: 1028772 },
    Sprite { x: 143, y: 563, w: 194, h: 25, off: 1029397 },
    Sprite { x: 128, y: 563, w: 223, h: 25, off: 1030022 },
    Sprite { x: 128, y: 563, w: 224, h: 25, off: 1030722 },
    Sprite { x: 128, y: 564, w: 223, h: 24, off: 1031422 },
    Sprite { x: 100, y: 564, w: 274, h: 24, off: 1032094 },
    Sprite { x: 171, y: 563, w: 138, h: 25, off: 1032934 },
    Sprite { x: 157, y: 564, w: 166, h: 24, off: 1033384 },
    Sprite { x: 129, y: 564, w: 221, h: 24, off: 1033888 },
    Sprite { x: 157, y: 563, w: 167, h: 25, off: 1034560 },
    Sprite { x: 142, y: 563, w: 190, h: 25, off: 1035085 },
    Sprite { x: 114, y: 563, w: 253, h: 25, off: 1035685 },
    Sprite { x: 143, y: 564, w: 194, h: 24, off: 1036485 },
    Sprite { x: 114, y: 564, w: 248, h: 24, off: 1037085 },
    Sprite { x: 114, y: 563, w: 251, h: 25, off: 1037829 },
    Sprite { x: 100, y: 563, w: 278, h: 25, off: 1038629 },
    Sprite { x: 143, y: 563, w: 194, h: 25, off: 1039504 },
];
/// 文（逆位置・2 行）。
pub const RUNE_TEXT_REV: [Sprite; 24] = [
    Sprite { x: 122, y: 610, w: 235, h: 50, off: 1040129 },
    Sprite { x: 141, y: 610, w: 194, h: 50, off: 1041629 },
    Sprite { x: 143, y: 610, w: 192, h: 50, off: 1042879 },
    Sprite { x: 141, y: 610, w: 196, h: 50, off: 1044079 },
    Sprite { x: 144, y: 610, w: 191, h: 50, off: 1045329 },
    Sprite { x: 152, y: 610, w: 176, h: 50, off: 1046529 },
    Sprite { x: 121, y: 610, w: 238, h: 50, off: 1047629 },
    Sprite { x: 133, y: 610, w: 212, h: 50, off: 1049129 },
    Sprite { x: 141, y: 610, w: 197, h: 50, off: 1050479 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 1051729 },
    Sprite { x: 144, y: 610, w: 194, h: 50, off: 1052879 },
    Sprite { x: 141, y: 610, w: 197, h: 51, off: 1054129 },
    Sprite { x: 131, y: 610, w: 217, h: 50, off: 1055404 },
    Sprite { x: 131, y: 610, w: 214, h: 50, off: 1056804 },
    Sprite { x: 151, y: 610, w: 174, h: 50, off: 1058154 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 1059254 },
    Sprite { x: 134, y: 610, w: 211, h: 50, off: 1060404 },
    Sprite { x: 132, y: 610, w: 216, h: 50, off: 1061754 },
    Sprite { x: 161, y: 610, w: 154, h: 50, off: 1063104 },
    Sprite { x: 151, y: 610, w: 174, h: 50, off: 1064104 },
    Sprite { x: 141, y: 610, w: 196, h: 50, off: 1065204 },
    Sprite { x: 151, y: 610, w: 177, h: 50, off: 1066454 },
    Sprite { x: 141, y: 610, w: 197, h: 50, off: 1067604 },
    Sprite { x: 142, y: 610, w: 197, h: 50, off: 1068854 },
];
/// カタカナ名の中心の高さ。
pub const RUNE_KANA_Y: i32 = 530;
/// カタカナ名と「逆位置」の札の間隔。
pub const RUNE_KANA_GAP: i32 = 12;
/// 名前（ログ用）。
pub const RUNE_NAMES: [&str; 24] = ["FEHU", "URUZ", "THURISAZ", "ANSUZ", "RAIDHO", "KENAZ", "GEBO", "WUNJO", "HAGALAZ", "NAUDIZ", "ISA", "JERA", "EIHWAZ", "PERTHRO", "ALGIZ", "SOWILO", "TIWAZ", "BERKANO", "EHWAZ", "MANNAZ", "LAGUZ", "INGWAZ", "DAGAZ", "OTHALA"];
