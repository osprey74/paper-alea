# DESIGN.md — Alea（paperAlea）

> M5Stack PaperMono 向け「偶然」ミニアプリ集
> 作成日：2026-10-02 ／ ステータス：設計段階（v0.1）

---

## 1. 概要

| 項目 | 内容 |
|---|---|
| 作品名（画面表記） | **Alea**（ラテン語「骰子・偶然」） |
| プロジェクト／リポジトリ名 | `paper-alea`（表記は `paperAlea`）※最終決定は未了 |
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

### リフレッシュモード（M5GFX 公式実測値）

| モード | 1回あたり | 本アプリでの用途 |
|---|---|---|
| `epd_quality` | 4.71 s | タロットのカード表示（4階調画像） |
| `epd_text` | 0.45 s | 通常の結果表示・ランチャー |
| `epd_fast` | 0.34 s | 易の爻追加など、小さな部分更新 |
| `epd_fastest` | 0.07 s | 原則使用しない |

> ⚠ 公式ドキュメントでは「M5GFX の PaperMono 用波形は現時点で不安定」とされ、メーカーの OTP サンプル（https://github.com/m5stack/M5PaperMono-OTP-Demo）が推奨されている。**描画ドライバはまず Nostos で実績のある方式を流用する**こと（§11 未確定事項 D-1）。

---

## 3. 開発環境

- PlatformIO + Arduino framework（C++17）
- 主要ライブラリ：M5Unified（develop）、M5GFX、M5PM1、M5IOE1、ArduinoJson
- `platformio.ini` の基本形は公式記載を踏襲する。LoRa・NFC 関連の依存は削除する。

```ini
[env:m5stack-papermono]
platform = espressif32@6.12.0
board = esp32-s3-devkitm-1
framework = arduino
board_build.partitions = default_16MB.csv
board_upload.flash_size = 16MB
board_upload.maximum_size = 16777216
board_build.arduino.memory_type = qio_opi
build_flags =
    -DESP32S3
    -DBOARD_HAS_PSRAM
    -mfix-esp32-psram-cache-issue
    -DCORE_DEBUG_LEVEL=0
    -DARDUINO_USB_CDC_ON_BOOT=1
    -DARDUINO_USB_MODE=1
lib_deps =
    M5Unified = https://github.com/m5stack/M5Unified#develop
    M5PM1 = https://github.com/m5stack/M5PM1
    M5IOE1 = https://github.com/m5stack/M5IOE1
    bblanchon/ArduinoJson@^7
```

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
│ HAL（M5Unified / M5PM1 / M5IOE1 / ドライバ）  │
└──────────────────────────────────────────────┘
```

- アプリは **Core Services だけに依存** し、HAL を直接呼ばない。
- アプリ同士は互いに依存しない。
- メインループは単一タスクのイベント駆動とする。`Input` と `Shake` がイベントを発行し、`AppManager` が現在のアプリに配送する。

### ディレクトリ構成

```
paper-alea/
├─ platformio.ini
├─ DESIGN.md / HANDOFF.md / CLAUDE.md
├─ src/
│  ├─ main.cpp
│  ├─ core/
│  │  ├─ App.h               # 共通インターフェース
│  │  ├─ AppManager.{h,cpp}
│  │  ├─ Display.{h,cpp}     # 描画・リフレッシュ回数管理
│  │  ├─ Input.{h,cpp}       # ボタン・タッチ
│  │  ├─ Shake.{h,cpp}       # IMU シェイク検出
│  │  ├─ Rng.{h,cpp}         # 一様乱数
│  │  ├─ Storage.{h,cpp}     # microSD 初期化・ファイル I/O
│  │  └─ Assets.{h,cpp}      # .a2b 画像・JSON・フォント読込
│  ├─ ui/
│  │  ├─ Widgets.{h,cpp}     # 見出し帯・ボタン・区切り線
│  │  └─ Layout.h            # 座標定数
│  └─ apps/
│     ├─ launcher/  tarot/  iching/  rune/  dice/
│     └─ coin/  stick/  amida/  omikuji/  yesno/
├─ sd/                        # microSD にコピーする内容の原本
│  └─ alea/ …（§7）
├─ tools/
│  ├─ convert_cards.py        # カード画像 → .a2b 一括変換
│  └─ preview_a2b.py          # .a2b → PNG 逆変換（目視確認用）
└─ test/                      # 乱数分布・ロジックのネイティブテスト
```

---

## 5. 共通インターフェース

```cpp
// src/core/App.h
enum class EventType { ShakeStart, ShakeEnd, Tap, ButtonA, ButtonB };

struct Event {
  EventType type;
  int16_t x = 0, y = 0;   // Tap のみ
};

class App {
public:
  virtual ~App() = default;
  virtual const char* id() const = 0;        // "tarot" など
  virtual const char* title() const = 0;     // ランチャー表示名
  virtual void onEnter() = 0;                // 起動時：初期画面を描画
  virtual void onEvent(const Event& e) = 0;  // イベント処理
  virtual void onExit() {}                   // 終了時：バッファ解放
};
```

- **ButtonA はすべてのアプリで「ランチャーへ戻る」に固定** する。`AppManager` が横取りし、アプリには配送しない。
- アプリは `onEnter` で状態を初期化する。前回の結果は保持しない（方針 D-5）。

---

## 6. Core Services 仕様

### 6.1 Display

- フレームバッファは PSRAM 上に確保する。
- API の例：`draw(mode)`、`drawImage2bpp(x, y, buf, w, h)`、`text(...)`、`fullRefresh()`
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

- 乱数源は `esp_random()`。
- `uniform(n)`：0〜n−1 を **棄却サンプリング** で返し、剰余による偏りを避ける。
- `bernoulli()`：0/1（コイン・正逆位置用）
- ⚠ ESP32 のハードウェア乱数が真性乱数として振る舞う条件（RF 有効時など）は、ESP-IDF ドキュメントで要確認（D-4）。

### 6.5 Storage / Assets

- 起動時に M5IOE1 経由で TF_EN を有効にし、TF_DET でカードの有無を確認する。
- SD が無い場合、SD 不要のアプリ（dice / coin / stick / amida / yesno）は動作させる。SD 必須のアプリは、ランチャー上でグレーアウト表示にする。
- JSON は ArduinoJson で読み込み、アプリの `onEnter` 時にロードする。

---

## 7. microSD 構成

```
/alea/
├─ config.json                # シェイク閾値など
├─ fonts/                     # VLW 等の日本語フォント（D-3）
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
│     カード画像 480×720       │  ← epd_quality
│   （逆位置は180°回転）       │
│                             │
├─────────────────────────────┤ y=720
│   XXI  THE WORLD  ／ 世界    │  80px：番号・名称・正逆
└─────────────────────────────┘ y=800
```

- 待機画面にはカード裏面（`img/back.a2b`）と「本体を振ってください」を表示する。
- B ボタンで「キーワード表示」に切り替える（正位置・逆位置の意味を `epd_text` で表示）。

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
- 上端の縦線をタップするとその経路を太線で表示する（1回の `epd_text` 描画で完結させ、辿るアニメーションは行わない）。

### 10.5 omikuji

- `omikuji.json` の重みに従って運勢を抽選し、その運勢の `messages` から1つを選んで表示する。
- 縦長の紙片風レイアウトにする（縦書き対応は D-3 のフォント次第）。

### 10.6 yesno

- YES / NO を大きく表示する。

### 10.7 iching（三枚硬貨法）

- 1回のシェイクで硬貨3枚を振る。表=3・裏=2 として合計を出す。
  - 6＝老陰（陰・変爻）、7＝少陽（陽）、8＝少陰（陰）、9＝老陽（陽・変爻）
- 爻は **下から上へ** 1本ずつ積み、毎回 `epd_fast` で部分更新する（6回で計6回の部分更新）。
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
| D-1 | 描画ドライバ：M5GFX か OTP サンプル系か | Nostos の実績を優先し、M1 で確定する |
| D-2 | 実機での4階調の濃度、変換パラメータ | 実機表示で Atkinson の階調点・コントラストを調整する |
| D-3 | 日本語フォント（形式・書体・サイズ・縦書き） | M1 で VLW 変換と SD 読込を検証する |
| D-4 | `esp_random()` が真性乱数となる条件 | ESP-IDF 公式ドキュメントで確認する |
| D-5 | 前回結果の保持や「今日の一枚」機能の有無 | 現状は保持しない。将来 RTC と組み合わせて検討する |
| D-6 | 易の擲銭法の点数配分 | 既存実装と照合し、ユニットテストで検証する |
| D-7 | 4階調画像を `epd_quality` 以外のモードで表示できるか | M2 で実機検証する |
| D-8 | リポジトリ名・公開範囲・ライセンス（カード画像の扱いを含む） | 公開前に決定する。M5Stack 非公式である旨を README に明記する |

---

## 12. 実装マイルストーン

| M | 内容 | 完了条件 |
|---|---|---|
| M1 | 土台：HAL 初期化、Display（回数管理含む）、Input、Storage、Launcher（ダミーアプリ2本） | ランチャーから切替・A で復帰でき、11回目の部分更新が全画面になる |
| M2 | Shake・Rng ＋ `yesno` / `coin` / `dice` | 振って結果が出る。誤検出が許容範囲。1D100 の 100 判定が正しい |
| M3 | `convert_cards.py` ＋ `tarot` | 78枚＋裏面が表示でき、正逆が 50% 前後に分布する |
| M4 | `stick` / `amida` / `omikuji` | 各仕様どおり |
| M5 | `iching` / `rune` | 64卦・24文字すべての表示を確認。易の分布テストに合格 |
| M6 | 仕上げ：SD なし時の挙動、config.json、README、省電力（任意） | 実機で全アプリを通しで操作できる |

### テスト方針

- 乱数・抽選ロジックは `test/` のネイティブ環境（PlatformIO `native`）で、10万回試行の分布テストを行う。
- 描画は実機で目視確認する。`preview_a2b.py` で PNG に戻して差分を確認できるようにする。

---

## 13. 参考

- PaperMono 公式ドキュメント：https://docs.m5stack.com/en/core/PaperMono
- PaperMono OTP サンプル：https://github.com/m5stack/M5PaperMono-OTP-Demo
- PaperMono 回路図：https://m5stack-doc.oss-cn-shenzhen.aliyuncs.com/1267/PaperMono_SCH_V0.6.2_20260522.pdf
- 易経（概要）：https://www.weblio.jp/content/%E5%91%A8%E6%98%93
- ルーン文字（概要）：https://www.weblio.jp/content/%E3%83%AB%E3%83%BC%E3%83%B3%E6%96%87%E5%AD%97
- 三枚硬貨法 実装例：https://pypi.org/project/iching-divination/
