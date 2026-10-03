# Alea (paper-alea)

M5Stack PaperMono 向けの「偶然」ミニアプリ集です。本体を振ると、タロット・易・ルーンの占いや、ダイス・コイントスなどの結果が e-paper に表示されます。

A pocket collection of "chance" apps for the M5Stack PaperMono. Shake the device to draw a tarot card, cast the I Ching, roll dice, flip a coin and more.

> ⚠️ 開発中です（9 本のアプリはすべて動作します。仕上げの段階）。 / Work in progress (all nine apps work; finishing touches).
>
> 本プロジェクトは M5Stack 社の公式製品ではありません。 / This is not an official M5Stack product.

## 収録アプリ / Apps

| アプリ | 内容 |
|---|---|
| タロット / Tarot | 大アルカナ・小アルカナ 78 枚から 1 枚（ワンオラクル・正逆あり。microSD にカード画像が必要） |
| 易 / I Ching | 三枚硬貨法で 6 爻を立て、本卦・之卦を表示 |
| ルーン / Runes | エルダー・フサルク 24 文字から 1 つ（ルーン文字を用いた現代の占い。逆位置は B で切替） |
| ダイス / Dice | 1D3〜1D100 の 10 種 |
| コイントス / Coin | 表・裏 |
| 棒倒し / Stick | 左右または 8 方位 |
| あみだくじ / Amida | 2〜6 本 |
| おみくじ / Omikuji | 運勢と一言 |
| Yes / No | YES か NO |

## 使い方 / Usage

- ランチャーでアプリのタイルをタップして開きます。ボタン A でいつでもランチャーに戻ります。
- 本体を振ると結果が出ます（あみだくじは上の番号をタップ）。ボタン B はアプリごとの切替（ダイスの種類はチップをタップ、棒倒しの方式、あみだくじの本数、ルーンの逆位置、タロットのキーワード表示）です。
- 電源ボタンを 1 回押すと表紙を表示して電源が切れ、もう一度押すと起動します（USB 接続中は切れずに待機し、もう一度押すと戻ります）。振っている間は緑の LED が点きます。
- Open an app by tapping its tile. Button A returns to the launcher. Shake the device to get a result. Button B switches app-specific options. Press the power button once to power off (shows a cover screen) and again to power on.

## 対応機種 / Hardware

- M5Stack PaperMono（C153）
- M5Stack PaperMono-Lite（C153-Lite）でも動く設計としています（LoRa・NFC は使いません）

## ビルド / Build

Rust（embassy）で書いています。BSP として [papermono-rs](https://github.com/canardleteer/papermono-rs) を使うため、このリポジトリと同じ階層に clone しておく必要があります。

```text
dev/
├─ paper-alea/
└─ papermono-rs/
```

```powershell
# ロジックのテスト
cargo test

# ファーム（espup で esp toolchain を導入済みであること）
cd firmware/alea-fw
cargo +esp build --release
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/alea-fw
```

microSD に入れるデータは [sd/alea/README.txt](sd/alea/README.txt) を参照してください。microSD はタロットだけが使います（カード画像は `tools/convert_cards.py`・`tools/render_tarot.py` で生成します）。ほかのアプリは microSD が無くても動きます。

`sd/alea/config.json` でシェイク検出の感度を調整できます（DESIGN.md §6.3）。

## ライセンス / License

- プログラムと文書：[MIT License](LICENSE)
- タロットカードの絵柄はこのリポジトリに含まれていません。絵柄は [Liber Arcanorum](https://github.com/osprey74/caelum-liber-arcanorum) の素材（© osprey74 All rights reserved）です。
  The tarot card art is not included in this repository.

## 謝辞 / Acknowledgements

- [canardleteer/papermono-rs](https://github.com/canardleteer/papermono-rs)（MIT）— PaperMono の BSP
