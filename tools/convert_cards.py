"""タロットの絵柄を Alea の 4 階調画像 `.a2b` に変換する（DESIGN.md §8）。

処理：グレースケール → 480×720 に LANCZOS で縮小（2:3 でなければ中央を切り抜く）→
autocontrast(cutoff) → コントラスト強調 → Atkinson ディザリングで 4 階調 → `.a2b`。

入力（既定）:
  g:/dev/caelum-liber-arcanorum/tools/tarot-gen/final/NN_<name>_<variant>.png  … 78 枚（絵柄のみ）
  g:/dev/caelum-liber-arcanorum/src/assets/cards/full/back.webp               … 裏面
出力:
  sd/alea/tarot/img/NN.a2b・back.a2b   … microSD にコピーする画像
  tools/out/cards/NN.png・back.png     … 確認用（4 階調に丸めた後の見た目）
  tools/out/cards_sheet.png            … 全カードの縮小一覧

絵柄は caelum-liber-arcanorum の素材（© osprey74 All rights reserved）。出力はリポジトリに含めない（.gitignore 済み）。

使い方:
  python tools/convert_cards.py                      # 全部
  python tools/convert_cards.py --only 21 back       # 一部だけ
  python tools/convert_cards.py --levels 80,175 --contrast 1.2   # 実機の濃度合わせ（D-2）
"""

import argparse
import re
import struct
from pathlib import Path

import numpy as np
from PIL import Image, ImageEnhance, ImageOps

ROOT = Path(__file__).resolve().parent.parent
SRC_FINAL = Path("g:/dev/caelum-liber-arcanorum/tools/tarot-gen/final")
SRC_BACK = Path("g:/dev/caelum-liber-arcanorum/src/assets/cards/full/back.webp")
DEST = ROOT / "sd" / "alea" / "tarot" / "img"
OUT = ROOT / "tools" / "out"

W, H = 480, 720
MAGIC = b"A2B1"


def prepare(path: Path, cutoff: float, contrast: float) -> np.ndarray:
    """読み込み・切り抜き・縮小・コントラスト調整。0〜255 の float 配列を返す。"""
    img = Image.open(path).convert("L")
    img = ImageOps.fit(img, (W, H), Image.LANCZOS)  # 2:3 ならそのまま縮小
    img = ImageOps.autocontrast(img, cutoff=cutoff)
    img = ImageEnhance.Contrast(img).enhance(contrast)
    return np.asarray(img, dtype=np.float32)


def atkinson(gray: np.ndarray, points: list[float]) -> np.ndarray:
    """Atkinson ディザリング。`points` は階調 0〜3 の輝度。戻り値は階調（0=黒〜3=白）。

    誤差の 6/8 だけを右 2・下段 3・2 段下 1 の 6 画素へ 1/8 ずつ配る（残りは捨てるので、白と黒が締まる）。
    """
    h, w = gray.shape
    buf = np.zeros((h + 2, w + 4), dtype=np.float32)  # 右に 2・左右に余白
    buf[:h, 2:w + 2] = gray
    out = np.zeros((h, w), dtype=np.uint8)
    rows = buf.tolist()
    for y in range(h):
        r0, r1, r2 = rows[y], rows[y + 1], rows[y + 2]
        orow = out[y]
        for x in range(2, w + 2):
            v = r0[x]
            # 最も近い階調。
            k = 0
            best = abs(v - points[0])
            for i in range(1, 4):
                d = abs(v - points[i])
                if d < best:
                    best, k = d, i
            orow[x - 2] = k
            e = (v - points[k]) / 8.0
            r0[x + 1] += e
            r0[x + 2] += e
            r1[x - 1] += e
            r1[x] += e
            r1[x + 1] += e
            r2[x] += e
    return out


def write_a2b(path: Path, levels: np.ndarray):
    h, w = levels.shape
    flat = levels.reshape(-1, 4)
    packed = (flat[:, 0] << 6) | (flat[:, 1] << 4) | (flat[:, 2] << 2) | flat[:, 3]
    header = MAGIC + struct.pack("<HH", w, h) + bytes(8)
    path.write_bytes(header + packed.astype(np.uint8).tobytes())


def preview(levels: np.ndarray, points: list[float]) -> Image.Image:
    lut = np.asarray([round(p) for p in points], dtype=np.uint8)
    return Image.fromarray(lut[levels])


def sources() -> dict[str, Path]:
    found = {}
    for p in sorted(SRC_FINAL.glob("*.png")):
        m = re.match(r"(\d\d)_", p.name)
        if m:
            found[m.group(1)] = p
    found["back"] = SRC_BACK
    return found


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--only", nargs="*", help="変換するカード（00〜77・back）")
    ap.add_argument("--levels", default="85,170", help="暗灰・明灰の輝度（既定 85,170）")
    ap.add_argument("--contrast", type=float, default=1.15, help="コントラスト係数（既定 1.15）")
    ap.add_argument("--cutoff", type=float, default=1.0, help="autocontrast の cutoff [%%]（既定 1）")
    args = ap.parse_args()

    dark, light = (float(v) for v in args.levels.split(","))
    points = [0.0, dark, light, 255.0]
    src = sources()
    if len([k for k in src if k != "back"]) != 78:
        print(f"warning: 絵柄が 78 枚ではありません（{len(src) - 1} 枚）")
    keys = args.only or list(src)
    DEST.mkdir(parents=True, exist_ok=True)
    (OUT / "cards").mkdir(parents=True, exist_ok=True)

    for key in keys:
        if key not in src:
            print(f"skip {key}: 入力がありません")
            continue
        levels = atkinson(prepare(src[key], args.cutoff, args.contrast), points)
        write_a2b(DEST / f"{key}.a2b", levels)
        preview(levels, points).save(OUT / "cards" / f"{key}.png")
        counts = np.bincount(levels.ravel(), minlength=4) / levels.size * 100
        print(f"{key}: {src[key].name}  black/dark/light/white = " + " / ".join(f"{c:.0f}%" for c in counts))

    # 一覧（10 列・1/4 縮小）。
    names = [k for k in list(src) if (OUT / "cards" / f"{k}.png").exists()]
    tw, th, cols = W // 4, H // 4, 10
    sheet = Image.new("L", (cols * (tw + 4), ((len(names) + cols - 1) // cols) * (th + 4)), 128)
    for i, k in enumerate(names):
        im = Image.open(OUT / "cards" / f"{k}.png").resize((tw, th), Image.LANCZOS)
        sheet.paste(im, ((i % cols) * (tw + 4), (i // cols) * (th + 4)))
    sheet.save(OUT / "cards_sheet.png")
    print("wrote", DEST, "and", OUT / "cards_sheet.png")


if __name__ == "__main__":
    main()
