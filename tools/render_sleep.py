"""終了画面（電源オフ・USB 接続中の待機）を 480×800・4 階調の画像にする。

ランチャー（B 案・書物調）と同じ二重枠・題字「ALEA」・罫と菱形・副題「骰子と偶然」を大きく中央に置き、
下に案内を添える。電源を切った後も e-paper に残る「表紙」になる（2026-10-03 決定）。

出力:
  firmware/alea-fw/assets/sleep_off.2bpp  … 電源オフ（電池）「電源ボタンを押すと起動します」
  firmware/alea-fw/assets/sleep_usb.2bpp  … USB 接続中の待機「USB 接続中は電源を切れません／電源ボタンで戻ります」
  tools/out/sleep_*.png                   … 確認用

使い方: python tools/render_sleep.py
"""

import numpy as np
from PIL import Image, ImageDraw

from render_launcher import ASSETS, BLACK, DARK, OUT, WHITE, H, S, W, font_mincho, font_serif, s, text_center


def cover(lines):
    img = Image.new("L", (W * S, H * S), WHITE)
    d = ImageDraw.Draw(img)
    d.rectangle([s(14), s(14), s(466) - 1, s(786) - 1], outline=DARK, width=s(2))
    d.rectangle([s(21), s(21), s(459) - 1, s(779) - 1], outline=BLACK, width=s(1))
    text_center(d, 240, 330, "ALEA", font_serif(80), BLACK, spacing_em=0.32)
    y = 392
    d.line([s(90), s(y), s(228), s(y)], fill=BLACK, width=s(1))
    d.line([s(252), s(y), s(390), s(y)], fill=BLACK, width=s(1))
    r = 7
    d.polygon([(s(240), s(y - r)), (s(240 + r), s(y)), (s(240), s(y + r)), (s(240 - r), s(y))], fill=BLACK)
    text_center(d, 240, 432, "骰子と偶然", font_mincho(24), DARK, spacing_em=0.4)
    # 下の案内（ランチャーの最下部の案内と同じ 20px）。
    d.line([s(40), s(690), s(440), s(690)], fill=BLACK, width=s(1))
    for i, line in enumerate(lines):
        cy = 735 - (len(lines) - 1) * 15 + i * 30
        text_center(d, 240, cy, line, font_mincho(20), BLACK, spacing_em=0.15, stroke=1)
    small = img.resize((W, H), Image.LANCZOS)
    return np.clip(np.rint(np.asarray(small, dtype=np.float32) / 85.0), 0, 3).astype(np.uint8)


def write(name, levels):
    flat = levels.reshape(-1, 4)
    packed = (flat[:, 0] << 6) | (flat[:, 1] << 4) | (flat[:, 2] << 2) | flat[:, 3]
    (ASSETS / f"sleep_{name}.2bpp").write_bytes(packed.astype(np.uint8).tobytes())
    Image.fromarray((levels * 85).astype(np.uint8)).save(OUT / f"sleep_{name}.png")


if __name__ == "__main__":
    OUT.mkdir(parents=True, exist_ok=True)
    write("off", cover(["電源ボタンを押すと起動します"]))
    write("usb", cover(["USB 接続中は電源を切れません", "電源ボタンで戻ります"]))
    print("wrote sleep_off / sleep_usb")
