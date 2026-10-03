"""M5（易・ルーン）の画像部品。`render_apps.py` から呼ばれ、同じ部品の束に加える。

確定デザイン：Claude Design「Alea ランチャー」5 段目（2026-10-03）。
- 易：途中（6 段の欄・爻の名前・硬貨の表裏・変爻の印）と結果（本卦・之卦の卦名・番号と読み・解釈）。
  爻の棒（卦画）は実機で描く。文は tools/data/iching.json
- ルーン：逆位置の札・石と字形（正・逆）・名前・キーワード・文。文と字形は tools/data/runes.json
"""

import json
from pathlib import Path

from render_apps import BLACK, Sprite, frame, orient_chip, render, s, text_center
from render_launcher import font_mincho, font_serif
from render_m4 import KANSUJI, kansuji

DATA = Path(__file__).resolve().parent / "data"

# ---- 易：途中 ----
# 上下の案内は 2 行・1 段大きく・輪郭で太く（2026-10-03 実機）。
IC_NOTE_Y, IC_NOTE_SIZE = (146, 178), 24
IC_TEXT_STROKE = 2
IC_ROW_BOTTOM_Y, IC_ROW_PITCH = 560, 52  # 初爻の中心の高さと行送り（上へ）
IC_LABEL_RIGHT = 82
IC_BAR_X, IC_BAR_W, IC_BAR_H = 96, 220, 18
IC_MARK_CX = 336
IC_COIN_CX = 400
IC_LEGEND_Y, IC_LEGEND_SIZE = (674, 702), 18
LINE_NAMES = ["初爻", "二爻", "三爻", "四爻", "五爻", "上爻"]

# ---- 易：結果 ----
IC_COL_CX = (130, 350)  # 本卦・之卦の列の中心（変爻が無いときは本卦を 240 に置く）
IC_COL_LABEL_Y = 160
IC_FIG_TOP, IC_FIG_W, IC_FIG_BAR, IC_FIG_GAP = 182, 120, 14, 11
IC_NAME_Y, IC_NO_Y, IC_READING_Y = 352, 386, 410
IC_ARROW_Y = IC_NAME_Y  # 矢印は卦名の間（卦画の横だと変爻の印と重なる）
IC_CHANGES_Y, IC_CHANGES_ADV, IC_CHANGES_SIZE = 452, 22, 18
IC_SUM_TOP = (500, 596)  # 解釈の欄の上端（本卦・之卦）
IC_SUM_LABEL_X, IC_SUM_TEXT_X, IC_SUM_SIZE, IC_SUM_LEADING = 40, 100, 22, 33
CHANGE_CHARS = "変爻初二三四五上・なし"
# 18px 以下の明朝は細い横画（三・二）が白黒で消えるため、2 値化の境目を明るい側に寄せる。
SMALL_TEXT_THRESHOLD = 170

# ---- ルーン ----
RN_CHIP_Y, RN_CHIP_W, RN_CHIP_H = 154, 150, 40
RN_STONE_C, RN_STONE_W, RN_STONE_H, RN_STONE_R = (240, 326), 200, 270, 44
RN_GLYPH_BOX = (120, 180)
RN_NAME_Y, RN_KANA_Y, RN_WORDS_Y = 492, 530, 574
RN_TEXT_Y, RN_TEXT_LEADING = 618, 32
RN_KANA_GAP = 12


# ---- 易 ----

def iching_cast_frame(d):
    frame(d, "II", "易", "振って爻を立てる")
    f = font_mincho(18)
    for i, name in enumerate(LINE_NAMES):
        cy = IC_ROW_BOTTOM_Y - i * IC_ROW_PITCH
        w = sum(f.getlength(c) for c in name) / 4
        text_center(d, IC_LABEL_RIGHT - w / 2, cy, name, f, BLACK)
    for y, line in zip(IC_LEGEND_Y, ["表＝三　裏＝二", "〇 老陽（九）　× 老陰（六）"]):
        text_center(d, 240, y, line, font_mincho(IC_LEGEND_SIZE), BLACK, spacing_em=0.05, stroke=IC_TEXT_STROKE)


def note(d, n):
    f = font_mincho(IC_NOTE_SIZE)
    for y, line in zip(IC_NOTE_Y, ["三枚の硬貨を振る", f"{KANSUJI[n]}／六"]):
        text_center(d, 240, y, line, f, BLACK, spacing_em=0.2, stroke=IC_TEXT_STROKE)


def mark(d, cx, cy, kind, r=8):
    if kind == "o":
        d.ellipse([s(cx - r), s(cy - r), s(cx + r), s(cy + r)], outline=BLACK, width=s(2.5))
    else:
        k = r * 0.875
        for a, b in [((cx - k, cy - k), (cx + k, cy + k)), ((cx + k, cy - k), (cx - k, cy + k))]:
            d.line([(s(a[0]), s(a[1])), (s(b[0]), s(b[1]))], fill=BLACK, width=s(2.5))


def iching_result_frame(d):
    frame(d, "II", "易", "振ってもう一度")


# ---- ルーン ----

def rune_chip_rects():
    gap = 6
    x0 = 240 - RN_CHIP_W - gap // 2
    return [(x0 + i * (RN_CHIP_W + gap), RN_CHIP_Y - RN_CHIP_H // 2, RN_CHIP_W, RN_CHIP_H) for i in range(2)]


def rune_frame(d):
    frame(d, "III", "ルーン", "B　逆位置　・　振って引く")
    f = font_mincho(20)
    for (x, y, w, h), lab in zip(rune_chip_rects(), ["正位置のみ", "逆位置あり"]):
        d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], outline=BLACK, width=s(1))
        text_center(d, x + w / 2, y + h / 2, lab, f, BLACK, spacing_em=0.1)
    cx, cy = RN_STONE_C
    d.rounded_rectangle([s(cx - RN_STONE_W / 2), s(cy - RN_STONE_H / 2), s(cx + RN_STONE_W / 2), s(cy + RN_STONE_H / 2)],
                        radius=s(RN_STONE_R), outline=BLACK, width=s(2))


def rune_glyph(d, strokes, rotate):
    cx, cy = RN_STONE_C
    bw, bh = RN_GLYPH_BOX
    w = 11
    for poly in strokes:
        pts = []
        for x, y in poly:
            if rotate:
                x, y = bw - x, bh - y
            pts.append((cx - bw / 2 + x, cy - bh / 2 + y))
        d.line([(s(x), s(y)) for x, y in pts], fill=BLACK, width=s(w), joint="curve")
        for x, y in pts:  # 端と角を丸める
            d.ellipse([s(x - w / 2), s(y - w / 2), s(x + w / 2), s(y + w / 2)], fill=BLACK)


def symmetric(strokes) -> bool:
    bw, bh = RN_GLYPH_BOX
    segs = {frozenset([tuple(a), tuple(b)]) for poly in strokes for a, b in zip(poly, poly[1:])}
    rot = {frozenset((bw - x, bh - y) for x, y in seg) for seg in segs}
    return segs == rot


def text_lines(d, x, top, lines, size, leading, center=False):
    f = font_mincho(size)
    for i, line in enumerate(lines):
        cy = top + size / 2 + i * leading
        if center:
            text_center(d, 240, cy, line, f, BLACK)
        else:
            d.text((s(x), s(cy)), line, font=f, fill=BLACK, anchor="lm")


def build_m5(atlas, out):
    """部品を `atlas` に足し、Rust の定義を `out` に書き足す。"""

    def add(sp: Sprite) -> str:
        atlas.add(sp)
        return sp.rust()

    def const(name, fn, doc, anchor=(0, 0), threshold=128):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: Sprite = {add(Sprite(render(fn, threshold), *anchor))};")

    def array(name, fns, doc, anchor=(0, 0), threshold=128):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: [Sprite; {len(fns)}] = [")
        out.extend(f"    {add(Sprite(render(fn, threshold), *anchor))}," for fn in fns)
        out.append("];")

    def num(name, v, doc, ty="i32"):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: {ty} = {v};")

    # 易・途中。
    const("IC_CAST_FRAME", iching_cast_frame, "易の途中の台紙（爻の名前・凡例を含む）。", threshold=SMALL_TEXT_THRESHOLD)
    array("IC_NOTE", [lambda d, n=n: note(d, n) for n in range(7)], "案内と進み具合（〇／六〜六／六・2 行）。")
    array("IC_COINS", [lambda d, k=k: text_center(d, 240, 400, " ".join("表" if k >> (2 - j) & 1 else "裏" for j in range(3)),
                                                  font_mincho(16), BLACK) for k in range(8)],
          "硬貨 3 枚の表裏（添字の bit2 が 1 枚目・1=表。中心からの相対）。", (240, 400), threshold=SMALL_TEXT_THRESHOLD)
    const("IC_MARK_O", lambda d: mark(d, 240, 400, "o"), "老陽の印〇（中心からの相対）。", (240, 400))
    const("IC_MARK_X", lambda d: mark(d, 240, 400, "x"), "老陰の印×（中心からの相対）。", (240, 400))
    for name, v, doc in [
        ("IC_ROW_BOTTOM_Y", IC_ROW_BOTTOM_Y, "初爻の欄の中心の高さ。"), ("IC_ROW_PITCH", IC_ROW_PITCH, "爻の欄の行送り（上へ）。"),
        ("IC_BAR_X", IC_BAR_X, "途中の爻の棒の左端。"), ("IC_BAR_W", IC_BAR_W, "途中の爻の棒の幅。"), ("IC_BAR_H", IC_BAR_H, "途中の爻の棒の太さ。"),
        ("IC_MARK_CX", IC_MARK_CX, "変爻の印の中心の x。"), ("IC_COIN_CX", IC_COIN_CX, "硬貨の表裏の中心の x。"),
    ]:
        num(name, v, doc)

    # 易・結果。
    hexes = json.loads((DATA / "iching.json").read_text(encoding="utf-8"))["hexagrams"]
    assert [h["no"] for h in hexes] == list(range(1, 65))
    const("IC_RESULT_FRAME", iching_result_frame, "易の結果の台紙。")
    const("IC_LABEL_PRIMARY", lambda d: text_center(d, 240, IC_COL_LABEL_Y, "本卦", font_mincho(18), BLACK, spacing_em=0.3),
          "「本卦」（列の中心 x からの相対）。", (240, 0), threshold=SMALL_TEXT_THRESHOLD)
    const("IC_LABEL_CHANGED", lambda d: text_center(d, 240, IC_COL_LABEL_Y, "之卦", font_mincho(18), BLACK, spacing_em=0.3),
          "「之卦」（列の中心 x からの相対）。", (240, 0), threshold=SMALL_TEXT_THRESHOLD)
    const("IC_ARROW", lambda d: text_center(d, 240, IC_ARROW_Y, "→", font_serif(30), BLACK), "本卦から之卦への矢印。")
    array("IC_NAME", [lambda d, h=h: text_center(d, 240, IC_NAME_Y, h["name"], font_mincho(30), BLACK, spacing_em=0.1)
                      for h in hexes], "卦名（文王の順・添字 0 が第一卦。列の中心 x からの相対）。", (240, 0))
    array("IC_NUMBER_READING", [lambda d, h=h: (text_center(d, 240, IC_NO_Y, f"第{kansuji(h['no'])}卦", font_mincho(16), BLACK,
                                                            spacing_em=0.1),
                                                text_center(d, 240, IC_READING_Y, h["reading"], font_mincho(16), BLACK,
                                                            spacing_em=0.05))
                                for h in hexes], "番号と読み（列の中心 x からの相対）。", (240, 0), threshold=SMALL_TEXT_THRESHOLD)
    array("IC_SUMMARY", [lambda d, h=h: text_lines(d, IC_SUM_TEXT_X, 400, h["summary"], IC_SUM_SIZE, IC_SUM_LEADING)
                         for h in hexes], "解釈（2 行・欄の上端からの相対）。", (0, 400))
    const("IC_SUM_PRIMARY", lambda d: orient_chip_small(d, "本卦"), "解釈の欄の見出し「本卦」（欄の上端からの相対）。", (0, 400), threshold=SMALL_TEXT_THRESHOLD)
    const("IC_SUM_CHANGED", lambda d: orient_chip_small(d, "之卦"), "解釈の欄の見出し「之卦」（欄の上端からの相対）。", (0, 400), threshold=SMALL_TEXT_THRESHOLD)
    out.append("/// 変爻の行に使う字（`IC_CHANGE_GLYPHS` と同じ順）。")
    out.append(f"pub const IC_CHANGE_CHARS: [char; {len(CHANGE_CHARS)}] = [" + ", ".join(f"'{c}'" for c in CHANGE_CHARS) + "];")
    array("IC_CHANGE_GLYPHS", [lambda d, c=c: text_center(d, 240, 400, c, font_mincho(IC_CHANGES_SIZE), BLACK) for c in CHANGE_CHARS],
          "変爻の行の字（1 字ずつ・中心からの相対）。", (240, 400), threshold=SMALL_TEXT_THRESHOLD)
    for name, v, doc in [
        ("IC_COL_CX", list(IC_COL_CX), "本卦・之卦の列の中心 x。"),
        ("IC_FIG_TOP", IC_FIG_TOP, "卦画の上端。"), ("IC_FIG_W", IC_FIG_W, "卦画の幅。"), ("IC_FIG_BAR", IC_FIG_BAR, "卦画の爻の太さ。"),
        ("IC_FIG_GAP", IC_FIG_GAP, "卦画の爻の間隔。"), ("IC_CHANGES_Y", IC_CHANGES_Y, "変爻の行の中心の高さ。"),
        ("IC_CHANGES_ADV", IC_CHANGES_ADV, "変爻の行の字送り。"), ("IC_SUM_TOP", list(IC_SUM_TOP), "解釈の欄の上端 [本卦, 之卦]。"),
    ]:
        num(name, v, doc, "[i32; 2]" if isinstance(v, list) else "i32")

    # ルーン。
    runes = json.loads((DATA / "runes.json").read_text(encoding="utf-8"))["runes"]
    assert len(runes) == 24
    sym = [symmetric(r["strokes"]) for r in runes]
    for r, sy in zip(runes, sym):
        assert sy == ("rev" not in r), r["name"]
    const("RUNE_FRAME", rune_frame, "ルーンの台紙（逆位置の札は未選択の姿・石の輪郭を含む）。")
    num("RUNE_CHIPS", rune_chip_rects(), "逆位置の札の矩形 (x, y, w, h)。[正位置のみ, 逆位置あり]。", "[(i32, i32, i32, i32); 2]")
    num("RUNE_SYMMETRIC", [str(b).lower() for b in sym], "点対称の字形か（逆位置にならない）。", "[bool; 24]")
    out[-1] = out[-1].replace("'", "")
    array("RUNE_GLYPH", [lambda d, r=r: rune_glyph(d, r["strokes"], False) for r in runes], "字形（正位置）。")
    array("RUNE_GLYPH_REV", [lambda d, r=r: rune_glyph(d, r["strokes"], True) for r in runes], "字形（逆位置・180° 回転）。")
    const("RUNE_WAIT", lambda d: text_center(d, RN_STONE_C[0], RN_STONE_C[1], "?", font_serif(80), BLACK), "待機中の「?」。")
    array("RUNE_NAME", [lambda d, r=r: text_center(d, 240, RN_NAME_Y, r["name"], font_serif(34), BLACK, spacing_em=0.3)
                        for r in runes], "名前（英字）。")
    array("RUNE_KANA", [lambda d, r=r: text_center(d, 240, 400, r["kana"], font_mincho(20), BLACK, spacing_em=0.2)
                        for r in runes], "名前（カタカナ・中心からの相対）。", (240, 400))
    const("RUNE_REV_CHIP", lambda d: orient_chip(d, 240, 400, "逆位置", True), "「逆位置」の札（中心からの相対）。", (240, 400))
    for key, label in [("up", "UP"), ("rev", "REV")]:
        array(f"RUNE_WORDS_{label}", [lambda d, r=r: text_center(d, 240, RN_WORDS_Y, r.get(key, r["up"])["words"], font_mincho(26),
                                                                  BLACK, spacing_em=0.1) for r in runes],
              f"キーワード（{'正位置' if key == 'up' else '逆位置・点対称の文字は正位置と同じ'}）。")
        array(f"RUNE_TEXT_{label}", [lambda d, r=r: text_lines(d, 0, RN_TEXT_Y - 10, r.get(key, r["up"])["text"], 20, RN_TEXT_LEADING,
                                                               center=True) for r in runes],
              f"文（{'正位置' if key == 'up' else '逆位置'}・2 行）。")
    num("RUNE_KANA_Y", RN_KANA_Y, "カタカナ名の中心の高さ。")
    num("RUNE_KANA_GAP", RN_KANA_GAP, "カタカナ名と「逆位置」の札の間隔。")
    out.append("/// 名前（ログ用）。")
    out.append("pub const RUNE_NAMES: [&str; 24] = [" + ", ".join(f'"{r["name"]}"' for r in runes) + "];")


def orient_chip_small(d, label):
    """解釈の欄の見出し（枠付き・16px）。欄の上端を y=400 として描く。"""
    f = font_mincho(16)
    w = sum(f.getlength(c) for c in label) / 4 + 12
    x, y, h = IC_SUM_LABEL_X, 400 + 4, 24
    d.rectangle([s(x), s(y), s(x + w), s(y + h)], outline=BLACK, width=s(1))
    text_center(d, x + w / 2, y + h / 2, label, f, BLACK)


