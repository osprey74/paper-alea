# CLAUDE.md — Alea（paper-alea）

> M5Stack PaperMono 向け「偶然」ミニアプリ集（タロット・易・ルーン・ダイス・コイン等）

## プロジェクト概要

*Alea*（ラテン語「骰子・偶然」）は、本体を振って結果を決めるポケットサイズの占い・偶然アプリ集。
ハードウェアの知見とドライバは **Nostos**（`g:\dev\Nostos`）から、タロットの絵柄は
**caelum-liber-arcanorum**（`g:\dev\caelum-liber-arcanorum`）から流用する。

- 仕様：[DESIGN.md](DESIGN.md)（**仕様の正**）
- 現行マイルストーンの作業指示：[HANDOFF.md](HANDOFF.md)（M1：土台とランチャー）

## ターゲット

- **デバイス**：M5Stack PaperMono（`C153`）。**PaperMono-Lite（`C153-Lite`）でも動く範囲に留める**
- **SoC**：ESP32-S3R8 / 16MB Flash / 8MB PSRAM
- **画面**：3.97" 480×800 4階調タッチ e-ink（SSD1677）。縦長（ポートレート）で使う

## 技術スタック

- **Rust / embassy（no_std）**。構成と依存の版は Nostos `firmware/nostos-fw` に揃える
- BSP：[`canardleteer/papermono-rs`](https://github.com/canardleteer/papermono-rs)（MIT）を path 依存
  （`g:\dev\papermono-rs`）。**`m5stack-papermono-lite` のみ使用**（`m5stack-papermono` は LoRa/NFC を含むため使わない）
- `crates/alea-core`：ハード非依存ロジック（no_std）。ホストで `cargo test`
- `firmware/alea-fw`：Xtensa ファーム（独立ワークスペース・親ワークスペースからは exclude）
- `tools/`：Python（Pillow + NumPy）の画像変換ツール

## ビルド・テスト（Windows）

```powershell
# ロジックのテスト（リポジトリ直下）
cargo test

# ファーム
. C:\Users\ospre\export-esp.ps1
cd g:\dev\paper-alea\firmware\alea-fw
cargo +esp build --release
espflash flash --port COM8 --monitor target\xtensa-esp32s3-none-elf\release\alea-fw
```

- シリアル観測は `espflash flash --monitor` で行う。素の SerialPort で COM8 を開くと USB-Serial-JTAG が
  download モードに落ちる（Nostos `firmware/nostos-fw/experiments/README.md`）。
- リンカの「LOAD segment with RWX permissions」警告はツールチェーン由来（Nostos でも同じ）。

## 厳守事項（パネル保護・互換性）

| # | ルール |
|---|---|
| R-1 | 部分リフレッシュ **10回ごとに全画面リフレッシュを1回**。Display 層で強制し、アプリからはバイパスできないようにする |
| R-2 | 部分高速リフレッシュをループで連続実行しない。描画はイベント起点のみとし、ループ内で定期描画しない |
| R-3 | カスタム波形（LUT 0x32）を使わない（工場 OTP 波形のみ） |
| R-4 | M5GFX 系のカスタム LUT 駆動（`epd_fastest` などの差分高速モード）は実装しない |
| R-5 | LoRa / NFC は初期化しない（PaperMono-Lite 互換）。C153 で LoRa 電源（M5PM1 の LoRa_EN）が既定で ON の場合は OFF にする |
| R-6 | ブザーは鳴らさない |

- PaperMono 実機に書き込む前に papermono-rs の `docs/SAFETY.md` を確認する（IP2315 による I2C 固着、PHY キャリブレーション消失など）。
- **PM1 への I2C 書き込みはバス整定（500ms）後**に行う。PM1 の `PWR_CFG` の BOOST bit は触らない（Nostos で表示系が落ちた実績あり）。
- コールドブートでは M5IOE1 の電源系ピンを読み戻して確認する（Nostos `ioe.rs` の `set_output_verified()`）。

## 作業ルール

- 仕様の正は `DESIGN.md`。仕様を変更したら DESIGN.md も更新し、§14 変更履歴に1行追記する。HANDOFF.md と食い違う場合は DESIGN.md を正とし、食い違いを報告する。
- アプリは `core/` の公開 API だけを使い、HAL（`board/`・esp-hal・papermono-rs）を直接呼ばない。
- 新しいアプリは `firmware/alea-fw/src/apps/<id>.rs`（大きくなれば `apps/<id>/`）に置き、`App` トレイトを実装して `main.rs` で登録する。
- 乱数・抽選・判定ロジックは可能な限り `crates/alea-core` に置き、ホストでテストする（10万回試行の分布テスト）。
- ログの接頭辞は `[Board] [Display] [Input] [Storage] [AppMgr] [<AppId>]` に統一する。
- Nostos・papermono-rs から流用したコードは、ファイル冒頭のコメントに出典（MIT）を書く。
- 実機でしか確認できない事項は、推測で「完了」にせず、確認手順を書いて総司さんに依頼する。
- 解釈文（タロット・易・ルーン・おみくじ）は既存書籍・サイトから流用せず、オリジナルの文で書く。

## カード画像の扱い（重要）

- タロットの絵柄は caelum-liber-arcanorum の素材（`LICENSE-ASSETS.md`：© osprey74 All rights reserved）。
  **このリポジトリは公開のため、カード画像・変換結果（`.a2b`・プレビュー PNG・一覧画像）はコミットしない**（`.gitignore` 済み）。
- 変換の入力：`g:\dev\caelum-liber-arcanorum\tools\tarot-gen\final\NN_<name>_<variant>.png`（1024×1536・絵柄のみ）。
  裏面：`src/assets/cards/full/back.webp`。
- 手元の参照画像は `local/`（gitignore 済み）に置く。

## 共有スキル用の設定

- **タスクファイル**：未作成（必要になったら `TASKS.md` を作り、ここに記載する）
- **バージョンファイル**：`firmware/alea-fw/Cargo.toml`（`version`）
- **CI/CD**：なし（実機書き込みは手動）
- **SNS**：未定
