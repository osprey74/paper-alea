"""設定画面の画像部品。`render_apps.py` から呼ばれ、同じ部品の束に加える。

確定デザイン：Claude Design「Alea ランチャー」6 段目（2026-10-03）。ランチャーでボタン B を押すと開く。
台紙（見出し「ALEA／設定」・節の見出しと罫・札の枠・開発用のボタン・最下部「B 戻る」「Alea vX.Y.Z」）と、
電池の表示に使う字形（電圧の数字・「V」・「約」・残量の数字と「%」・「USB 給電中」）を作る。
選択中の札は実機で内側を反転する。
"""

import re
from pathlib import Path

from render_apps import BLACK, S, Sprite, render, rule, s, text_center
from render_launcher import font_mincho, font_serif

ROOT = Path(__file__).resolve().parent.parent

CX0, CX1 = 40, 440
SECTION_Y = {"light": 160, "auto": 274, "battery": 388, "dev": 476}
LIGHT_TOP, AUTO_TOP, DEV_TOP, CHIP_H, GAP = 178, 292, 494, 64, 6
BATTERY_Y = 432
LIGHT_LABELS = ["消灯", "弱", "強"]
AUTO_LABELS = ["しない", "1 分", "3 分", "5 分", "10 分"]
DEV_LABELS = ["Refresh test", "SD check"]


def version() -> str:
    text = (ROOT / "firmware" / "alea-fw" / "Cargo.toml").read_text(encoding="utf-8")
    return re.search(r'(?m)^version = "([^"]+)"', text).group(1)


def row_rects(top, n):
    w = (CX1 - CX0 - (n - 1) * GAP) / n
    rects = []
    for i in range(n):
        x0 = round(CX0 + i * (w + GAP))
        x1 = round(CX0 + i * (w + GAP) + w)
        rects.append((x0, top, x1 - x0, CHIP_H))
    return rects


def section(d, y, label):
    f = font_mincho(20)
    gap = 0.2 * f.size
    w = (sum(f.getlength(c) for c in label) + gap * (len(label) - 1)) / S
    x = CX0
    for ch in label:
        d.text((s(x), s(y)), ch, font=f, fill=BLACK, anchor="lm")
        x += f.getlength(ch) / S + gap / S
    d.line([(s(CX0 + w + 10), s(y)), (s(CX1), s(y))], fill=BLACK, width=s(1))


def boxes(d, rects, labels, font, spacing):
    for (x, y, w, h), lab in zip(rects, labels):
        d.rectangle([s(x), s(y), s(x + w) - 1, s(y + h) - 1], outline=BLACK, width=s(1))
        text_center(d, x + w / 2, y + h / 2, lab, font, BLACK, spacing_em=spacing)


def settings_frame(d):
    d.rectangle([s(14), s(14), s(466) - 1, s(786) - 1], outline=BLACK, width=s(2))
    d.rectangle([s(21), s(21), s(459) - 1, s(779) - 1], outline=BLACK, width=s(1))
    text_center(d, 240, 45, "ALEA", font_serif(24), BLACK, spacing_em=0.2)
    text_center(d, 240, 83, "設定", font_mincho(30), BLACK, spacing_em=0.3)
    rule(d, 240, 114, 220, 7)
    for key, label in [("light", "バックライト"), ("auto", "自動電源オフ"), ("battery", "電池"), ("dev", "開発用")]:
        section(d, SECTION_Y[key], label)
    boxes(d, row_rects(LIGHT_TOP, 3), LIGHT_LABELS, font_mincho(22), 0.1)
    boxes(d, row_rects(AUTO_TOP, 5), AUTO_LABELS, font_mincho(20), 0.05)
    boxes(d, row_rects(DEV_TOP, 2), DEV_LABELS, font_serif(22), 0.08)
    d.line([s(CX0), s(730), s(CX1), s(730)], fill=BLACK, width=s(1))
    f = font_mincho(20)
    d.text((s(CX0), s(754)), "B　戻る", font=f, fill=BLACK, anchor="lm")
    v = f"Alea v{version()}"
    fs = font_serif(20)
    w = sum(fs.getlength(c) for c in v) / S + 0.1 * 20 * (len(v) - 1)
    text_center(d, CX1 - w / 2, 754, v, fs, BLACK, spacing_em=0.1)


def glyph(ch, font):
    px, cy = 100, 400
    m = render(lambda d: d.text((s(px), s(cy)), ch, font=font, fill=BLACK, anchor="lm"))
    return Sprite(m, px, cy), round(font.getlength(ch) / S * 16)


def build_m6(atlas, out):
    def add(sp):
        atlas.add(sp)
        return sp.rust()

    def const(name, fn, doc, anchor=(0, 0)):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: Sprite = {add(Sprite(render(fn), *anchor))};")

    def num(name, v, doc, ty="i32"):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: {ty} = {v};")

    def font(name, chars, f, doc):
        out.append(f"/// {doc}")
        out.append(f"pub const {name}: [Glyph; {len(chars)}] = [")
        for ch in chars:
            sp, adv = glyph(ch, f)
            out.append(f"    Glyph {{ ch: b'{ch}', adv: {adv}, sprite: {add(sp)} }},")
        out.append("];")

    const("SETTINGS_FRAME", settings_frame, f"設定画面の台紙（札はすべて未選択の姿・最下部に Alea v{version()}）。")
    num("SETTINGS_LIGHT", row_rects(LIGHT_TOP, 3), "バックライトの札の矩形 [消灯, 弱, 強]。", "[(i32, i32, i32, i32); 3]")
    num("SETTINGS_AUTO", row_rects(AUTO_TOP, 5), "自動電源オフの札の矩形（`AUTO_OFF_CHOICES` の順）。", "[(i32, i32, i32, i32); 5]")
    num("SETTINGS_DEV", row_rects(DEV_TOP, 2), "開発用のボタンの矩形 [Refresh test, SD check]。", "[(i32, i32, i32, i32); 2]")
    num("SETTINGS_BATTERY_Y", BATTERY_Y, "電池の行の中心の高さ。")
    font("FONT_BATT_V", "0123456789.", font_serif(34), "電池電圧の数字（34px）。")
    const("BATT_V", lambda d: text_center(d, 240, 400, "V", font_serif(34), BLACK), "電圧の単位「V」（中心からの相対）。", (240, 400))
    font("FONT_BATT_PCT", "0123456789%", font_mincho(22), "残量の数字と「%」（22px）。")
    const("BATT_ABOUT", lambda d: text_center(d, 240, 400, "約", font_mincho(22), BLACK), "「約」（中心からの相対）。", (240, 400))
    const("BATT_USB", lambda d: text_center(d, 240, 400, "USB 給電中", font_mincho(22), BLACK, spacing_em=0.05),
          "「USB 給電中」（中心からの相対）。", (240, 400))
