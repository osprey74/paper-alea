"""アプリ画面（ダイス・コイン・是か非か）の画像部品を作る。

Claude Design「Alea ランチャー」2 段目の確定デザインを Pillow で描き、1bit の部品にする。
結果の更新はモノクロの部分更新なので、4 倍で描いて縮小した後に 128 で 2 値化する。

- 台紙：二重枠・見出し・選択チップ・最下部の案内など、結果によらない部分（アプリごとに 1 枚）
- 部品：ダイスの輪郭と D6 の目、コインの面、YES / NO・HEADS / TAILS・表裏・是非・合計
- 字形：出目の数字（Cormorant Garamond Bold・0〜9 と ?）。実機で文字列に組む

出力:
  firmware/alea-fw/assets/app_art.1bpp  … 部品の画素（1bit/画素・行ごとにバイト境界・MSB から・1=黒）
  firmware/alea-fw/assets/app_art.rs   … 部品の位置と大きさ・配置の定数（firmware の ui/art.rs が include する）
  tools/out/app_*.png                  … 確認用（実機と同じ組み方で合成した画面）

使い方: python tools/render_apps.py
書体: tools/fonts/ の Cormorant Garamond（可変・Bold）と Shippori Mincho Bold（いずれも SIL OFL 1.1）。
"""

import math

import numpy as np
from PIL import Image, ImageDraw

from render_launcher import ASSETS, BLACK, OUT, WHITE, H, S, W, font_mincho, font_serif, s, text_center

# ---- 配置（実機で決めた文字の大きさ：HANDOFF §9 の表） ----

# 内容領域（内枠の内側から左右 18px）。
CX0, CX1 = 40, 440
FOOTER_TOP = 730
# チップの文字の輪郭に足す太さ（描画倍率の px → 実寸で約 0.5px）。
CHIP_STROKE = 2
CHIP_LABELS = ["1D3", "1D4", "1D6", "2D6", "3D6", "1D8", "1D10", "1D12", "1D20", "1D100"]
CHIP_TOP, CHIP_H, CHIP_GAP = 134, 64, 6

# ダイスの並び。2 個・3 個は上段にダイス、下に合計。1 個はチップと案内の間の中央。
DICE_ROW_Y = 414
DICE_SINGLE_Y = 492
TOTAL_LABEL_Y_D6 = 524
TOTAL_LABEL_Y_D100 = 534
TOTAL_VALUE_DY = 73
# D6 の大きさの倍率（デザインの 150px を 1 とする）。
D6_SCALES = {"L": 1.4, "M": 1.0, "S": 0.8}
# D6 以外の輪郭の倍率（デザインの D10 の 160×170 を 1 とする）。
DIE_SCALES = {"L": 1.4, "M": 1.0}

# 数字の大きさ。
FONT_SIZES = {"TOTAL": 120, "DIE_M": 58, "DIE_L": 84}
GLYPHS = "0123456789?"

# コイン・是か非か。
COIN_CX, COIN_CY = 240, 371
YESNO_Y = 370


# ---- 共通の描画 ----

def new():
    img = Image.new("L", (W * S, H * S), WHITE)
    return img, ImageDraw.Draw(img)


def to_mask(img: Image.Image, threshold: int = 128) -> np.ndarray:
    """縮小して 2 値化する（True = 黒）。threshold を上げると細い線が残りやすい。"""
    return np.asarray(img.resize((W, H), Image.LANCZOS)) < threshold


def render(fn, threshold: int = 128) -> np.ndarray:
    img, d = new()
    fn(d)
    return to_mask(img, threshold)


def rule(d: ImageDraw.ImageDraw, cx: float, y: float, width: float, diamond: float):
    """菱形付きの罫（左右の線と中央の菱形）。"""
    r = diamond * 0.71
    half = width / 2
    d.line([s(cx - half), s(y), s(cx - 10), s(y)], fill=BLACK, width=s(1))
    d.line([s(cx + 10), s(y), s(cx + half), s(y)], fill=BLACK, width=s(1))
    d.polygon([(s(cx), s(y - r)), (s(cx + r), s(y)), (s(cx), s(y + r)), (s(cx - r), s(y))], fill=BLACK)


def frame(d: ImageDraw.ImageDraw, numeral: str | None, title: str | None, hint: str = "振って決める"):
    """二重枠・見出し（ローマ数字／アプリ名／菱形付きの罫）・最下部の案内。
    title が None なら見出しを描かない（タロットの結果画面）。"""
    d.rectangle([s(14), s(14), s(466) - 1, s(786) - 1], outline=BLACK, width=s(2))
    d.rectangle([s(21), s(21), s(459) - 1, s(779) - 1], outline=BLACK, width=s(1))
    if title is not None:
        if numeral:
            text_center(d, 240, 45, numeral, font_serif(24), BLACK)
        text_center(d, 240, 83, title, font_mincho(30), BLACK, spacing_em=0.3)
        rule(d, 240, 114, 220, 7)
    d.line([s(CX0), s(FOOTER_TOP), s(CX1), s(FOOTER_TOP)], fill=BLACK, width=s(1))
    # 最下部の案内（20px・太くしない）。
    f = font_mincho(20)
    d.text((s(CX0), s(754)), "A　戻る", font=f, fill=BLACK, anchor="lm")
    gap = 0.2 * f.size
    total = sum(f.getlength(c) for c in hint) + gap * (len(hint) - 1)
    text_center(d, CX1 - total / S / 2, 754, hint, f, BLACK, spacing_em=0.2)


def chip_rects():
    """選択チップの矩形 (x, y, w, h)。選択中の反転を画素単位で行うため整数に丸める。"""
    w = (CX1 - CX0 - 4 * CHIP_GAP) / 5
    rects = []
    for i in range(10):
        x0 = round(CX0 + (i % 5) * (w + CHIP_GAP))
        x1 = round(CX0 + (i % 5) * (w + CHIP_GAP) + w)
        y = CHIP_TOP + (i // 5) * (CHIP_H + CHIP_GAP)
        rects.append((x0, y, x1 - x0, CHIP_H))
    return rects


def chips(d: ImageDraw.ImageDraw):
    """選択チップ（すべて未選択の姿。選択中は実機で内側を反転する）。"""
    f = font_serif(26)
    for (x, y, w, h), lab in zip(chip_rects(), CHIP_LABELS):
        d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], outline=BLACK, width=s(1))
        text_center(d, x + w / 2, y + h / 2, lab, f, BLACK, stroke=CHIP_STROKE)


# ---- タロット（B 案・額装） ----

# カード（360×540）の左上。待機は見出しと案内の間の中央、結果は内枠の上から 16px。
TAROT_CARD_W, TAROT_CARD_H = 360, 540
TAROT_WAIT_CARD = (60, 162)
TAROT_RESULT_CARD = (60, 38)
# 結果画面の名前の帯（英名・和名・正逆の札）の中心の高さ。
TAROT_CAP_EN_Y, TAROT_CAP_JA_Y, TAROT_CAP_CHIP_Y = 603, 643, 688
# キーワード画面：正逆の札の中心の高さ、キーワード 4 行の最初の高さと行送り、間の罫。
TAROT_WORD_CHIP_Y = (215, 495)
TAROT_WORD_LINE0, TAROT_WORD_PITCH = 53, 47
TAROT_WORD_RULE_Y = 452


def orient_chip(d: ImageDraw.ImageDraw, cx: int, cy: int, label: str, filled: bool):
    """正位置・逆位置の札。filled なら黒地に白抜き、でなければ枠だけ。矩形 (x, y, w, h) を返す。"""
    f = font_mincho(20)
    gap = 0.2 * f.size
    tw = (sum(f.getlength(c) for c in label) + gap * (len(label) - 1)) / S
    w, h = round(tw) + 24, 32
    x, y = cx - w // 2, cy - h // 2
    if filled:
        d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], fill=BLACK)
    else:
        d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], outline=BLACK, width=s(1))
    text_center(d, cx, cy, label, f, WHITE if filled else BLACK, spacing_em=0.2)
    return (x, y, w, h)


def tarot_word_chip_rects():
    img, d = new()
    return [orient_chip(d, 240, cy, lab, False) for cy, lab in zip(TAROT_WORD_CHIP_Y, ["正位置", "逆位置"])]


# ---- ダイス ----

PIPS = {
    1: [(75, 75)],
    2: [(40, 40), (110, 110)],
    3: [(40, 40), (75, 75), (110, 110)],
    4: [(40, 40), (110, 40), (40, 110), (110, 110)],
    5: [(40, 40), (110, 40), (75, 75), (40, 110), (110, 110)],
    6: [(40, 40), (110, 40), (40, 75), (110, 75), (40, 110), (110, 110)],
}


def d6(d: ImageDraw.ImageDraw, cx: float, cy: float, face: int, k: float):
    """D6（角丸の正方形と目）。face 0 は「?」（待機中）。"""
    x0, y0 = cx - 75 * k, cy - 75 * k
    d.rounded_rectangle([s(x0 + 3 * k), s(y0 + 3 * k), s(x0 + 147 * k), s(y0 + 147 * k)],
                        radius=s(18 * k), outline=BLACK, width=s(4 * k))
    if face == 0:
        text_center(d, cx, cy - 4 * k, "?", font_serif(round(70 * k)), BLACK)
        return
    r = 12 * k
    for px, py in PIPS[face]:
        d.ellipse([s(x0 + px * k - r), s(y0 + py * k - r), s(x0 + px * k + r), s(y0 + py * k + r)], fill=BLACK)


def regular(n: int, r: float, cx: float, cy: float):
    return [(cx + r * math.cos(math.radians(-90 + 360 * i / n)), cy + r * math.sin(math.radians(-90 + 360 * i / n)))
            for i in range(n)]


# D6 以外の輪郭（160×170 の枠の中の座標）と、図形の重心の枠中央（y=85）からのずれ。
# 数字は字の上端と下端の中央をこの重心に合わせる（firmware の `art::text_ink_centered`）。
SHAPES = {
    "TRIANGLE": ([(80, 10), (156, 142), (4, 142)], 13),
    "DIAMOND": ([(80, 4), (156, 85), (80, 166), (4, 85)], 0),
    "KITE": ([(80, 4), (154, 72), (80, 166), (6, 72)], -4),
    "PENTAGON": (regular(5, 84, 80, 90), 5),
    "HEXAGON": (regular(6, 84, 80, 85), 0),
}


def die_outline(d: ImageDraw.ImageDraw, cx: float, cy: float, shape: str, k: float):
    pts, _ = SHAPES[shape]
    poly = [(s(cx + (x - 80) * k), s(cy + (y - 85) * k)) for x, y in pts]
    w = s(4 * k)
    d.line(poly + [poly[0], poly[1]], fill=BLACK, width=w, joint="curve")


# ---- コイン ----

def coin_rings(d: ImageDraw.ImageDraw):
    for r, w in [(150, 5), (131, 2)]:
        d.ellipse([s(COIN_CX - r), s(COIN_CY - r), s(COIN_CX + r) - 1, s(COIN_CY + r) - 1], outline=BLACK, width=s(w))


def coin_face(d: ImageDraw.ImageDraw, face: str):
    coin_rings(d)
    if face == "heads":
        text_center(d, COIN_CX, COIN_CY - 10, "A", font_serif(170), BLACK)
    elif face == "wait":
        text_center(d, COIN_CX, COIN_CY - 10, "?", font_serif(170), BLACK)
    else:
        # 裏：放射状の線（長短 16 本）と中央の菱形（見出しの罫と同じ意匠）。
        for i in range(16):
            a = math.radians(i * 22.5)
            r0, r1 = 34, (104 if i % 2 == 0 else 80)
            d.line([(s(COIN_CX + r0 * math.cos(a)), s(COIN_CY + r0 * math.sin(a))),
                    (s(COIN_CX + r1 * math.cos(a)), s(COIN_CY + r1 * math.sin(a)))], fill=BLACK, width=s(3))
        r = 18
        d.polygon([(s(COIN_CX), s(COIN_CY - r)), (s(COIN_CX + r), s(COIN_CY)), (s(COIN_CX), s(COIN_CY + r)),
                   (s(COIN_CX - r), s(COIN_CY))], fill=BLACK)


# ---- 部品の書き出し ----

class Sprite:
    def __init__(self, mask: np.ndarray, ax: int = 0, ay: int = 0):
        ys, xs = np.nonzero(mask)
        y0, y1, x0, x1 = ys.min(), ys.max() + 1, xs.min(), xs.max() + 1
        self.bits = mask[y0:y1, x0:x1]
        self.x, self.y = int(x0) - ax, int(y0) - ay
        self.h, self.w = self.bits.shape
        self.off = None

    def packed(self) -> bytes:
        return np.packbits(self.bits, axis=1).tobytes()

    def rust(self) -> str:
        return f"Sprite {{ x: {self.x}, y: {self.y}, w: {self.w}, h: {self.h}, off: {self.off} }}"


class Atlas:
    def __init__(self):
        self.data = bytearray()

    def add(self, sp: Sprite) -> Sprite:
        sp.off = len(self.data)
        self.data += sp.packed()
        return sp


def glyph(ch: str, size: int) -> tuple[Sprite, int]:
    """字形 1 つ。位置はペン位置（左端）と文字の中心線からの相対。送り幅は 1/16px。"""
    px, cy = 100, 400
    f = font_serif(size)
    m = render(lambda d: d.text((s(px), s(cy)), ch, font=f, fill=BLACK, anchor="lm"))
    return Sprite(m, px, cy), round(f.getlength(ch) / S * 16)


def build():
    atlas = Atlas()
    out = ["// tools/render_apps.py が生成。手で編集しない。", ""]
    sprites = {}

    def const(name: str, sp: Sprite, doc: str):
        atlas.add(sp)
        sprites[name] = sp
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: Sprite = {sp.rust()};")

    # 台紙。
    const("DICE_FRAME", Sprite(render(lambda d: (frame(d, "IV", "ダイス"), chips(d)))), "ダイスの台紙（チップはすべて未選択の姿）。")
    const("COIN_FRAME", Sprite(render(lambda d: frame(d, "V", "コイン"))), "コインの台紙。")
    const("YESNO_FRAME", Sprite(render(lambda d: (frame(d, "IX", "是か非か"), rule(d, 240, 474, 260, 9)))),
          "是か非かの台紙（結果の下の罫を含む）。")

    # タロット。
    const("TAROT_WAIT_FRAME", Sprite(render(lambda d: frame(d, "I", "タロット", "振って一枚を引く"))),
          "タロットの待機画面の台紙（裏面はカード画像を重ねる）。")
    for name, lab in [("UPRIGHT", "正位置"), ("REVERSED", "逆位置")]:
        const(f"TAROT_CHIP_{name}", Sprite(render(lambda d: orient_chip(d, 240, TAROT_CAP_CHIP_Y, lab, True))),
              f"結果画面の札「{lab}」（黒地に白抜き）。")
    for name, (x, y) in [("TAROT_WAIT_CARD", TAROT_WAIT_CARD), ("TAROT_RESULT_CARD", TAROT_RESULT_CARD)]:
        out.append(f"/// カード（{TAROT_CARD_W}×{TAROT_CARD_H}）の左上。")
        out.append(f"pub const {name}: (i32, i32) = ({x}, {y});")
    out.append("/// キーワード画面の札の矩形 (x, y, w, h)。[正位置, 逆位置]。今の向きの札の内側を反転する。")
    out.append(f"pub const TAROT_WORD_CHIPS: [(i32, i32, i32, i32); 2] = {list(tarot_word_chip_rects())};".replace("[(", "[(").replace(")]", ")]"))

    # ダイス。
    rects = chip_rects()
    out.append("/// 選択チップの矩形 (x, y, w, h)。`DiceKind::ALL` の順。")
    out.append("pub const DICE_CHIPS: [(i32, i32, i32, i32); 10] = [")
    out += [f"    {r}, // {lab}" for r, lab in zip(rects, CHIP_LABELS)]
    out.append("];")
    for name, v, doc in [
        ("DICE_ROW_Y", DICE_ROW_Y, "2 個・3 個のダイスと 1D100 の中心の高さ。"),
        ("DICE_SINGLE_Y", DICE_SINGLE_Y, "1 個のダイスの中心の高さ。"),
        ("TOTAL_LABEL_Y_D6", TOTAL_LABEL_Y_D6, "「合計」の中心の高さ（2D6・3D6）。"),
        ("TOTAL_LABEL_Y_D100", TOTAL_LABEL_Y_D100, "「合計」の中心の高さ（1D100）。"),
        ("TOTAL_VALUE_DY", TOTAL_VALUE_DY, "「合計」から合計値の中心までの距離。"),
    ]:
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: i32 = {v};")
    const("TOTAL_LABEL", Sprite(render(lambda d: text_center(d, 240, 400, "合計", font_mincho(24), BLACK,
                                                            spacing_em=0.4)), 240, 400), "「合計」（中心からの相対）。")
    for size, k in D6_SCALES.items():
        names = []
        for face in range(7):
            sp = atlas.add(Sprite(render(lambda d: d6(d, 240, 400, face, k)), 240, 400))
            names.append(sp.rust())
        out.append(f"/// D6（{round(150 * k)}px）の面。添字 0 は「?」、1〜6 は出目（中心からの相対）。")
        out.append(f"pub const D6_{size}: [Sprite; 7] = [")
        out += [f"    {n}," for n in names]
        out.append("];")
    for size, k in DIE_SCALES.items():
        shapes = SHAPES if size == "L" else {"KITE": SHAPES["KITE"]}
        for shape, (_, dy) in shapes.items():
            sp = atlas.add(Sprite(render(lambda d: die_outline(d, 240, 400, shape, k)), 240, 400))
            out.append(f"/// {shape.lower()} の輪郭（倍率 {k}・中心からの相対）と数字の中心のずれ。")
            out.append(f"pub const DIE_{size}_{shape}: Die = Die {{ outline: {sp.rust()}, text_dy: {round(dy * k)} }};")

    # 数字。
    for name, size in FONT_SIZES.items():
        out.append(f"/// 数字 {size}px（Cormorant Garamond Bold）。")
        out.append(f"pub const FONT_{name}: [Glyph; {len(GLYPHS)}] = [")
        for ch in GLYPHS:
            sp, adv = glyph(ch, size)
            atlas.add(sp)
            out.append(f"    Glyph {{ ch: b'{ch}', adv: {adv}, sprite: {sp.rust()} }},")
        out.append("];")

    # コイン。
    for face in ["heads", "tails", "wait"]:
        const(f"COIN_{face.upper()}", Sprite(render(lambda d: coin_face(d, face))), f"コインの面（{face}）。")
    const("COIN_WORD_HEADS", Sprite(render(lambda d: text_center(d, 240, 577, "HEADS", font_serif(52), BLACK, spacing_em=0.3))), "HEADS。")
    const("COIN_WORD_TAILS", Sprite(render(lambda d: text_center(d, 240, 577, "TAILS", font_serif(52), BLACK, spacing_em=0.3))), "TAILS。")
    const("COIN_KANJI_HEADS", Sprite(render(lambda d: text_center(d, 240, 627, "表", font_mincho(24), BLACK))), "表。")
    const("COIN_KANJI_TAILS", Sprite(render(lambda d: text_center(d, 240, 627, "裏", font_mincho(24), BLACK))), "裏。")

    # 是か非か。
    for name, word in [("YES", "YES"), ("NO", "NO"), ("WAIT", "?")]:
        const(f"YESNO_{name}", Sprite(render(lambda d: text_center(d, 240, YESNO_Y, word, font_serif(150), BLACK,
                                                                    spacing_em=0.08))), f"{word}。")
    const("YESNO_KANJI_YES", Sprite(render(lambda d: text_center(d, 240, 536, "是", font_mincho(44), BLACK))), "是。")
    const("YESNO_KANJI_NO", Sprite(render(lambda d: text_center(d, 240, 536, "非", font_mincho(44), BLACK))), "非。")

    # 棒倒し・あみだくじ・おみくじ（M4）。
    from render_m4 import build_m4
    build_m4(atlas, out)

    # 易・ルーン（M5）。
    from render_m5 import build_m5
    build_m5(atlas, out)

    return atlas, out


# ---- 確認用の合成（firmware と同じ組み方） ----

def blit(canvas: np.ndarray, sp: Sprite, ax: int = 0, ay: int = 0):
    y, x = sp.y + ay, sp.x + ax
    canvas[y:y + sp.h, x:x + sp.w] |= sp.bits


def text(canvas: np.ndarray, size: int, value: str, cx: int, cy: int, ink=False, cache={}):
    gl = []
    for ch in value:
        key = (ch, size)
        if key not in cache:
            cache[key] = glyph(ch, size)
        gl.append(cache[key])
    if ink:  # 字の上端と下端の中央を cy に合わせる（firmware の text_ink_centered と同じ）
        top = min(sp.y for sp, _ in gl)
        bottom = max(sp.y + sp.h for sp, _ in gl)
        cy -= int((top + bottom) / 2)
    total = sum(a for _, a in gl)
    pen = cx * 16 - total // 2
    for sp, adv in gl:
        blit(canvas, sp, (pen + 8) // 16, cy)
        pen += adv


def previews():
    def save(name, canvas):
        Image.fromarray(np.where(canvas, 0, 255).astype(np.uint8)).save(OUT / f"app_{name}.png")

    dice_frame = Sprite(render(lambda d: (frame(d, "IV", "ダイス"), chips(d))))

    def dice_base(selected: int):
        c = np.zeros((H, W), bool)
        blit(c, dice_frame)
        x, y, w, h = chip_rects()[selected]
        c[y + 1:y + h - 1, x + 1:x + w - 1] ^= True
        return c

    def d6_face(face, k, cx, cy):
        return Sprite(render(lambda d: d6(d, cx, cy, face, k)))

    # 2D6（4・5 → 9）
    c = dice_base(3)
    for cx, f in [(145, 4), (335, 5)]:
        blit(c, d6_face(f, 1.0, cx, DICE_ROW_Y))
    blit(c, Sprite(render(lambda d: text_center(d, 240, TOTAL_LABEL_Y_D6, "合計", font_mincho(24), BLACK, spacing_em=0.4))))
    text(c, 120, "9", 240, TOTAL_LABEL_Y_D6 + TOTAL_VALUE_DY)
    save("dice_2d6", c)

    # 3D6（2・6・3 → 11）
    c = dice_base(4)
    for cx, f in [(100, 2), (240, 6), (380, 3)]:
        blit(c, d6_face(f, 0.8, cx, DICE_ROW_Y))
    blit(c, Sprite(render(lambda d: text_center(d, 240, TOTAL_LABEL_Y_D6, "合計", font_mincho(24), BLACK, spacing_em=0.4))))
    text(c, 120, "11", 240, TOTAL_LABEL_Y_D6 + TOTAL_VALUE_DY)
    save("dice_3d6", c)

    # 1D6（待機）
    c = dice_base(2)
    blit(c, d6_face(0, 1.4, 240, DICE_SINGLE_Y))
    save("dice_1d6_wait", c)

    # 1D100（40・7 → 47）
    c = dice_base(9)
    for cx, v in [(146, "40"), (334, "7")]:
        blit(c, Sprite(render(lambda d: die_outline(d, cx, DICE_ROW_Y, "KITE", 1.0))))
        text(c, 58, v, cx, DICE_ROW_Y + SHAPES["KITE"][1], ink=True)
    blit(c, Sprite(render(lambda d: text_center(d, 240, TOTAL_LABEL_Y_D100, "合計", font_mincho(24), BLACK, spacing_em=0.4))))
    text(c, 120, "47", 240, TOTAL_LABEL_Y_D100 + TOTAL_VALUE_DY)
    save("dice_1d100", c)

    # 1 個の多面体（1D4・1D8・1D10・1D12・1D20）
    for i, shape, v in [(1, "TRIANGLE", "3"), (5, "DIAMOND", "7"), (6, "KITE", "10"), (7, "PENTAGON", "12"),
                        (8, "HEXAGON", "20")]:
        c = dice_base(i)
        blit(c, Sprite(render(lambda d: die_outline(d, 240, DICE_SINGLE_Y, shape, 1.4))))
        text(c, FONT_SIZES["DIE_L"], v, 240, DICE_SINGLE_Y + round(SHAPES[shape][1] * 1.4), ink=True)
        save(f"dice_{shape.lower()}", c)

    for face, word, kanji in [("heads", "HEADS", "表"), ("tails", "TAILS", "裏"), ("wait", None, None)]:
        c = np.zeros((H, W), bool)
        blit(c, Sprite(render(lambda d: (frame(d, "V", "コイン"), coin_face(d, face)))))
        if word:
            blit(c, Sprite(render(lambda d: (text_center(d, 240, 577, word, font_serif(52), BLACK, spacing_em=0.3),
                                             text_center(d, 240, 627, kanji, font_mincho(24), BLACK)))))
        save(f"coin_{face}", c)

    for word, kanji in [("YES", "是"), ("NO", "非"), ("?", None)]:
        def draw(d):
            frame(d, "IX", "是か非か")
            rule(d, 240, 474, 260, 9)
            text_center(d, 240, YESNO_Y, word, font_serif(150), BLACK, spacing_em=0.08)
            if kanji:
                text_center(d, 240, 536, kanji, font_mincho(44), BLACK)
        save(f"yesno_{'wait' if word == '?' else word.lower()}", render(draw))


if __name__ == "__main__":
    ASSETS.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    atlas, lines = build()
    (ASSETS / "app_art.1bpp").write_bytes(bytes(atlas.data))
    (ASSETS / "app_art.rs").write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote app_art.1bpp ({len(atlas.data)} bytes) and app_art.rs")
    previews()
    print("wrote previews to", OUT)
