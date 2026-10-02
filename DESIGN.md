# DESIGN.md — Alea（paperAlea）

> M5Stack PaperMono 向け「偶然」ミニアプリ集
> 作成日：2026-10-02 ／ 更新日：2026-10-02 ／ ステータス：設計段階（v0.2）

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
// firmware/alea-fw/src/services/app.rs
pub enum Event {
    ShakeStart,
    ShakeEnd,
    Tap { x: i16, y: i16 },
    ButtonA,
    ButtonB,
}

pub trait App {
    fn id(&self) -> &'static str;         // "tarot" など
    fn title(&self) -> &'static str;      // ランチャー表示名
    fn requires_sd(&self) -> bool { false }
    fn on_enter(&mut self, ctx: &mut Ctx);              // 起動時：初期画面を描画
    fn on_event(&mut self, ctx: &mut Ctx, e: &Event);   // イベント処理
    fn on_exit(&mut self, _ctx: &mut Ctx) {}            // 終了時：バッファ解放
}
// Ctx は Display / Storage / Rng など Core Services への参照をまとめたもの。
// 描画完了待ち（BUSY）を async で扱うかどうかは M1 で確定する。
```

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

- `ShakeStart` 発生時に画面表示などは行わない（アニメーションなし方針）。
- パラメータは `sd/alea/config.json` で上書きできるようにする（実機でのチューニング用）。

### 6.4 Rng

- 乱数源は esp-hal の `Rng`（ESP32-S3 のハードウェア乱数）。`alea-core` の `RandomSource` トレイト越しに使う。
- `uniform(n)`：0〜n−1 を **棄却サンプリング** で返し、剰余による偏りを避ける。
- `bernoulli()`：0/1（コイン・正逆位置用）
- ⚠ ESP32 のハードウェア乱数が真性乱数として振る舞う条件（RF 有効時など）は、ESP-IDF／esp-hal のドキュメントで要確認（D-4）。Alea は無線を使わないため特に注意する。

### 6.5 Storage / Assets

- 起動時に M5IOE1 経由で TF_EN を有効にし、TF_DET でカードの有無を確認する。
- Nostos の実績は SDHOST の **1bit**（CLK=G13 / CMD=G12 / DAT0=G11）＋ `sdio` ＋ `embedded-fatfs`。まずこの構成を流用し、4bit 化は読込速度の実測を見て判断する。
- SD が無い場合、SD 不要のアプリ（dice / coin / stick / amida / yesno）は動作させる。SD 必須のアプリは、ランチャー上でグレーアウト表示にする。
- JSON は `serde-json-core` などの no_std パーサで読み込み、アプリの `on_enter` 時にロードする（採用 crate は M3 までに確定）。

---

## 7. microSD 構成

```
/alea/
├─ config.json                # シェイク閾値など
├─ fonts/                     # 日本語ビットマップフォント（D-3）
├─ tarot/
│  ├─ cards.json              # 78枚のメタデータ
│  └─ img/00.a2b … 77.a2b     # 画像（§8）
├─ iching/hexagrams.json      # 64卦
├─ rune/runes.json            # 24文字
└─ omikuji/omikuji.json       # 運勢と一言
```

### JSON スキーマ（例）

```jsonc
// tarot/cards.json
[{ "id": 21, "arcana": "major", "numeral": "XXI",
   "name_en": "THE WORLD", "name_ja": "世界",
   "upright": "完成・統合", "reversed": "未完・停滞" }]

// iching/hexagrams.json（King Wen 順、lines は下→上、1=陽 0=陰）
[{ "no": 1, "name": "乾為天", "lines": [1,1,1,1,1,1], "summary": "…" }]

// rune/runes.json
[{ "id": 0, "name": "Fehu", "aett": 1,
   "strokes": [[[x1,y1],[x2,y2]], …], "meaning": "…" }]

// omikuji/omikuji.json
{ "fortunes": [{ "label": "大吉", "weight": 1, "messages": ["…"] }] }
```

> 解釈文は既存書籍・サイトからの流用を避け、作者自身の言葉で執筆する。

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
- タロットの標準サイズは **480×720**（86,400 バイト＋ヘッダ16バイト）。
- 逆位置は画像を別に持たず、描画時に180°回転させる。

### 変換ツール `tools/convert_cards.py`（Pillow + NumPy）

試作で良好だった処理を正式仕様とする。

1. グレースケール化（`convert('L')`）
2. 480×720 に LANCZOS で縮小
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

### Launcher

- 見出し帯に「Alea」を表示する。
- 本体部分は 2列×5段のタイル（1タイル 約 240×148px）に、アイコン（線画）と名称を配置する。
- SD 必須のアプリで SD が無い場合は、タイルを明灰色にしてタップを無効にする。

### Tarot

```
┌──────────── 480 ────────────┐
│                             │
│     カード画像 480×720       │  ← Gray（4階調）
│   （逆位置は180°回転）       │
│                             │
├─────────────────────────────┤ y=720
│   XXI  THE WORLD  ／ 世界    │  80px：番号・名称・正逆
└─────────────────────────────┘ y=800
```

- 待機画面にはカード裏面（`img/back.a2b`）と「本体を振ってください」を表示する。
- B ボタンで「キーワード表示」に切り替える（正位置・逆位置の意味を `Partial` で表示）。

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

- 表裏を `bernoulli()` で決め、表裏それぞれの意匠（線画）と「表／裏」を表示する。

### 10.3 stick

- 設定で「左右（2方向）」と「8方位」を B ボタンで切り替える。
- 倒れた状態の棒を線画で描き、方向名を表示する。

### 10.4 amida

- 本数は 2〜8 本で、B ボタンで循環切替する。下端の当たり項目は「1, 2, …」の番号とする（ラベル編集は将来対応）。
- 横線はランダム生成する。隣接する横線が同じ高さで連続しないようにする。
- 上端の縦線をタップするとその経路を太線で表示する（1回の `Partial` 描画で完結させ、辿るアニメーションは行わない）。

### 10.5 omikuji

- `omikuji.json` の重みに従って運勢を抽選し、その運勢の `messages` から1つを選んで表示する。
- 縦長の紙片風レイアウトにする（縦書き対応は D-3 のフォント次第）。

### 10.6 yesno

- YES / NO を大きく表示する。

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
| D-3 | 日本語フォント（形式・書体・サイズ・縦書き） | Nostos は BIZ UDゴシック Bold から 16×16 1bpp グリフをビルド時に生成（`gen_jpfont.py`）。Alea は文字数が多いため、SD 読込のビットマップフォント形式を M1 で検討する |
| D-4 | ハードウェア乱数が真性乱数となる条件（無線を使わない場合） | ESP-IDF／esp-hal 公式ドキュメントで確認する |
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
