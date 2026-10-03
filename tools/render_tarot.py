"""タロットのカードごとの文字画面を 1bit 画像 `.a1b` にする（B 案・額装）。

カード名とキーワードは caelum-liber-arcanorum（同じ作者）の
`src/data/cards.json`・`src/data/meanings/*.json` から取る（DESIGN.md §7）。

出力（microSD にコピーする。リポジトリには含めない）:
  sd/alea/tarot/cap/NN.a1b   … 結果画面：二重枠・名前の帯（英名・和名）・案内。カード画像と正逆の札は実機で重ねる
  sd/alea/tarot/word/NN.a1b  … キーワード画面：見出し・正位置／逆位置のキーワード各 4 語・案内。今の向きの札は実機で反転する
  tools/out/tarot/*.png      … 確認用

`.a1b`（Alea 1-bit）：16 バイトのヘッダ（"A1B1"・幅・高さ・左上 x・y（いずれも LE の u16）・予約 4 バイト）と、
行ごとにバイト境界で MSB から詰めた画素（1=黒）。

使い方: python tools/render_tarot.py
"""

import json
import struct
from pathlib import Path

import numpy as np
from PIL import Image

from render_apps import (TAROT_CAP_EN_Y, TAROT_CAP_JA_Y, TAROT_WORD_CHIP_Y, TAROT_WORD_LINE0, TAROT_WORD_PITCH,
                         TAROT_WORD_RULE_Y, Sprite, frame, orient_chip, render, rule)
from render_launcher import BLACK, OUT, ROOT, S, font_mincho, font_serif, text_center

CAELUM = Path("g:/dev/caelum-liber-arcanorum/src/data")
DEST = ROOT / "sd" / "alea" / "tarot"
PREVIEW = OUT / "tarot"
SUITS = ["major", "wands", "cups", "swords", "pentacles"]
CONTENT_W = 400  # 内容領域の幅（x 40〜440）


def load_cards():
    cards = json.loads((CAELUM / "cards.json").read_text(encoding="utf-8"))
    meanings = {}
    for suit in SUITS:
        for m in json.loads((CAELUM / "meanings" / f"{suit}.json").read_text(encoding="utf-8")):
            meanings[m["id"]] = m
    assert len(cards) == 78 and len(meanings) == 78
    for c in cards:
        m = meanings[c["id"]]
        c["up"], c["rev"] = m["keywords_upright"], m["keywords_reversed"]
        assert len(c["up"]) == 4 and len(c["rev"]) == 4, c["id"]
    return cards


def width(text: str, font, spacing_em: float) -> float:
    gap = spacing_em * font.size
    return (sum(font.getlength(ch) for ch in text) + gap * (len(text) - 1)) / S


def check(text: str, font, spacing_em: float, where: str):
    w = width(text, font, spacing_em)
    if w > CONTENT_W:
        print(f"warning: {where} 「{text}」が {w:.0f}px で内容領域（{CONTENT_W}px）を超えます")


def caption(c) -> str:
    """結果画面の 1 行目。大アルカナは「XXI · THE WORLD」、小アルカナは英名のみ。"""
    name = c["name_en"].upper()
    return f"{c['roman']} · {name}" if c["arcana"] == "major" else name


def draw_cap(d, c):
    frame(d, None, None, "B　言葉を見る")
    f_en, f_ja = font_serif(22), font_mincho(30)
    check(caption(c), f_en, 0.15, f"{c['id']:02} 英名")
    check(c["name_ja"], f_ja, 0.3, f"{c['id']:02} 和名")
    text_center(d, 240, TAROT_CAP_EN_Y, caption(c), f_en, BLACK, spacing_em=0.15)
    text_center(d, 240, TAROT_CAP_JA_Y, c["name_ja"], f_ja, BLACK, spacing_em=0.3)


def draw_word(d, c):
    major = c["arcana"] == "major"
    frame(d, c["roman"] if major else None, c["name_ja"], "B　絵に戻る")
    text_center(d, 240, 150, c["name_en"].upper(), font_serif(22), BLACK, spacing_em=0.2)
    f = font_mincho(26)
    for cy, label, words in zip(TAROT_WORD_CHIP_Y, ["正位置", "逆位置"], [c["up"], c["rev"]]):
        orient_chip(d, 240, cy, label, False)
        for i, w in enumerate(words):
            check(w, f, 0, f"{c['id']:02} キーワード")
            text_center(d, 240, cy + TAROT_WORD_LINE0 + i * TAROT_WORD_PITCH, w, f, BLACK)
    rule(d, 240, TAROT_WORD_RULE_Y, 260, 9)


def write_a1b(path: Path, sp: Sprite):
    header = b"A1B1" + struct.pack("<HHHH", sp.w, sp.h, sp.x, sp.y) + bytes(4)
    path.write_bytes(header + sp.packed())


def main():
    cards = load_cards()
    for sub in ["cap", "word"]:
        (DEST / sub).mkdir(parents=True, exist_ok=True)
    PREVIEW.mkdir(parents=True, exist_ok=True)
    for c in cards:
        n = f"{c['id']:02}"
        for sub, fn in [("cap", draw_cap), ("word", draw_word)]:
            mask = render(lambda d: fn(d, c))
            write_a1b(DEST / sub / f"{n}.a1b", Sprite(mask))
            if c["id"] in (0, 21, 40, 75):
                Image.fromarray(np.where(mask, 0, 255).astype(np.uint8)).save(PREVIEW / f"{sub}_{n}.png")
    print(f"wrote {len(cards)} cards to", DEST)


if __name__ == "__main__":
    main()
