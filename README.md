# Alea (paper-alea)

M5Stack PaperMono 向けの「偶然」ミニアプリ集です。本体を振ると、タロット・易・ルーンの占いや、ダイス・コイントスなどの結果が e-paper に表示されます。

A pocket collection of "chance" apps for the M5Stack PaperMono. Shake the device to draw a tarot card, cast the I Ching, roll dice, flip a coin and more.

> ⚠️ 開発中（設計段階・M1 着手前）です。 / Work in progress.
>
> 本プロジェクトは M5Stack 社の公式製品ではありません。 / This is not an official M5Stack product.

## 収録アプリ / Apps

| アプリ | 内容 |
|---|---|
| タロット / Tarot | 大アルカナ・小アルカナ 78 枚から 1 枚（ワンオラクル・正逆あり） |
| 易 / I Ching | 三枚硬貨法で 6 爻を立て、本卦・之卦を表示 |
| ルーン / Runes | エルダー・フサルク 24 文字から 1 つ |
| ダイス / Dice | 1D3〜1D100 の 10 種 |
| コイントス / Coin | 表・裏 |
| 棒倒し / Stick | 左右または 8 方位 |
| あみだくじ / Amida | 2〜8 本 |
| おみくじ / Omikuji | 運勢と一言 |
| Yes / No | YES か NO |

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

microSD に入れるデータは [sd/alea/README.txt](sd/alea/README.txt) を参照してください。

## ライセンス / License

- プログラムと文書：[MIT License](LICENSE)
- タロットカードの絵柄はこのリポジトリに含まれていません。絵柄は [Liber Arcanorum](https://github.com/osprey74/caelum-liber-arcanorum) の素材（© osprey74 All rights reserved）です。
  The tarot card art is not included in this repository.

## 謝辞 / Acknowledgements

- [canardleteer/papermono-rs](https://github.com/canardleteer/papermono-rs)（MIT）— PaperMono の BSP
