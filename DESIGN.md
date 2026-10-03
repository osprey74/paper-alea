# DESIGN.md — Alea（paperAlea）

> M5Stack PaperMono 向け「偶然」ミニアプリ集
> 作成日：2026-10-02 ／ 更新日：2026-10-03 ／ ステータス：M1 完了（v0.3）

---

## 1. 概要

| 項目 | 内容 |
|---|---|
| 作品名（画面表記） | **Alea**（ラテン語「骰子・偶然」） |
| プロジェクト／リポジトリ名 | `paper-alea`（表記は `paperAlea`）／ https://github.com/osprey74/paper-alea |
| 対象機種 | M5Stack PaperMono（C153）／PaperMono-Lite（C153-Lite）でも動作可能な範囲に留める |
| コンセプト | タロット・易・ルーンなどの占いと、ダイス・コインなどの日常の偶然を1台で切り替えて実行する |
| 基本操作 | **本体を振る**ことで結果を決める。選択系の操作はタッチとボタンで行う |
| 対象外 | 効果音（SE）とアニメーションは実装しない |

### 収録ミニアプリ（全10本）

| # | ID | 名称 | 区分 | 確定トリガー |
|---|---|---|---|---|
| 1 | `tarot` | タロット（ワンオラクル） | 占い | シェイク |
| 2 | `iching` | 易 | 占い | シェイク×6回 |
| 3 | `rune` | ルーン | 占い | シェイク |
| 4 | `dice` | ダイス（10種） | 偶然 | シェイク |
| 5 | `coin` | コイントス | 偶然 | シェイク |
| 6 | `stick` | 棒倒し | 偶然 | シェイク |
| 7 | `amida` | あみだくじ | 偶然 | タッチ |
| 8 | `omikuji` | おみくじ | 偶然 | シェイク |
| 9 | `yesno` | Yes / No | 偶然 | シェイク |
| 10 | `launcher` | ランチャー | システム | タッチ |

---

## 2. 対象ハードウェア（実装に関係する部分のみ）

出典：https://docs.m5stack.com/en/core/PaperMono

| 項目 | 仕様 | 本アプリでの用途 |
|---|---|---|
| SoC | ESP32-S3R8（240MHz デュアルコア） | — |
| Flash / PSRAM | 16MB / 8MB Octal | PSRAM はフレームバッファと画像読込バッファに使用 |
| 画面 | 3.97" e-paper、SSD1677、480×800、**4階調** | 全描画 |
| タッチ | FT6336G（有効範囲 X:5〜475, Y:5〜795） | ランチャー・設定・あみだくじ |
| IMU | BMI270（I2C 0x68、IMU_INT→M5PM1） | シェイク検出（将来的にスリープ復帰も） |
| ボタン | KEY1=G2（A）、KEY2=G3（B）、電源ボタン | A=ランチャーへ戻る、B=アプリ内の補助操作 |
| microSD | SDMMC 4bit（CMD=G12, CLK=G13, D0〜D3=G11/G10/G9/G8）、電源=M5IOE1 PYG14（TF_EN）、検出=PYG1（TF_DET） | 画像・テキスト・フォントの格納 |
| RTC | RX8130CE | 未使用（将来的に「今日の一枚」の日付固定などに使う余地あり） |
| ブザー | G42（BB_PWM） | **使用しない** |
| LoRa / NFC | SX1262 / ST25R3916 | **使用しない**（Lite との互換を保つため） |

### e-paper 運用ルール（公式注意事項に基づく・必須）

1. 部分高速リフレッシュを **約10回行うごとに全画面リフレッシュを1回** 挟む。
2. 部分高速リフレッシュを途切れなく連続で行わない（DCバランスの崩れによる不可逆的なパネル損傷の恐れ）。
3. カスタム波形は使わない。内蔵 OTP 波形を使用する。
4. 長時間の連続使用後に黒点が出た場合は、しばらく放置してから全画面リフレッシュを行う。

### リフレッシュ方式（Nostos の実機実績に基づく）

描画は **工場 OTP 波形のみ** を使う（papermono-rs `ssd1677-otp`、Nostos `panel.rs` を流用）。SSD1677 の 2 枚の 1bpp プレーン（BW / RED、各 480×800/8 = 48,000 バイト）で 4 階調を表す。

| 方式 | Display Update Control 2 | 区分 | 所要時間（実測） | 本アプリでの用途 |
|---|---|---|---|---|
| `Mono`（モノクロ全面） | `0xF8`（反転同期）→ `0x14` | 全画面 | 約 4.65 s | アプリ切替、10 回ごとの残像消去 |
| `Partial`（モノクロ部分） | `0xFF` | 部分 | 約 1.10 s | 通常の結果表示・ランチャー・易の爻追加 |
| `Gray`（4 階調） | `0xD7` | 全画面（常に） | 約 4.41 s | タロットのカード表示など 4 階調画像 |

- 所要時間は 2026-10-02 に C153 実機で計測した（USB 給電・`present()` 呼び出しから Deep Sleep までの時間。フレームバッファから RAM への転送も含む）。

- M5GFX の `epd_quality` / `epd_text` / `epd_fast` / `epd_fastest`（カスタム LUT＋明示電圧）は **使わない**。Nostos で移植・実機評価したが、fast は黒が薄い、quality（約 3.4 s）はタップを取りこぼす、fastest 差分は残像が残る、という結果で撤回された（Nostos `firmware/nostos-fw/README.md`「パネル駆動方式」）。公式ドキュメントも「M5GFX の PaperMono 用波形は現時点で不安定」とし、OTP サンプル（https://github.com/m5stack/M5PaperMono-OTP-Demo）を推奨している。
- 更新後は必ず Deep Sleep Mode 1 に入れる。BUSY（GPIO18）が LOW に戻るまで次のコマンドを送らない。
- Nostos は部分更新 18 回ごとにフル更新しているが、本アプリは公式注意事項に従い **10 回** とする。18 は papermono-rs が独自に決めた値（`m5stack-papermono-lite/src/display.rs` の `PARTIALS_BEFORE_FULL`。同ファイルのコメントに「公式指針は約 10。旧ファームの 6 の 3 倍として 18」とある）で、実測の裏付けは記録されていない。

### コールドブート時の注意（Nostos で判明）

完全電源断からの起動直後、M5IOE1 の io3（EPD_VDD_ENABLE）が出力設定どおりに駆動されず、パネルが無電源のまま固まることがある。電源・リセット系の IOE1 ピンは、出力設定後に IN レジスタを読み戻し、追従しなければ「入力＋プルアップ → 出力」に切り替え直す（Nostos `ioe.rs` の `set_output_verified()` を流用）。

---

## 3. 開発環境

- **Rust / embassy（no_std）**。Nostos と同じ構成で、BSP は [`canardleteer/papermono-rs`](https://github.com/canardleteer/papermono-rs)（MIT、`g:\dev\papermono-rs`）を path 依存で参照する。
  - 依存は **`m5stack-papermono-lite` のみ**（LoRa/NFC を含む `m5stack-papermono` は使わない＝Lite 互換）。
  - esp-hal / esp-rtos / embassy の版は Nostos（`esp-hal-v1.2.0-rc.0` tag）に揃える。`Cargo.lock` も Nostos から引き継いだ。
- ツールチェーン：espup の `esp` toolchain。`. C:\Users\ospre\export-esp.ps1` で環境を通す。
- ビルド：`firmware/alea-fw` で `cargo +esp build --release`
- 書き込み：`espflash flash --port COM8 --monitor target\xtensa-esp32s3-none-elf\release\alea-fw`
- ハード非依存のロジックは `crates/alea-core`（`no_std`）に置き、リポジトリ直下で `cargo test` する。
- 補助ツール（画像変換など）は Python（Pillow + NumPy）。

---

## 4. アーキテクチャ

```
┌──────────────────────────────────────────────┐
│ Apps（tarot / iching / rune / dice / coin /   │
│       stick / amida / omikuji / yesno）       │
├──────────────────────────────────────────────┤
│ Launcher ・ AppManager（アプリ登録・切替）     │
├──────────────────────────────────────────────┤
│ Core Services                                │
│  Display  Input  Shake  Rng  Storage  Assets │
├──────────────────────────────────────────────┤
│ HAL（esp-hal / papermono-rs / Nostos 流用部） │
└──────────────────────────────────────────────┘
```

- アプリは **Core Services だけに依存** し、HAL を直接呼ばない。
- アプリ同士は互いに依存しない。
- メインループは embassy の単一タスクのイベント駆動とする。`Input` と `Shake` がイベントを発行し、`AppManager` が現在のアプリに配送する。

### ディレクトリ構成

```
paper-alea/
├─ Cargo.toml                 # ホスト用ワークスペース（crates/*）。firmware は exclude
├─ DESIGN.md / HANDOFF.md / CLAUDE.md / README.md
├─ crates/
│  └─ alea-core/              # no_std のハード非依存ロジック（乱数・抽選・易・ダイス等）
├─ firmware/
│  └─ alea-fw/                # Xtensa ファーム（独立ワークスペース）
│     ├─ .cargo/config.toml   # target / build-std 固定
│     └─ src/
│        ├─ main.rs
│        ├─ services/         # Core Services: app / app_manager / display / input / shake / storage / assets
│        ├─ board/            # ioe / panel / sd（Nostos・papermono-rs 由来）
│        ├─ ui/               # widgets / layout
│        └─ apps/             # launcher / tarot / iching / rune / dice /
│                             # coin / stick / amida / omikuji / yesno
├─ sd/                        # microSD にコピーする内容の原本
│  └─ alea/ …（§7）
└─ tools/
   ├─ convert_cards.py        # カード画像 → .a2b 一括変換
   └─ preview_a2b.py          # .a2b → PNG 逆変換（目視確認用）
```

- 乱数・抽選などのテストは `crates/alea-core` の `cargo test` で行う（§12）。

---

## 5. 共通インターフェース

```rust
// firmware/alea-fw/src/services/app.rs（M1 で確定）
pub enum Event { ShakeStart, ShakeEnd, Tap { x: i16, y: i16 }, ButtonA, ButtonB }
pub enum Action { None, Open(usize) }   // Open はランチャーだけが使う

pub trait App {
    fn id(&self) -> &'static str;         // "tarot" など
    fn title(&self) -> &'static str;      // ランチャー表示名
    fn requires_sd(&self) -> bool { false }
    async fn on_enter(&mut self, ctx: &mut Ctx);                 // 状態の初期化と初期画面の描画（反映は AppManager）
    async fn on_ready(&mut self, _ctx: &mut Ctx) {}              // 初期画面の全面更新の後（SD 読込など）
    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action;  // 描き直すときは自分で ctx.present()
    async fn on_exit(&mut self, _ctx: &mut Ctx) {}
}
// Ctx は Display / Input / Storage と I2C を所有し、アプリには描画・反映・ストレージの API だけを見せる。
// アプリは AppManager の enum（AnyApp）に静的に登録する（no_std・ヒープ無しのため dyn は使わない）。
```

- アプリ切替の手順：`on_exit` → 画面を消して `on_enter`（描画のみ）→ 全面更新 → `on_ready`。
- 直前の描画から 500ms 以内の操作は捨てる（R-2）。描画中の操作も捨てる。

- **ButtonA はすべてのアプリで「ランチャーへ戻る」に固定** する。`AppManager` が横取りし、アプリには配送しない。
- アプリは `on_enter` で状態を初期化する。前回の結果は保持しない（方針 D-5）。

---

## 6. Core Services 仕様

### 6.1 Display

- フレームバッファは BW / RED の 1bpp プレーン 2 枚（各 48,000 バイト）。Nostos と同様に静的確保とし、画像読込バッファは PSRAM に置く。
- 描画は `embedded-graphics` の `DrawTarget` として実装する。
- API の例：`present(Refresh)`、`draw_image_2bpp(x, y, buf, w, h)`、`full_refresh()`
- **リフレッシュ回数管理**：部分リフレッシュの回数をカウントし、10回に達したら次の描画を自動的に全画面リフレッシュにする。アプリ側では意識しない。
- アプリ切替時は必ず全画面リフレッシュを行う。

### 6.2 Input

- ボタン A/B の押下（短押しのみ。長押しは将来用に予約）
- タッチは「Tap」のみ扱う（ドラッグは不要）。座標は有効範囲（5〜475, 5〜795）にクランプする。

### 6.3 Shake（シェイク検出）

| パラメータ | 初期値 | 説明 |
|---|---|---|
| サンプリング | 100 Hz | 加速度のみ使用 |
| 判定量 | \|‖a‖ − 1g\| | 重力成分を除いた変動量 |
| 揺れ閾値 | 0.8 g | これを超えたらピークとして数える |
| 開始条件 | 600 ms 以内にピーク2回以上 | 机に置いた衝撃などの単発ピークでは反応させない |
| 終了条件 | 閾値未満が 300 ms 継続 | `ShakeEnd` を発行し、この時点で結果を確定する |

- BMI270 は papermono-rs の手順で初期化し、測定範囲を **±8g** にする（±2g では振ると飽和する）。判定ロジックは `alea-core::shake`（ホストでテスト）。
- 実機（2026-10-03）：振り 12 回で揺れ量の最大値は 1.6〜8.0g。誤検出・取りこぼしの報告なし。
- `ShakeStart` 発生時に画面表示などは行わない（アニメーションなし方針）。
- パラメータは `sd/alea/config.json` で上書きできるようにする（実機でのチューニング用）。

### 6.4 Rng

- 乱数源は esp-hal の `Trng`（`TrngSource` で RNG＋ADC1 の雑音を混ぜた真性乱数・D-4）。`alea-core` の `RandomSource` トレイト越しに使う。
- `uniform(n)`：0〜n−1 を **棄却サンプリング** で返し、剰余による偏りを避ける。
- `bernoulli()`：0/1（コイン・正逆位置用）
- ⚠ ESP32 のハードウェア乱数が真性乱数として振る舞う条件（RF 有効時など）は、ESP-IDF／esp-hal のドキュメントで要確認（D-4）。Alea は無線を使わないため特に注意する。

### 6.5 Storage / Assets

- 起動時に M5IOE1 経由で TF_EN を有効にし、TF_DET でカードの有無を確認する。
- Nostos の実績は SDHOST の **1bit**（CLK=G13 / CMD=G12 / DAT0=G11）＋ `sdio` ＋ `embedded-fatfs`。まずこの構成を流用し、4bit 化は読込速度の実測を見て判断する。
  - **実測（2026-10-03・C153・32GB カード）**：100KB の読み込みが 61〜62ms（約 1.6MB/s）。タロット 1 枚（約 86KB）で約 55ms の見込みのため、**4bit 化は行わない**。
  - TF_DET（IOE1 PYG1、挿入で LOW）は挿入・抜去の両方で正しく読めた。カードの初期化は TF_DET によらず常に試す。
  - `embedded-fatfs` は `lfn` を有効にする（`hexagrams.json` など 8.3 に収まらない名前を使うため）。
  - ⚠ SDHOST の DMA はフラッシュ上のデータを読めない。`embedded-fatfs` はクラスタのゼロ埋めにフラッシュ上の定数を使うため、書き込みは必ず RAM のバッファに写してから渡す（`board/sd.rs` の `RamBounce`）。これが無いとディレクトリ作成が `Io` エラーで失敗する。
  - 空き容量の取得は、FSInfo が無効なときは FAT 全体の走査になり 32GB カードで約 15 秒かかった（書き込み失敗の後）。FSInfo が有効なら約 5ms。起動時には呼ばない（SD 確認画面のみ）。
- SD が無い場合、SD 不要のアプリ（dice / coin / stick / amida / yesno）は動作させる。SD 必須のアプリは、ランチャー上でグレーアウト表示にする。
- JSON は `serde-json-core` などの no_std パーサで読み込み、アプリの `on_enter` 時にロードする（採用 crate は M3 までに確定）。

---

## 7. microSD 構成

```
/alea/
├─ config.json                # シェイク閾値など
├─ fonts/                     # 日本語ビットマップフォント（D-3）
├─ tarot/
│  ├─ img/00.a2b … 77.a2b・back.a2b  # カード画像 360×540（§8）
│  ├─ cap/00.a1b … 77.a1b     # 結果画面の名前の帯（§9 Tarot）
│  └─ word/00.a1b … 77.a1b    # キーワード画面
├─ iching/hexagrams.json      # 64卦
└─ rune/runes.json            # 24文字
```

- おみくじの運勢と一言はファームに組み込む（§10.5。素データは `tools/data/omikuji.json`）。

### JSON スキーマ（例）

```jsonc
// iching/hexagrams.json（King Wen 順、lines は下→上、1=陽 0=陰）
[{ "no": 1, "name": "乾為天", "lines": [1,1,1,1,1,1], "summary": "…" }]

// rune/runes.json
[{ "id": 0, "name": "Fehu", "aett": 1,
   "strokes": [[[x1,y1],[x2,y2]], …], "meaning": "…" }]

```

> 解釈文は既存書籍・サイトからの流用を避け、作者自身の言葉で執筆する。
> タロットのカード名・キーワードは、同じ作者の caelum-liber-arcanorum（`src/data/cards.json`・`src/data/meanings/*.json`）の文を流用する（2026-10-03 決定）。

---

## 8. 画像フォーマット `.a2b`（Alea 2-bit）

| オフセット | サイズ | 内容 |
|---|---|---|
| 0 | 4 | マジック `"A2B1"` |
| 4 | 2 | 幅（LE, uint16） |
| 6 | 2 | 高さ（LE, uint16） |
| 8 | 8 | 予約（0埋め） |
| 16 | w×h/4 | 画素データ |

- 画素は1画素2bitで、`0=黒, 1=暗灰, 2=明灰, 3=白`。
- 横方向に MSB から順に4画素を1バイトへ詰める。行の間にパディングは入れない。
- タロットの標準サイズは **360×540**（48,600 バイト＋ヘッダ16バイト。B 案・額装に合わせて 480×720 から変更・2026-10-03）。
- 逆位置は画像を別に持たず、描画時に180°回転させる。

**`.a1b`（Alea 1-bit）**：文字画面など白黒の画像。ヘッダ 16 バイト（`"A1B1"`・幅・高さ・左上 x・左上 y（いずれも LE の uint16）・予約 4 バイト）＋画素（1bit/画素・行ごとにバイト境界・MSB から・1=黒）。黒い画素だけを重ねて描く。

### 変換ツール `tools/convert_cards.py`（Pillow + NumPy）

試作で良好だった処理を正式仕様とする。

1. グレースケール化（`convert('L')`）
2. 360×540 に LANCZOS で縮小
3. `ImageOps.autocontrast(cutoff=1)`
4. `ImageEnhance.Contrast(1.15)`
5. **Atkinson ディザリング** で 0/85/170/255 の4階調に量子化
6. `.a2b` に書き出す。同時に確認用のプレビュー PNG と、全カードの縮小一覧画像を出力する

- 階調点（85/170）とコントラスト係数は CLI 引数で変更できるようにする（実機キャリブレーション用、D-2）。
- 入力は **フレーム・カード名を合成する前の絵柄のみ** とする。カード名は実機のフォントで描画する。
  - 原本：`g:\dev\caelum-liber-arcanorum\tools\tarot-gen\final\NN_<name>_<variant>.png`（1024×1536、2:3）。裏面は `src/assets/cards/full/back.webp`。
- **カード画像・変換結果（.a2b・プレビュー）はリポジトリに含めない**（`.gitignore` 済み）。絵柄は caelum-liber-arcanorum の `LICENSE-ASSETS.md` により © osprey74 All rights reserved。

---

## 9. 画面レイアウト

### 共通

- 画面は縦長の 480×800（ポートレート）で使う。
- 上端の 56px を見出し帯とし、アプリ名と「A:戻る」のヒントを表示する（タロットのみ例外）。
- **タップ対象は最小 64px 角** とする。**画面の下端 80px（y ≥ 720）には小さなタップ対象を置かない。**
  - 根拠（2026-10-03 C153 実機・机上・人差し指の腹・5 点×3 回）：タッチ座標は全体に右下へ約 (+9, +9) px ずれるため、Input で一律 (−9, −9) 補正した。補正後、上端と中央の 3 点は全打 ±8px 以内に収まったが、下端の 2 点は左下で y が −11〜+26px とばらつき、右下で x が平均 +12px（最大 +20px）ずれた。原因（指の角度か、パネル下部の特性か）は切り分けていない。

### Launcher

- **デザイン確定（2026-10-03）**：Claude Design「Alea ランチャー」（https://claude.ai/artifact/2oF1T17tXXRsjEz3YymGcm）の **B 案（書物調）**。二重枠・題字「ALEA」（Cormorant Garamond Bold）・罫と菱形・副題「骰子と偶然」・3×3 のタイル（ローマ数字 I〜IX・線画アイコン・Shippori Mincho Bold の名称）。
- 実装：`tools/render_launcher.py` で 4 階調の画像（`firmware/alea-fw/assets/launcher.2bpp`）とタイルの矩形を生成し、`Refresh::Gray` で表示する。未実装・SD が必要で使えないタイルは内側を明灰に薄めてタップを無効にする。
- 開発用アプリ（Refresh test・SD check）は、ランチャーでボタン B を押すと出る「Developer」ページに置く。
- 以下は M1 時点の仮デザイン（2 列×5 段のタイル）の記述。

- 見出し帯に「Alea」を表示する。
- 本体部分は 2列×5段のタイル（1タイル 約 240×148px）に、アイコン（線画）と名称を配置する。
- SD 必須のアプリで SD が無い場合は、タイルを明灰色にしてタップを無効にする。

### アプリ画面（dice・coin・yesno）

- **デザイン確定（2026-10-03）**：同キャンバスの 2 段目（書物調）。二重枠・見出し（ローマ数字／アプリ名／菱形付きの罫）・最下部の案内「A　戻る」「振って決める」。文字の大きさは実機で決めた値（ローマ数字・「合計」「表」「裏」24px、案内 20px、チップ 26px）で、`tools/render_apps.py` が正。
- 実装：`tools/render_apps.py` が、結果によらない部分を 1bit の台紙に、結果の部品（ダイスの輪郭と D6 の目・コインの面・YES / NO・HEADS / TAILS・表裏・是非・合計）と出目の数字の字形（Cormorant Garamond Bold・オールドスタイル数字）を 1bit の部品にして `firmware/alea-fw/assets/app_art.{1bpp,rs}` に書き出す。実機は台紙を貼り、部品と数字を黒で重ねる（`ui/art.rs`）。
- 結果の更新はモノクロの部分更新（灰色は使わない）。待機画面は結果の欄を「?」にする。
- ダイスの図形の中の数字は、字の上端と下端の中央を図形の重心に合わせる（オールドスタイル数字は字ごとに高さと位置が違い、書体の行の中央で置くと下がって見えるため。2026-10-03 実機で確認）。
- dice の選択チップは 5 列×2 段（各 75〜76×64px）。選択中は内側を反転して黒地に白抜きにする。

### Tarot

- **デザイン確定（2026-10-03）**：同キャンバスの 3 段目・**B 案（額装）**。二重枠の中にカード 360×540 を置く。

```
待機（Gray）              結果（Gray）               キーワード（B で切替・Partial）
┌ 二重枠 ─────────┐      ┌ 二重枠 ─────────┐       ┌ 二重枠 ─────────┐
│ I／タロット／罫  │      │ カード 360×540  │       │ XXI／世界／罫    │
│ 裏面 360×540     │      │ (60,38)         │       │ THE WORLD        │
│ (60,162)         │      │ 逆位置は 180°   │       │ [正位置] 4 語    │
│                  │      │ XXI · THE WORLD │       │ ─◆─              │
│                  │      │ 世界  [逆位置]  │       │ [逆位置] 4 語    │
│ A 戻る  振って…  │      │ A 戻る  B 言葉… │       │ A 戻る  B 絵に…  │
└──────────────────┘      └─────────────────┘       └──────────────────┘
```

- 待機は裏面（`img/back.a2b`）と「振って一枚を引く」。結果は 4 階調（約 4.4 s）、キーワードは白黒の部分更新。
- 結果画面の名前の帯とキーワード画面はカードごとに PC で 1bit 画像（`cap/NN.a1b`・`word/NN.a1b`）にする（`tools/render_tarot.py`）。
  台紙・正逆の札は firmware の画像部品（`tools/render_apps.py`）。キーワード画面は今の向きの札の内側を反転する。
- 1 行目は大アルカナが「ローマ数字 · 英名」、小アルカナは英名のみ。キーワード画面の見出しのローマ数字も大アルカナだけ。
- 実機（2026-10-03・C153）：表示・逆位置の回転・正逆とも正常。所要時間は待機・結果とも約 4.41 s（4 階調）。キーワード画面は 4 階調の直後のため部分更新がモノクロ全面に置き換わり約 4.47 s（ログ `Partial->MonoFull`）。
- microSD のパスは先頭の `/` の有無を問わない（Storage が取り除く。embedded-fatfs は `/` 付きでは開けない）。

---

## 10. 各ミニアプリ仕様

### 10.1 dice

- 種類：1D3 / 1D4 / 1D6 / 2D6 / 3D6 / 1D8 / 1D10 / 1D12 / 1D20 / 1D100
- 画面上部に10種の選択チップを並べ、タップで切り替える（初期値 1D6、アプリ終了まで保持）。
- シェイク終了で振る。各ダイスの出目を個別に表示し、複数個の場合は合計も表示する。
- D6 は目（ピップ）で、それ以外は多角形の輪郭と数字で描く。
- **1D100**：十の位 D10（00〜90）と一の位 D10（0〜9）を並べて表示する。「00＋0」を 100 とする。合計値を大きく表示する。
- 1D3 は 1〜3 を直接生成する。

### 10.2 coin

- 表裏を `bernoulli()` で決め、表裏それぞれの意匠（線画）と「HEADS／表」「TAILS／裏」を表示する。表は二重円に「A」（Alea の頭文字）、裏は二重円に放射状の線（長短 16 本）と中央の菱形。

### 10.3 stick

- 方式「左右（2方向）」と「8方位」を B ボタンで切り替える（初期値 8 方位・今の方式の札を反転）。
- 8 方位は方位盤と倒れた棒、方位名（北東・NORTH-EAST など）。左右は地面と倒れた棒、左／右（LEFT／RIGHT）。待機中は「?」。
- デザイン確定（2026-10-03）：キャンバス 4 段目。部品は `tools/render_m4.py`。

### 10.4 amida

- 本数は **2〜6 本**（初期値 5 本）で、B ボタンで循環切替して引き直す。振ると同じ本数で引き直す。下端の当たり項目は「1, 2, …」の番号とする（ラベル編集は将来対応）。
  - 上限を 6 本にしたのは、上の札の列の幅を 64px 以上に保つため（§9。8 本では約 50px になる・2026-10-03 決定）。
- 横線は 10 段の高さにランダムに引く。同じ高さで隣り合う横線は引かず、どの間にも最低 1 本引く（`alea-core::amida`）。
- 上の札（ローマ数字）またはその縦線の列をタップすると経路を太線で表示し、着いた番号を反転する（1回の `Partial` 描画で完結させ、辿るアニメーションは行わない）。
- 縦線・横線・経路は実機で描き、案内・ローマ数字・番号は画像部品。本数の案内は 22px（20px では潰れた）。

### 10.5 omikuji

- 運勢を重みに従って抽選し、その運勢の一言から1つを等確率で選ぶ（`alea-core::omikuji`）。
- 運勢は大吉・吉・中吉・小吉・末吉・凶の 6 段階（重み 15・20・20・18・15・12）、一言は各 5 文の計 30 文（Alea のオリジナル文・2026-10-03 総司さん確認）。素データは `tools/data/omikuji.json`。
- 二重枠の紙片に縦書きで「第N番」（一言の通し番号）・運勢・一言（3 行・各 10 字以内）を表示する。待機中は「?」。
- 文は画像部品として**ファームに組み込む**（microSD 不要・2026-10-03 決定）。縦書きは 1 字ずつ並べ、小書きの仮名は右上に寄せる。句読点と長音記号は使わない。
- 番号は 20px。11 以降は「第」と「番」の間を横書き（縦中横）にし、細い横画が消えないよう 2 値化の境目を 170 にする。

### 10.6 yesno

- YES / NO を大きく表示し、下に罫と「是／非」を添える。

### 10.7 iching（三枚硬貨法）

- 1回のシェイクで硬貨3枚を振る。表=3・裏=2 として合計を出す。
  - 6＝老陰（陰・変爻）、7＝少陽（陽）、8＝少陰（陰）、9＝老陽（陽・変爻）
- 爻は **下から上へ** 1本ずつ積み、毎回 `Partial` で部分更新する（6回で計6回の部分更新）。
- 6本がそろったら全画面リフレッシュして結果を表示する。
  - 本卦の名称と卦画を表示し、変爻には印（例：○／×）を付ける。
  - 変爻がある場合は、陰陽を反転させた之卦の名称と卦画も並べて表示する。
- 変爻が複数出た場合に読む爻辞の古典ルール（朱熹『易学啓蒙』）は実装しない。本卦・之卦・変爻位置の表示に留める。
- ⚠ 点数配分は実装前に既存実装（例：https://pypi.org/project/iching-divination/ ）と照合し、テストで検証する（D-6）。

### 10.8 rune

- エルダー・フサルク24文字から1つを `uniform(24)` で引く。
- 字形はフォントを使わず、`runes.json` の `strokes`（線分座標）で描画する。
- 逆位置（merkstave）は既定で OFF とし、設定で ON にできるようにする。字形が点対称の文字は逆位置の対象外とする。
- 説明文では「古代から伝わる占い」と断定せず、「ルーン文字を用いた現代の占い」と表現する。

---

## 11. 未確定事項・要検証

| ID | 内容 | 決め方 |
|---|---|---|
| ~~D-1~~ | 描画ドライバ：M5GFX か OTP サンプル系か | **決定（2026-10-02）**：Rust/embassy＋papermono-rs の OTP 駆動（Nostos 流用）。§2・§3 |
| D-2 | 実機での4階調の濃度、変換パラメータ | 実機表示で Atkinson の階調点・コントラストを調整する |
| D-3 | 日本語フォント（形式・書体・サイズ・縦書き） | Nostos は BIZ UDゴシック Bold から 16×16 1bpp グリフをビルド時に生成（`gen_jpfont.py`）。Alea は文字数が多いため、SD 読込のビットマップフォント形式を検討する。**yesno・coin・dice は日本語を画像部品（`tools/render_apps.py` で生成）で持つ（2026-10-03）**ため、M2 では不要。omikuji・iching・tarot の名称表示（M3〜M5）までに決める。**tarot は PC で画像化して SD に置く（2026-10-03 決定）**：カード名の帯とキーワード画面を render 系ツールで 1bit 画像にする（実機にフォント処理を持たない） |
| ~~D-4~~ | ハードウェア乱数が真性乱数となる条件（無線を使わない場合） | **決定（2026-10-03）**：無線も ADC も使わないと擬似乱数扱い（esp-hal `src/rng/mod.rs`）。esp-hal の `TrngSource`（RNG＋ADC1 の雑音）を常に生かし、`Trng` から取る。ADC1 は他に使わない |
| D-5 | 前回結果の保持や「今日の一枚」機能の有無 | 現状は保持しない。将来 RTC と組み合わせて検討する |
| D-6 | 易の擲銭法の点数配分 | 既存実装と照合し、ユニットテストで検証する |
| D-7 | 4階調画像の表示時間と、部分更新との併用可否 | M1/M3 で実機計測する |
| D-8 | リポジトリ名・公開範囲・ライセンス（カード画像の扱いを含む） | **一部決定（2026-10-02）**：`paper-alea`・公開・コードは MIT。カード画像はリポジトリに含めない。M5Stack 非公式である旨を README に明記済み |

---

## 12. 実装マイルストーン

| M | 内容 | 完了条件 |
|---|---|---|
| M1 | 土台：HAL 初期化（Nostos 流用）、Display（回数管理含む）、Input、Storage、Launcher（ダミーアプリ2本） | ランチャーから切替・A で復帰でき、11回目の部分更新が全画面になる |
| M2 | Shake・Rng ＋ `yesno` / `coin` / `dice` | 振って結果が出る。誤検出が許容範囲。1D100 の 100 判定が正しい |
| M3 | `convert_cards.py` ＋ `tarot` | 78枚＋裏面が表示でき、正逆が 50% 前後に分布する |
| M4 | `stick` / `amida` / `omikuji` | 各仕様どおり |
| M5 | `iching` / `rune` | 64卦・24文字すべての表示を確認。易の分布テストに合格 |
| M6 | 仕上げ：SD なし時の挙動、config.json、README、省電力（任意） | 実機で全アプリを通しで操作できる |

### テスト方針

- 乱数・抽選ロジックは `crates/alea-core` のホストテスト（`cargo test`）で、10万回試行の分布テストを行う。
- 描画は実機で目視確認する。`preview_a2b.py` で PNG に戻して差分を確認できるようにする。

---

## 13. 参考

- PaperMono 公式ドキュメント：https://docs.m5stack.com/en/core/PaperMono
- PaperMono OTP サンプル：https://github.com/m5stack/M5PaperMono-OTP-Demo
- papermono-rs（BSP）：https://github.com/canardleteer/papermono-rs
- Nostos（PaperMono 実機知見の出典）：`g:\dev\Nostos`（`firmware/nostos-fw/README.md`）
- PaperMono 回路図：https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf
- 易経（概要）：https://www.weblio.jp/content/%E5%91%A8%E6%98%93
- ルーン文字（概要）：https://www.weblio.jp/content/%E3%83%AB%E3%83%BC%E3%83%B3%E6%96%87%E5%AD%97
- 三枚硬貨法 実装例：https://pypi.org/project/iching-divination/

---

## 14. 変更履歴

| 日付 | 版 | 内容 |
|---|---|---|
| 2026-10-02 | v0.1 | 初版 |
| 2026-10-02 | v0.2 | 開発スタックを PlatformIO/Arduino から Rust/embassy（Nostos・papermono-rs 流用）へ変更。リフレッシュ方式を OTP 駆動（Mono / Partial / Gray）に改め、コールドブートの注意を追記。カード画像をリポジトリ外とする方針を明記（D-1 決定、D-8 一部決定） |
| 2026-10-02 | v0.2.1 | §2 にリフレッシュ方式ごとの実測所要時間を追記。ファームの Core Services のモジュール名を `services/` に変更（標準の `core` クレートとの衝突回避） |
| 2026-10-03 | v0.2.2 | §9「共通」にタップ対象の設計ルール（最小 64px 角・下端 80px に小さな対象を置かない）とタッチ補正の実測根拠を追記 |
| 2026-10-03 | v0.2.3 | §6.5 に microSD の実測（1bit で約 1.6MB/s・4bit 化しない）、TF_DET の確認結果、LFN の有効化、DMA とフラッシュ上データの注意を追記 |
| 2026-10-03 | v0.3 | M1 完了。§5 を実装に合わせて確定（async トレイト・`on_ready`・`Action`・静的登録、切替手順の変更） |
| 2026-10-03 | v0.4 | M2 の途中経過：D-4 決定（TRNG）、シェイク（±8g・実測）、yesno / coin / dice（英数字のみ）、ランチャーのデザイン確定（B 案・4 階調画像） |
| 2026-10-03 | v0.4.1 | §9 にアプリ画面（dice・coin・yesno）の確定デザインと実装（台紙＋1bit 部品＋数字の字形）を追記。§10.2・§10.6 に意匠と和文の添え字を追記。D-3 の M2 の扱いを更新 |
| 2026-10-03 | v0.4.2 | M2 完了。D-3 に tarot の日本語の扱い（PC で画像化して SD に置く）、§7 にタロットの文の出典（caelum-liber-arcanorum）を追記。`tools/convert_cards.py` を実装 |
| 2026-10-03 | v0.5 | M3：タロットのデザイン確定（B 案・額装）。カード画像を 360×540 に変更（§8）、`.a1b` を追加、§7 の SD 構成（`cap/`・`word/`、`cards.json` は使わない）、§9 Tarot を更新 |
| 2026-10-03 | v0.6 | M4：stick / amida / omikuji を実装（キャンバス 4 段目）。amida の本数を 2〜6 本に変更、omikuji の文をファームに組み込み（§7 から `omikuji.json` を削除）、§10.3〜§10.5 を更新 |
