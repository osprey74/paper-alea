"""記事・紹介用のスクリーンショットを作る（実機のファームと同じ組み方で画面を合成する）。

`firmware/alea-fw/assets/app_art.{1bpp,rs}` の部品と microSD 用の画像（`sd/alea/tarot/`）を、
各アプリの `draw` と同じ順・同じ座標で重ねる。4 階調は 0/85/170/255 の灰色で書き出す。

出力: tools/out/screenshots/*.png（480×800）と *_2x.png（960×1600・最近傍で拡大）
タロットの画面はカードの絵柄（© osprey74）を含む。リポジトリには入れない（tools/out/ は .gitignore 済み）。

使い方: python tools/screenshots.py
"""

import re
import struct
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "firmware" / "alea-fw" / "assets"
SD = ROOT / "sd" / "alea" / "tarot"
OUT = ROOT / "tools" / "out" / "screenshots"
W, H = 480, 800

ATLAS = (ASSETS / "app_art.1bpp").read_bytes()
SPRITE = re.compile(r"Sprite \{ x: (-?\d+), y: (-?\d+), w: (\d+), h: (\d+), off: (\d+) \}")


def parse_art():
    """app_art.rs の定義を {名前: 値} にする（Sprite は (x, y, w, h, off)、配列はリスト）。"""
    defs, name, items = {}, None, None
    for line in (ASSETS / "app_art.rs").read_text(encoding="utf-8").splitlines():
        m = re.match(r"pub const (\w+): ([^=]+)= (.*)$", line)
        if m:
            name, rhs = m.group(1), m.group(3)
            if rhs == "[":
                items = []
                continue
            sp = SPRITE.findall(rhs)
            if sp:
                v = tuple(map(int, sp[0]))
                dy = re.search(r"text_dy: (-?\d+)", rhs)
                defs[name] = (v, int(dy.group(1))) if dy else v
            else:
                defs[name] = eval(rhs.rstrip(";").replace("true", "True").replace("false", "False"))
            continue
        if items is not None:
            if line.startswith("];"):
                defs[name], items = items, None
                continue
            sp = SPRITE.findall(line)
            g = re.search(r"ch: b'(.)', adv: (\d+)", line)
            if sp:
                v = tuple(map(int, sp[0]))
            else:  # (x, y, w, h), // コメント の行
                tup = re.search(r"\(([-\d, ]+)\)", line)
                v = tuple(int(n) for n in tup.group(1).split(",")) if tup else None
            items.append((g.group(1), int(g.group(2)), v) if g else v)
    return defs


A = parse_art()


def canvas():
    return np.full((H, W), 3, np.uint8)  # 0=黒〜3=白


def blit1(c, x, y, w, h, data):
    stride = (w + 7) // 8
    bits = np.unpackbits(np.frombuffer(data, np.uint8)[: stride * h].reshape(h, stride), axis=1)[:, :w].astype(bool)
    c[y:y + h, x:x + w][bits] = 0


def draw(c, sp, at=(0, 0)):
    x, y, w, h, off = sp
    blit1(c, at[0] + x, at[1] + y, w, h, ATLAS[off:off + (w + 7) // 8 * h])


def text(c, font, s, center):
    glyphs = [g for ch in s for g in font if g[0] == ch]
    total = sum(g[1] for g in glyphs)
    pen = center[0] * 16 - total // 2
    for _, adv, sp in glyphs:
        draw(c, sp, ((pen + 8) // 16, center[1]))
        pen += adv


def invert(c, x, y, w, h):
    c[y:y + h, x:x + w] = 3 - c[y:y + h, x:x + w]


def inset_invert(c, rect):
    x, y, w, h = rect
    invert(c, x + 1, y + 1, w - 2, h - 2)


def a1b(c, path):
    b = path.read_bytes()
    w, h, x, y = struct.unpack("<HHHH", b[4:12])
    blit1(c, x, y, w, h, b[16:])


def a2b(c, path, x, y, rot180=False):
    b = path.read_bytes()
    w, h = struct.unpack("<HH", b[4:8])
    v = np.frombuffer(b[16:16 + w * h // 4], np.uint8)
    lv = np.stack([(v >> 6) & 3, (v >> 4) & 3, (v >> 2) & 3, v & 3], 1).reshape(h, w)
    c[y:y + h, x:x + w] = lv[::-1, ::-1] if rot180 else lv


def rect_fill(c, x, y, w, h):
    c[y:y + h, x:x + w] = 0


# ---- 各画面（apps/*.rs の draw と同じ順） ----

def tarot_result(card=21, reversed_=False):
    c = canvas()
    n = f"{card:02}"
    a1b(c, SD / "cap" / f"{n}.a1b")
    a2b(c, SD / "img" / f"{n}.a2b", *A["TAROT_RESULT_CARD"], reversed_)
    draw(c, A["TAROT_CHIP_REVERSED" if reversed_ else "TAROT_CHIP_UPRIGHT"])
    return c


def tarot_words(card=21, reversed_=False):
    c = canvas()
    a1b(c, SD / "word" / f"{card:02}.a1b")
    inset_invert(c, A["TAROT_WORD_CHIPS"][int(reversed_)])
    return c


def dice_3d6(faces=(4, 6, 5)):
    c = canvas()
    draw(c, A["DICE_FRAME"])
    inset_invert(c, A["DICE_CHIPS"][4])  # 3D6
    for x, f in zip([100, 240, 380], faces):
        draw(c, A["D6_S"][f], (x, A["DICE_ROW_Y"]))
    y = A["TOTAL_LABEL_Y_D6"]
    draw(c, A["TOTAL_LABEL"], (240, y))
    text(c, A["FONT_TOTAL"], str(sum(faces)), (240, y + A["TOTAL_VALUE_DY"]))
    return c


def stick_eight(direction=1):
    c = canvas()
    draw(c, A["STICK_FRAME"])
    inset_invert(c, A["STICK_CHIPS"][1])  # 八方位
    for name in ["STICK_COMPASS"]:
        draw(c, A[name])
    for name in ["STICK_DIR", "STICK_DIR_JA", "STICK_DIR_EN"]:
        draw(c, A[name][direction])
    return c


def iching_result(values=(7, 7, 9, 8, 6, 8)):
    """values は下の爻から（6 老陰・7 少陽・8 少陰・9 老陽）。"""
    king_wen = [2, 24, 7, 19, 15, 36, 46, 11, 16, 51, 40, 54, 62, 55, 32, 34, 8, 3, 29, 60, 39, 63, 48, 5, 45, 17,
                47, 58, 31, 49, 28, 43, 23, 27, 4, 41, 52, 22, 18, 26, 35, 21, 64, 38, 56, 30, 50, 14, 20, 42, 59,
                61, 53, 37, 57, 9, 12, 25, 6, 10, 33, 13, 44, 1]
    yang = [v % 2 == 1 for v in values]
    changing = [v in (6, 9) for v in values]
    primary = sum(1 << i for i, y in enumerate(yang) if y)
    mask = sum(1 << i for i, ch in enumerate(changing) if ch)
    changed = primary ^ mask if mask else None

    c = canvas()
    draw(c, A["IC_RESULT_FRAME"])
    cols = A["IC_COL_CX"] if changed is not None else [240]
    fw, bar, gap, top = A["IC_FIG_W"], A["IC_FIG_BAR"], A["IC_FIG_GAP"], A["IC_FIG_TOP"]
    for k, cx in enumerate(cols):
        code = primary if k == 0 else changed
        no = king_wen[code] - 1
        draw(c, A["IC_LABEL_PRIMARY" if k == 0 else "IC_LABEL_CHANGED"], (cx, 0))
        x = cx - fw // 2
        for i in range(6):
            y = top + (5 - i) * (bar + gap)
            if code >> i & 1:
                rect_fill(c, x, y, fw, bar)
            else:
                seg = fw * 41 // 100
                rect_fill(c, x, y, seg, bar)
                rect_fill(c, x + fw - seg, y, seg, bar)
            if k == 0 and changing[i]:
                draw(c, A["IC_MARK_O" if yang[i] else "IC_MARK_X"], (x + fw + 18, y + bar // 2))
        draw(c, A["IC_NAME"][no], (cx, 0))
        draw(c, A["IC_NUMBER_READING"][no], (cx, 0))
        t = A["IC_SUM_TOP"][k]
        draw(c, A["IC_SUM_PRIMARY" if k == 0 else "IC_SUM_CHANGED"], (0, t))
        draw(c, A["IC_SUMMARY"][no], (0, t))
    if changed is not None:
        draw(c, A["IC_ARROW"])
    row = ["変", "爻", "　"]
    pos = [p for p, ch in zip("初二三四五上", changing) if ch]
    for i, p in enumerate(pos):
        row += (["・"] if i else []) + [p]
    if not pos:
        row += ["な", "し"]
    adv = A["IC_CHANGES_ADV"]
    x0 = 240 - adv * len(row) // 2 + adv // 2
    for k, ch in enumerate(row):
        if ch in A["IC_CHANGE_CHARS"]:
            draw(c, A["IC_CHANGE_GLYPHS"][A["IC_CHANGE_CHARS"].index(ch)], (x0 + k * adv, A["IC_CHANGES_Y"]))
    return c


def save(name, c):
    OUT.mkdir(parents=True, exist_ok=True)
    img = Image.fromarray((c * 85).astype(np.uint8))
    img.save(OUT / f"{name}.png")
    img.resize((W * 2, H * 2), Image.NEAREST).save(OUT / f"{name}_2x.png")


if __name__ == "__main__":
    save("tarot_world_upright", tarot_result(21, False))
    save("tarot_world_keywords", tarot_words(21, False))
    save("iching_result", iching_result())
    save("dice_3d6", dice_3d6())
    save("stick_northeast", stick_eight(1))
    print("wrote", OUT)
