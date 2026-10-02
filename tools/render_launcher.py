"""ランチャー画面（B 案・書物調）を 480×800・4 階調の画像として描く。

Claude Design のキャンバス「Alea ランチャー」の B 案（Liber.dc.html）を Pillow で再現する。
4 倍の解像度で描いてから縮小し（文字と線の縁を滑らかにする）、4 階調に丸める。

出力:
  firmware/alea-fw/assets/launcher.2bpp      … 画素データ（2bit/画素・横に MSB から 4 画素/バイト・
                                               0=黒 1=暗灰 2=明灰 3=白。.a2b のヘッダ無し版）
  firmware/alea-fw/assets/launcher_tiles.rs  … タイル 9 枚の矩形（タップ判定・薄く表示する範囲）
  tools/out/launcher_preview.png             … 確認用（4 階調に丸めた後の見た目）

使い方: python tools/render_launcher.py
書体: tools/fonts/ の Cormorant Garamond（可変・Bold）と Shippori Mincho Bold（いずれも SIL OFL 1.1）。
"""

from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
FONTS = ROOT / "tools" / "fonts"
ASSETS = ROOT / "firmware" / "alea-fw" / "assets"
OUT = ROOT / "tools" / "out"

W, H = 480, 800
S = 4  # 描画の倍率

# 4 階調（デザインの #000 / #555 / #AAA / #FFF）。
BLACK, DARK, LIGHT, WHITE = 0, 85, 170, 255

# タイルの並び（ローマ数字の順）。名称はデザインのとおり。
TILES = [
    ("I", "タロット", "tarot"),
    ("II", "易", "iching"),
    ("III", "ルーン", "rune"),
    ("IV", "ダイス", "dice"),
    ("V", "コイン", "coin"),
    ("VI", "棒倒し", "stick"),
    ("VII", "あみだくじ", "amida"),
    ("VIII", "おみくじ", "omikuji"),
    ("IX", "是か非か", "yesno"),
]


def font_serif(px: int) -> ImageFont.FreeTypeFont:
    f = ImageFont.truetype(str(FONTS / "CormorantGaramond[wght].ttf"), px * S)
    f.set_variation_by_name("Bold")
    return f


def font_mincho(px: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(FONTS / "ShipporiMincho-Bold.ttf"), px * S)


def s(v: float) -> int:
    """1 倍の座標 → 描画倍率の座標。"""
    return round(v * S)


def text_center(d: ImageDraw.ImageDraw, cx: float, cy: float, text: str, font, fill, spacing_em=0.0):
    """文字列を (cx, cy) を中心に描く。spacing_em は字間（CSS の letter-spacing・em）。"""
    size = font.size
    gap = spacing_em * size
    widths = [font.getlength(ch) for ch in text]
    total = sum(widths) + gap * (len(text) - 1)
    x = s(cx) - total / 2
    for ch, w in zip(text, widths):
        d.text((x, s(cy)), ch, font=font, fill=fill, anchor="lm")
        x += w + gap


# ---- アイコン（デザインの SVG・viewBox 0 0 48 48 を 56px で描く） ----

def icon(d: ImageDraw.ImageDraw, key: str, cx: float, cy: float, size: float = 56):
    k = size / 48 * S
    ox, oy = s(cx) - 24 * k, s(cy) - 24 * k

    def p(x, y):
        return (ox + x * k, oy + y * k)

    def lw(w):
        return max(1, round(w * k))

    def line(pts, w, closed=False):
        pts = [p(*q) for q in pts]
        if closed:
            pts = pts + [pts[0]]
        d.line(pts, fill=BLACK, width=lw(w), joint="curve")
        r = lw(w) / 2
        for q in pts:  # 角と端を丸める
            d.ellipse([q[0] - r, q[1] - r, q[0] + r, q[1] + r], fill=BLACK)

    def rrect(x, y, w, h, rx, sw):
        d.rounded_rectangle([*p(x, y), *p(x + w, y + h)], radius=rx * k, outline=BLACK, width=lw(sw))

    def circle(x, y, r, sw=None, fill=False):
        box = [*p(x - r, y - r), *p(x + r, y + r)]
        if fill:
            d.ellipse(box, fill=BLACK)
        else:
            d.ellipse(box, outline=BLACK, width=lw(sw))

    if key == "tarot":
        rrect(11, 4, 26, 40, 3, 2)
        rrect(15, 8, 18, 32, 1, 2)
        star = [(24, 16), (26.4, 21.6), (32, 22), (27.6, 25.6), (29, 31), (24, 28),
                (19, 31), (20.4, 25.6), (16, 22), (21.6, 21.6)]
        line(star, 2, closed=True)
    elif key == "iching":
        for y, broken in [(8, False), (14.5, True), (21, False), (27.5, True), (34, True), (40.5, False)]:
            if broken:
                d.line([p(8, y), p(20, y)], fill=BLACK, width=lw(3))
                d.line([p(28, y), p(40, y)], fill=BLACK, width=lw(3))
            else:
                d.line([p(8, y), p(40, y)], fill=BLACK, width=lw(3))
    elif key == "rune":
        line([(17, 5), (17, 43)], 2.5)
        line([(17, 15), (31, 7)], 2.5)
        line([(17, 25), (31, 17)], 2.5)
    elif key == "dice":
        rrect(7, 7, 34, 34, 6, 2)
        for c in (16, 24, 32):
            circle(c, c, 3, fill=True)
    elif key == "coin":
        circle(24, 24, 19, 2)
        circle(24, 24, 14, 2)
        line([(19, 30), (24, 17), (29, 30)], 2)
        line([(21, 26), (27, 26)], 2)
    elif key == "stick":
        line([(4, 42), (44, 42)], 2)
        line([(12, 40), (38, 10)], 3.5)
        circle(38, 10, 3, fill=True)
    elif key == "amida":
        for x in (11, 24, 37):
            line([(x, 5), (x, 43)], 2)
        for a, b, y in [(11, 24, 13), (24, 37, 22), (11, 24, 31), (24, 37, 38)]:
            line([(a, y), (b, y)], 2)
    elif key == "omikuji":
        rrect(13, 5, 22, 38, 2, 2)
        line([(13, 13), (35, 13)], 2)
        line([(24, 19), (24, 37)], 2)
        line([(19, 23), (29, 23)], 2)
    elif key == "yesno":
        line([(5, 25), (12, 32), (22, 16)], 2.5)
        line([(29, 18), (43, 32)], 2.5)
        line([(43, 18), (29, 32)], 2.5)


def render():
    img = Image.new("L", (W * S, H * S), WHITE)
    d = ImageDraw.Draw(img)

    # 外枠（2px・暗灰）と内枠（1px・黒）。
    d.rectangle([s(14), s(14), s(466) - 1, s(786) - 1], outline=DARK, width=s(2))
    d.rectangle([s(21), s(21), s(459) - 1, s(779) - 1], outline=BLACK, width=s(1))

    # 見出し：ALEA ／ 罫と菱形 ／ 骰子と偶然。
    text_center(d, 240, 66, "ALEA", font_serif(52), BLACK, spacing_em=0.32)
    rule_y = 101
    d.line([s(110), s(rule_y), s(234), s(rule_y)], fill=BLACK, width=s(1))
    d.line([s(246), s(rule_y), s(370), s(rule_y)], fill=BLACK, width=s(1))
    r = 5
    d.polygon([(s(240), s(rule_y - r)), (s(240 + r), s(rule_y)), (s(240), s(rule_y + r)),
               (s(240 - r), s(rule_y))], fill=BLACK)
    text_center(d, 240, 126, "骰子と偶然", font_mincho(18), DARK, spacing_em=0.4)

    # タイル：3×3、内容領域 x 36〜444・y 154〜764、間隔 8px。
    x0, y0, x1, y1, gap = 36, 154, 444, 764, 8
    tw = (x1 - x0 - 2 * gap) / 3
    th = (y1 - y0 - 2 * gap) / 3
    rects = []
    for i, (numeral, name, key) in enumerate(TILES):
        col, row = i % 3, i // 3
        tx = x0 + col * (tw + gap)
        ty = y0 + row * (th + gap)
        rects.append((round(tx), round(ty), round(tw), round(th)))
        d.rectangle([s(tx), s(ty), s(tx + tw) - 1, s(ty + th) - 1], outline=BLACK, width=s(1))
        cx = tx + tw / 2
        cy = ty + th / 2
        text_center(d, cx, cy - 48, numeral, font_serif(15), DARK)
        icon(d, key, cx, cy + 2)
        text_center(d, cx, cy + 52, name, font_mincho(18), BLACK)

    small = img.resize((W, H), Image.LANCZOS)
    a = np.asarray(small, dtype=np.float32)
    levels = np.clip(np.rint(a / 85.0), 0, 3).astype(np.uint8)  # 0=黒 … 3=白
    return levels, rects


def write_outputs(levels: np.ndarray, rects):
    ASSETS.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    # 2bit/画素・MSB から 4 画素/バイト。
    flat = levels.reshape(-1, 4)
    packed = (flat[:, 0] << 6) | (flat[:, 1] << 4) | (flat[:, 2] << 2) | flat[:, 3]
    (ASSETS / "launcher.2bpp").write_bytes(packed.astype(np.uint8).tobytes())
    Image.fromarray((levels * 85).astype(np.uint8)).save(OUT / "launcher_preview.png")

    lines = [
        "// tools/render_launcher.py が生成。手で編集しない。",
        "/// ランチャーのタイル 9 枚の矩形 (x, y, w, h)。ローマ数字 I〜IX の順。",
        "pub const LAUNCHER_TILES: [(i32, i32, i32, i32); 9] = [",
    ]
    for (x, y, w, h), (_, _, key) in zip(rects, TILES):
        lines.append(f"    ({x}, {y}, {w}, {h}), // {key}")
    lines.append("];")
    (ASSETS / "launcher_tiles.rs").write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    lv, rc = render()
    write_outputs(lv, rc)
    counts = np.bincount(lv.ravel(), minlength=4)
    print("levels black/dark/light/white =", counts.tolist())
    print("wrote", ASSETS / "launcher.2bpp", "and", OUT / "launcher_preview.png")
