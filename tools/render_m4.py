"""M4（棒倒し・あみだくじ・おみくじ）の画像部品。`render_apps.py` から呼ばれ、同じ部品の束に加える。

確定デザイン：Claude Design「Alea ランチャー」4 段目（2026-10-03）。
- 棒倒し：方式の札（左右／八方位）・方位盤と倒れた棒・方位名（和英）、左右は地面と倒れた棒・左右（和英）
- あみだくじ：本数の案内・上の札のローマ数字・下の番号。縦線・横線・道筋は実機で描く
- おみくじ：紙片の台紙・番号・運勢・一言（縦書き）。文は tools/data/omikuji.json
"""

import json
import math
from pathlib import Path

from render_apps import BLACK, WHITE, S, Sprite, frame, render, s, text_center
from render_launcher import font_mincho, font_serif

DATA = Path(__file__).resolve().parent / "data"

# ---- 棒倒し ----
STICK_CHIP_Y, STICK_CHIP_W, STICK_CHIP_H = 154, 110, 40
COMPASS_C, COMPASS_R = (240, 382), 150
DIR_JA = ["北", "北東", "東", "南東", "南", "南西", "西", "北西"]
DIR_EN = ["NORTH", "NORTH-EAST", "EAST", "SOUTH-EAST", "SOUTH", "SOUTH-WEST", "WEST", "NORTH-WEST"]
EIGHT_JA_Y, EIGHT_EN_Y = 606, 656
GROUND_Y, PIVOT_X = 410, 240
SIDE_JA_Y, SIDE_EN_Y = 540, 620

# ---- あみだくじ ----
# 本数の案内は 22px（20px では白黒で潰れた・2026-10-03 実機）。
AMIDA_NOTE_Y = 152
AMIDA_BOX_TOP, AMIDA_BOX_W, AMIDA_BOX_H = 176, 52, 44
AMIDA_LADDER_TOP, AMIDA_LADDER_BOTTOM = 228, 642
AMIDA_GOAL_TOP, AMIDA_GOAL_H = 652, 48
AMIDA_SPAN, AMIDA_MAX_SPACING = 340, 110
KANSUJI = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九", "十"]
ROMAN = ["I", "II", "III", "IV", "V", "VI"]

# ---- おみくじ ----
STRIP = (115, 162, 250, 540)  # 紙片の外枠 (x, y, w, h)
# 番号は 20px（18px では白黒で潰れた）。11 以降の数は「第」と「番」の間を横書き（縦中横）にする（2026-10-03 実機）。
OMI_NUM_TOP, OMI_NUM_SIZE, OMI_NUM_ZONE, OMI_NUM_SPACING = 180, 20, 104, 0.25
# 明朝の細い横画（二・三）が白黒で消えないよう、2 値化の境目を明るい側に寄せる。
OMI_NUM_THRESHOLD = 170
OMI_RULE1_Y = 290
OMI_FORTUNE_C, OMI_FORTUNE_SIZE = 364, 68
OMI_RULE2_Y = 440
OMI_MSG_TOP, OMI_MSG_SIZE, OMI_MSG_PITCH = 454, 22, 38
SMALL_KANA = set("ぁぃぅぇぉっゃゅょゎァィゥェォッャュョヮ")


def kansuji(n: int) -> str:
    """1〜99 の漢数字（十七・二十・三十）。"""
    tens, ones = divmod(n, 10)
    out = ("" if tens == 0 else ("十" if tens == 1 else KANSUJI[tens] + "十"))
    return out + ("" if ones == 0 else KANSUJI[ones])


def vertical(d, cx: float, top: float, text: str, size: int, spacing_em: float = 0.0, stroke: int = 0):
    """縦書き（1 字ずつ中央に置く）。小書きの仮名は右上に寄せる（縦書き用の字形の代わり）。"""
    f = font_mincho(size)
    pitch = size * (1 + spacing_em)
    for i, ch in enumerate(text):
        x, y = cx, top + i * pitch + size / 2
        if ch in SMALL_KANA:
            x, y = x + 0.12 * size, y - 0.12 * size
        d.text((s(x), s(y)), ch, font=f, fill=BLACK, anchor="mm", stroke_width=stroke, stroke_fill=BLACK)


def vertical_len(text: str, size: int, spacing_em: float = 0.0) -> float:
    return len(text) * size * (1 + spacing_em) - size * spacing_em


def chip_rects():
    gap = 6
    x0 = 240 - STICK_CHIP_W - gap // 2
    return [(x0 + i * (STICK_CHIP_W + gap), STICK_CHIP_Y - STICK_CHIP_H // 2, STICK_CHIP_W, STICK_CHIP_H) for i in range(2)]


def stick_frame(d):
    frame(d, "VI", "棒倒し", "B　方式　・　振って倒す")
    f = font_mincho(20)
    for (x, y, w, h), lab in zip(chip_rects(), ["左右", "八方位"]):
        d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], outline=BLACK, width=s(1))
        text_center(d, x + w / 2, y + h / 2, lab, f, BLACK, spacing_em=0.2)


def compass(d):
    cx, cy = COMPASS_C
    R = COMPASS_R
    d.ellipse([s(cx - R), s(cy - R), s(cx + R), s(cy + R)], outline=BLACK, width=s(2))
    d.ellipse([s(cx - R + 16), s(cy - R + 16), s(cx + R - 16), s(cy + R - 16)], outline=BLACK, width=s(1))
    f = font_mincho(16)
    for i, n in enumerate(DIR_JA):
        a = math.radians(-90 + 45 * i)
        d.line([(s(cx + (R - 10) * math.cos(a)), s(cy + (R - 10) * math.sin(a))),
                (s(cx + R * math.cos(a)), s(cy + R * math.sin(a)))], fill=BLACK, width=s(2))
        text_center(d, cx + (R + 24) * math.cos(a), cy + (R + 24) * math.sin(a), n, f, BLACK)


def pivot(d, cx, cy):
    """棒の支点（白抜きの小円）。"""
    d.ellipse([s(cx - 8.5), s(cy - 8.5), s(cx + 8.5), s(cy + 8.5)], fill=BLACK)
    d.ellipse([s(cx - 5.5), s(cy - 5.5), s(cx + 5.5), s(cy + 5.5)], fill=WHITE)


def stick_line(d, x0, y0, x1, y1):
    """棒（太さ 9・端は丸）と先の重り。"""
    r = 4.5
    d.line([(s(x0), s(y0)), (s(x1), s(y1))], fill=BLACK, width=s(9))
    for x, y in [(x0, y0), (x1, y1)]:
        d.ellipse([s(x - r), s(y - r), s(x + r), s(y + r)], fill=BLACK)
    d.ellipse([s(x1 - 11), s(y1 - 11), s(x1 + 11), s(y1 + 11)], fill=BLACK)


def stick_dir(d, k):
    cx, cy = COMPASS_C
    a = math.radians(-90 + 45 * k)
    L = COMPASS_R - 28
    stick_line(d, cx, cy, cx + L * math.cos(a), cy + L * math.sin(a))
    pivot(d, cx, cy)


def ground(d):
    d.line([(s(60), s(GROUND_Y)), (s(420), s(GROUND_Y))], fill=BLACK, width=s(2))
    for y in range(GROUND_Y - 110, GROUND_Y, 10):  # 破線（立っていた位置）
        d.line([(s(PIVOT_X), s(y)), (s(PIVOT_X), s(y + 4))], fill=BLACK, width=s(2))
    d.polygon([(s(PIVOT_X), s(GROUND_Y)), (s(PIVOT_X - 8), s(GROUND_Y + 14)), (s(PIVOT_X + 8), s(GROUND_Y + 14))], fill=BLACK)


def stick_side(d, right: bool):
    sign = 1 if right else -1
    y = GROUND_Y - 9
    stick_line(d, PIVOT_X, y, PIVOT_X + sign * 142, y)
    # 倒れた向きを示す弧と矢じり（2 次ベジェ）。
    p0, p1, p2 = (PIVOT_X + sign * 50, GROUND_Y - 80), (PIVOT_X + sign * 110, GROUND_Y - 78), (PIVOT_X + sign * 130, GROUND_Y - 32)
    pts = []
    for i in range(21):
        t = i / 20
        x = (1 - t) ** 2 * p0[0] + 2 * (1 - t) * t * p1[0] + t * t * p2[0]
        y2 = (1 - t) ** 2 * p0[1] + 2 * (1 - t) * t * p1[1] + t * t * p2[1]
        pts.append((s(x), s(y2)))
    d.line(pts, fill=BLACK, width=s(2))
    tip = (PIVOT_X + sign * 130, GROUND_Y - 30)
    d.line([(s(tip[0] - sign * 8), s(tip[1] - 10)), (s(tip[0]), s(tip[1])), (s(tip[0] + sign * 9), s(tip[1] - 9))],
           fill=BLACK, width=s(2))


def stick_stand(d):
    stick_line(d, PIVOT_X, GROUND_Y - 4, PIVOT_X, GROUND_Y - 130)


# ---- おみくじ ----

def omikuji_frame(d):
    frame(d, "VIII", "おみくじ", "振って引く")
    x, y, w, h = STRIP
    d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], outline=BLACK, width=s(2))
    d.rectangle([s(x + 7), s(y + 7), s(x + w - 7) - 1, s(y + h - 7) - 1], outline=BLACK, width=s(1))
    for ry in (OMI_RULE1_Y, OMI_RULE2_Y):
        d.line([(s(170), s(ry)), (s(310), s(ry))], fill=BLACK, width=s(1))


def omikuji_fortune(d, label):
    size = OMI_FORTUNE_SIZE
    top = OMI_FORTUNE_C - vertical_len(label, size) / 2
    vertical(d, 240, top, label, size)


def omikuji_number(d, n):
    """「第N番」を縦に並べる。N が 2 字以上（十一以降）なら N の部分は横書きで 1 字分の高さに収める。"""
    num = kansuji(n)
    cells = ["第"] + ([num] if len(num) > 1 else list(num)) + ["番"]
    size = OMI_NUM_SIZE
    pitch = size * (1 + OMI_NUM_SPACING)
    h = len(cells) * pitch - size * OMI_NUM_SPACING
    assert h <= OMI_NUM_ZONE + 0.5, num
    top = OMI_NUM_TOP + (OMI_NUM_ZONE - h) / 2
    f = font_mincho(size)
    for i, cell in enumerate(cells):
        text_center(d, 240, top + i * pitch + size / 2, cell, f, BLACK)


def omikuji_message(d, lines):
    k = len(lines)
    for i, line in enumerate(lines):  # 右の行から
        cx = 240 + ((k - 1) / 2 - i) * OMI_MSG_PITCH
        vertical(d, cx, OMI_MSG_TOP, line, OMI_MSG_SIZE)


def build_m4(atlas, out):
    """部品を `atlas` に足し、Rust の定義を `out` に書き足す。"""

    def add(sp: Sprite) -> str:
        atlas.add(sp)
        return sp.rust()

    def const(name, fn, doc, anchor=(0, 0)):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: Sprite = {add(Sprite(render(fn), *anchor))};")

    def array(name, fns, doc, anchor=(0, 0), threshold=128):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: [Sprite; {len(fns)}] = [")
        out.extend(f"    {add(Sprite(render(fn, threshold), *anchor))}," for fn in fns)
        out.append("];")

    def num(name, v, doc, ty="i32"):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: {ty} = {v};")

    # 棒倒し。
    const("STICK_FRAME", stick_frame, "棒倒しの台紙（方式の札はどちらも未選択の姿）。")
    num("STICK_CHIPS", chip_rects(), "方式の札の矩形 (x, y, w, h)。[左右, 八方位]。", "[(i32, i32, i32, i32); 2]")
    const("STICK_COMPASS", compass, "方位盤。")
    array("STICK_DIR", [lambda d, k=k: stick_dir(d, k) for k in range(8)], "8 方位に倒れた棒（0=北から時計回り）。")
    const("STICK_PIVOT", lambda d: pivot(d, *COMPASS_C), "方位盤の中心の支点（待機中）。")
    array("STICK_DIR_JA", [lambda d, t=t: text_center(d, 240, EIGHT_JA_Y, t, font_mincho(52), BLACK, spacing_em=0.2)
                           for t in DIR_JA], "方位名（和）。")
    array("STICK_DIR_EN", [lambda d, t=t: text_center(d, 240, EIGHT_EN_Y, t, font_serif(22), BLACK, spacing_em=0.3)
                           for t in DIR_EN], "方位名（英）。")
    const("STICK_EIGHT_WAIT", lambda d: text_center(d, 240, EIGHT_JA_Y, "?", font_serif(60), BLACK), "8 方位の待機中の「?」。")
    const("STICK_GROUND", ground, "地面・立っていた位置の破線・支点。")
    array("STICK_SIDE", [lambda d, r=r: stick_side(d, r) for r in (False, True)], "左右に倒れた棒。[左, 右]。")
    const("STICK_STAND", stick_stand, "立っている棒（左右の待機中）。")
    array("STICK_SIDE_JA", [lambda d, t=t: text_center(d, 240, SIDE_JA_Y, t, font_mincho(96), BLACK) for t in "左右"],
          "左・右（和）。")
    array("STICK_SIDE_EN", [lambda d, t=t: text_center(d, 240, SIDE_EN_Y, t, font_serif(22), BLACK, spacing_em=0.3)
                            for t in ["LEFT", "RIGHT"]], "LEFT・RIGHT。")
    const("STICK_SIDE_WAIT", lambda d: text_center(d, 240, SIDE_JA_Y, "?", font_serif(96), BLACK), "左右の待機中の「?」。")

    # あみだくじ。
    const("AMIDA_FRAME", lambda d: frame(d, "VII", "あみだくじ", "B　本数　・　振って引く"), "あみだくじの台紙。")
    array("AMIDA_NOTE", [lambda d, n=n: text_center(d, 240, AMIDA_NOTE_Y, f"{KANSUJI[n]}本　・　上の番号をタップ",
                                                     font_mincho(22), BLACK, spacing_em=0.2) for n in range(2, 7)],
          "本数の案内（2〜6 本）。")
    array("AMIDA_ROMAN", [lambda d, t=t: text_center(d, 240, 400, t, font_serif(24), BLACK) for t in ROMAN],
          "上の札のローマ数字（中心からの相対）。", (240, 400))
    array("AMIDA_GOAL", [lambda d, t=t: text_center(d, 240, 400, t, font_serif(34), BLACK) for t in "123456"],
          "下の番号（中心からの相対）。", (240, 400))
    for name, v, doc in [
        ("AMIDA_BOX_TOP", AMIDA_BOX_TOP, "上の札の上端。"), ("AMIDA_BOX_W", AMIDA_BOX_W, "上の札・下の番号の枠の幅。"),
        ("AMIDA_BOX_H", AMIDA_BOX_H, "上の札の高さ。"), ("AMIDA_LADDER_TOP", AMIDA_LADDER_TOP, "縦線の上端。"),
        ("AMIDA_LADDER_BOTTOM", AMIDA_LADDER_BOTTOM, "縦線の下端。"), ("AMIDA_GOAL_TOP", AMIDA_GOAL_TOP, "下の番号の枠の上端。"),
        ("AMIDA_GOAL_H", AMIDA_GOAL_H, "下の番号の枠の高さ。"), ("AMIDA_SPAN", AMIDA_SPAN, "両端の縦線の間隔の上限。"),
        ("AMIDA_MAX_SPACING", AMIDA_MAX_SPACING, "縦線の間隔の上限（本数が少ないとき）。"),
    ]:
        num(name, v, doc)

    # おみくじ。
    data = json.loads((DATA / "omikuji.json").read_text(encoding="utf-8"))
    fortunes = data["fortunes"]
    const("OMIKUJI_FRAME", omikuji_frame, "おみくじの台紙（紙片と 2 本の罫を含む）。")
    array("OMIKUJI_FORTUNE", [lambda d, t=f["label"]: omikuji_fortune(d, t) for f in fortunes], "運勢（縦書き）。")
    const("OMIKUJI_WAIT", lambda d: text_center(d, 240, OMI_FORTUNE_C, "?", font_serif(72), BLACK), "待機中の「?」。")
    msgs = [m for f in fortunes for m in f["messages"]]
    for m in msgs:
        for line in m:
            assert vertical_len(line, OMI_MSG_SIZE) <= STRIP[1] + STRIP[3] - 18 - OMI_MSG_TOP, line
    array("OMIKUJI_NUMBER", [lambda d, n=n: omikuji_number(d, n) for n in range(1, len(msgs) + 1)],
          "番号（第一番〜・縦書き）。運勢の順に一言を通し番号にしたもの。", threshold=OMI_NUM_THRESHOLD)
    array("OMIKUJI_MESSAGE", [lambda d, m=m: omikuji_message(d, m) for m in msgs], "一言（縦書き・通し番号順）。")
    num("OMIKUJI_WEIGHTS", [f["weight"] for f in fortunes], "運勢の重み。", f"[u32; {len(fortunes)}]")
    num("OMIKUJI_COUNTS", [len(f["messages"]) for f in fortunes], "運勢ごとの一言の数。", f"[u32; {len(fortunes)}]")
    out.append("/// 運勢の名前（ログ用）。")
    out.append(f"pub const OMIKUJI_LABELS: [&str; {len(fortunes)}] = [" + ", ".join(f'"{f["label"]}"' for f in fortunes) + "];")
