"""アプリ画面のデザイン案（Claude Design「Alea ランチャー」2 段目）を実機確認用の画像にする。

ダイス 2D6・ダイス 1D100・コイン・是か非かの 4 画面を、キャンバスの配置どおり Pillow で描き、
白黒に 2 値化する（結果の更新はモノクロの部分更新なので、実機の見え方に合わせる）。

出力:
  firmware/alea-fw/assets/mock_{dice,dice100,coin,yesno}.2bpp  … 2bit/画素（0 と 3 だけを使う）
  tools/out/mock_*.png                                          … 確認用

使い方: python tools/render_app_mockups.py
"""

import numpy as np
from PIL import Image, ImageDraw

from render_launcher import (ASSETS, BLACK, OUT, WHITE, H, S, W, font_mincho, font_serif,
                             s, text_center)

# 内容領域（内枠の内側から左右 18px）。
CX0, CX1 = 40, 440
HEADER_TOP, HEADER_H = 22, 112
FOOTER_TOP = 730
# チップの文字の輪郭に足す太さ（描画倍率 4 の px → 実寸で約 0.5px）。最下部の案内には足さない（2026-10-03 確認）。
CHIP_STROKE = 2


def frame(d: ImageDraw.ImageDraw, numeral: str, title: str):
    """二重枠・見出し（ローマ数字／アプリ名／菱形付きの罫）・最下部の案内。"""
    d.rectangle([s(14), s(14), s(466) - 1, s(786) - 1], outline=BLACK, width=s(2))
    d.rectangle([s(21), s(21), s(459) - 1, s(779) - 1], outline=BLACK, width=s(1))
    text_center(d, 240, 45, numeral, font_serif(24), BLACK)
    text_center(d, 240, 83, title, font_mincho(30), BLACK, spacing_em=0.3)
    rule(d, 240, 114, 220, 7)
    d.line([s(CX0), s(FOOTER_TOP), s(CX1), s(FOOTER_TOP)], fill=BLACK, width=s(1))
    # 最下部の案内（20px。15px では白黒で細い線が途切れたため大きくした）。
    f = font_mincho(20)
    d.text((s(CX0), s(754)), "A　戻る", font=f, fill=BLACK, anchor="lm")
    hint = "振って決める"
    gap = 0.2 * f.size
    total = sum(f.getlength(c) for c in hint) + gap * (len(hint) - 1)
    text_center(d, CX1 - total / S / 2, 754, hint, f, BLACK, spacing_em=0.2)


def rule(d: ImageDraw.ImageDraw, cx: float, y: float, width: float, diamond: float):
    """菱形付きの罫（左右の線と中央の菱形）。"""
    r = diamond * 0.71
    half = width / 2
    d.line([s(cx - half), s(y), s(cx - r - 10 + r), s(y)], fill=BLACK, width=s(1))
    d.line([s(cx + 10), s(y), s(cx + half), s(y)], fill=BLACK, width=s(1))
    d.polygon([(s(cx), s(y - r)), (s(cx + r), s(y)), (s(cx), s(y + r)), (s(cx - r), s(y))], fill=BLACK)


def chips(d: ImageDraw.ImageDraw, selected: str):
    labels = ["1D3", "1D4", "1D6", "2D6", "3D6", "1D8", "1D10", "1D12", "1D20", "1D100"]
    gap, top, h = 6, 134, 64
    w = (CX1 - CX0 - 4 * gap) / 5
    f = font_serif(26)
    for i, lab in enumerate(labels):
        x = CX0 + (i % 5) * (w + gap)
        y = top + (i // 5) * (h + gap)
        box = [s(x), s(y), s(x + w) - 1, s(y + h) - 1]
        if lab == selected:
            d.rectangle(box, fill=BLACK)
            text_center(d, x + w / 2, y + h / 2, lab, f, WHITE, stroke=CHIP_STROKE)
        else:
            d.rectangle(box, outline=BLACK, width=s(1))
            text_center(d, x + w / 2, y + h / 2, lab, f, BLACK, stroke=CHIP_STROKE)


def total_block(d: ImageDraw.ImageDraw, label_y: float, value: str):
    text_center(d, 240, label_y, "合計", font_mincho(24), BLACK, spacing_em=0.4)
    text_center(d, 240, label_y + 73, value, font_serif(120), BLACK)


def d6(d: ImageDraw.ImageDraw, cx: float, cy: float, pips):
    x0, y0 = cx - 75, cy - 75
    d.rounded_rectangle([s(x0 + 3), s(y0 + 3), s(x0 + 147), s(y0 + 147)], radius=s(18),
                        outline=BLACK, width=s(4))
    for px, py in pips:
        d.ellipse([s(x0 + px - 12), s(y0 + py - 12), s(x0 + px + 12), s(y0 + py + 12)], fill=BLACK)


def kite(d: ImageDraw.ImageDraw, cx: float, cy: float, value: str):
    x0, y0 = cx - 80, cy - 85
    pts = [(80, 4), (154, 72), (80, 166), (6, 72)]
    poly = [(s(x0 + x), s(y0 + y)) for x, y in pts]
    d.line(poly + [poly[0]], fill=BLACK, width=s(4), joint="curve")
    text_center(d, cx, cy - 6, value, font_serif(58), BLACK)


def mock_dice():
    img, d = new()
    frame(d, "IV", "ダイス")
    chips(d, "2D6")
    d6(d, 145, 414, [(40, 40), (110, 40), (40, 110), (110, 110)])
    d6(d, 335, 414, [(40, 40), (110, 40), (75, 75), (40, 110), (110, 110)])
    total_block(d, 524, "9")
    return img


def mock_dice100():
    img, d = new()
    frame(d, "IV", "ダイス")
    chips(d, "1D100")
    kite(d, 146, 414, "40")
    kite(d, 334, 414, "7")
    total_block(d, 534, "47")
    return img


def mock_coin():
    img, d = new()
    frame(d, "V", "コイン")
    cx, cy = 240, 371
    for r, w in [(150, 5), (131, 2)]:
        d.ellipse([s(cx - r), s(cy - r), s(cx + r) - 1, s(cy + r) - 1], outline=BLACK, width=s(w))
    text_center(d, cx, cy - 10, "A", font_serif(170), BLACK)
    text_center(d, 240, 577, "HEADS", font_serif(52), BLACK, spacing_em=0.3)
    text_center(d, 240, 627, "表", font_mincho(24), BLACK)
    return img


def mock_yesno():
    img, d = new()
    frame(d, "IX", "是か非か")
    text_center(d, 240, 370, "YES", font_serif(150), BLACK, spacing_em=0.08)
    rule(d, 240, 474, 260, 9)
    text_center(d, 240, 536, "是", font_mincho(44), BLACK)
    return img


def new():
    img = Image.new("L", (W * S, H * S), WHITE)
    return img, ImageDraw.Draw(img)


def to_mono_levels(img: Image.Image) -> np.ndarray:
    """縮小して 2 値化（中間は 128 で切る）し、2bit の階調 0（黒）/3（白）にする。"""
    small = np.asarray(img.resize((W, H), Image.LANCZOS))
    return np.where(small < 128, 0, 3).astype(np.uint8)


def write(name: str, levels: np.ndarray):
    flat = levels.reshape(-1, 4)
    packed = (flat[:, 0] << 6) | (flat[:, 1] << 4) | (flat[:, 2] << 2) | flat[:, 3]
    (ASSETS / f"mock_{name}.2bpp").write_bytes(packed.astype(np.uint8).tobytes())
    Image.fromarray((levels * 85).astype(np.uint8)).save(OUT / f"mock_{name}.png")


if __name__ == "__main__":
    ASSETS.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    for name, fn in [("dice", mock_dice), ("dice100", mock_dice100), ("coin", mock_coin), ("yesno", mock_yesno)]:
        write(name, to_mono_levels(fn()))
        print("wrote", name)
