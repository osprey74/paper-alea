# HANDOFF.md — paperAlea / M1：土台とランチャー

> 作成日：2026-10-02 ／ 更新日：2026-10-02（Rust/embassy 化に合わせて改訂）／ 対象マイルストーン：**M1**
> 前提資料：`DESIGN.md`（v0.2）。本書と DESIGN.md が食い違う場合は **DESIGN.md を正** とし、食い違いを報告すること。

---

## 0. このハンドオフの目的

M2 以降のミニアプリがすべて載る **土台** を完成させる。

- HAL の初期化（電源・IO エキスパンダ・画面・タッチ・ボタン・microSD）
- Core Services のうち **Display / Input / Storage / AppManager**
- **Launcher** と、土台検証用の **ダミーアプリ2本**

Shake・Rng・正式なミニアプリ（dice など）は **M1 の範囲外**。

---

## 1. 作業前に必ず行うこと

1. `DESIGN.md` の §2（ハードウェア・e-paper 運用ルール）、§4〜§6、§9 を読む。
2. **Nostos の実装を確認する。** 以下は Nostos で実機実績があるため、可能な限り流用する（Nostos・papermono-rs とも MIT。流用元はファイル冒頭のコメントに明記する）。
   - e-paper の描画ドライバと初期化手順：`g:\dev\Nostos\firmware\nostos-fw\src\panel.rs`（OTP 駆動・DESIGN.md §2）
   - M5PM1 / M5IOE1 の初期化手順と電源レール：同 `src/ioe.rs`（`set_output_verified()` によるコールドブート固着対策を含む）
   - 起動順序：同 `src/main.rs` の `main()` 冒頭
   - microSD の初期化（TF_EN の有効化を含む）：同 `src/sdlog.rs`
   - 日本語フォントの扱い（DESIGN.md D-3）：同 `src/jpfont.rs` と `tools/gen_jpfont.py`
   - 経緯と注意点：同 `README.md`（パネル駆動方式・コールドブート固着・電源の節）
   > BSP は `g:\dev\papermono-rs` を path 依存で参照する（`firmware/alea-fw/Cargo.toml`）。
3. 不明点が実装の方向性を左右する場合は、推測で進めずに質問を返す。

---

## 2. 厳守事項（パネル保護・互換性）

| # | ルール |
|---|---|
| R-1 | 部分リフレッシュ **10回ごとに全画面リフレッシュを1回**。Display 層で強制し、アプリからはバイパスできないようにする |
| R-2 | 部分高速リフレッシュをループで連続実行しない。描画はイベント起点のみとし、`loop()` 内で定期描画しない |
| R-3 | カスタム波形を使わない（内蔵 OTP 波形のみ） |
| R-4 | M5GFX 系のカスタム LUT 駆動（`epd_fastest` などの差分高速モード）は実装しない |
| R-5 | LoRa / NFC は初期化しない（PaperMono-Lite 互換。`m5stack-papermono` crate に依存しない）。C153 で LoRa 電源（M5PM1 の LoRa_EN）が既定で ON の場合は OFF にする（既定状態は要確認） |
| R-6 | ブザーは鳴らさない |

---

## 3. 作成するファイル

```
paper-alea/firmware/alea-fw/src/
├─ main.rs
├─ board/
│  ├─ mod.rs                   # bring_up()：HAL 初期化を集約（M5PM1・M5IOE1・電源系統）
│  ├─ ioe.rs                   # Nostos ioe.rs 由来
│  ├─ panel.rs                 # Nostos panel.rs 由来
│  └─ sd.rs                    # Nostos sdlog.rs 由来（読み込み API を追加）
├─ services/                  # Core Services（標準の `core` クレートと衝突しないよう改名）
│  ├─ app.rs                   # App トレイト・Event
│  ├─ app_manager.rs
│  ├─ display.rs
│  ├─ input.rs
│  └─ storage.rs
├─ ui/
│  ├─ layout.rs                # 座標定数
│  └─ widgets.rs               # 見出し帯・タイル・区切り線
└─ apps/
   ├─ launcher.rs
   ├─ debug_refresh.rs
   └─ debug_sd.rs
```

- 雛形（`Cargo.toml`・`.cargo/config.toml`・起動ログだけの `main.rs`）、`CLAUDE.md`、`sd/alea/` の空構成は初期設定で作成済み。
- shake / rng（ファーム側）/ assets は **作成しない**（M2 以降）。乱数の棄却サンプリングは `crates/alea-core` に実装済み。

---

## 4. タスク詳細

### T1. プロジェクト雛形

- 雛形は作成済み（2026-10-02 にビルド成功を確認）。
- **完了条件**：書き込みでき、シリアルに `[Board] alea-fw 0.1.0 boot` が出る（**2026-10-02 実機確認済み**・C153・COM8）。

### T2. Board（HAL 初期化）

- `board::bring_up()` に初期化をまとめる。順序は Nostos `main.rs` を優先し、以下は目安とする。
  1. esp-hal / esp-rtos 初期化
  2. システム I2C、M5PM1 / M5IOE1 初期化（PM1 への書き込みはバス整定 500ms 後。Nostos README「電源」節）
  3. e-paper 電源（M5IOE1 PYG3：EPD_3V3_EN）ON → リセット（PYG5）
  4. タッチ電源（PYG13：TP_VDD_EN）ON → リセット（PYG6）
  5. R-5 の LoRa 電源確認
- 充電 IC（IP2315）は I2C バスに常時接続しない（公式注意事項）。M1 では触らない。
- 各段の成否をシリアルに出力する。
- **完了条件**：起動ログに全段の OK が出る。失敗した段があっても停止せず、ログを残して続行する。
- **2026-10-02 実機確認済み**（C153）：PM1 電源保持・電源ボタン設定・IOE1（0x4F）・IP2315 隔離・EPD_VDD・タッチ電源とも OK。
  R-5：LoRa 電源（PM1 G2）は **出力 HIGH（ON）のまま残っていた**（Nostos が ON にした状態を PM1 が保持）ため、起動時に OFF にした。
  SKU は `0x50`（ST25R3916）の ACK で判定する（読み出しのみ・初期化しない）。Lite では G2 に触らない。

### T3. Display

```rust
pub enum Refresh { Gray, Partial }   // 差分高速モードは定義しない（R-4）

pub struct Display { /* panel・BW/RED プレーン・partial_count */ }

impl Display {
    pub const MAX_PARTIAL: u8 = 10;
    pub fn canvas(&mut self) -> &mut impl DrawTarget;   // 480x800・4階調（embedded-graphics）
    pub async fn present(&mut self, mode: Refresh);     // 描画反映。必要なら自動で全画面化
    pub async fn full_refresh(&mut self);               // 明示的なモノクロ全面更新（カウンタを0に戻す）
    pub fn partial_count(&self) -> u8;
}
```

- `present()` の規則：
  - `Gray`（4 階調・`0xD7`）は全画面扱いとし、カウンタを 0 にする。
  - `Partial`（モノクロ部分・`0xFF`）はカウンタを +1 する。
  - **カウンタが `MAX_PARTIAL` に達している状態で部分描画が要求されたら、その描画をモノクロ全面更新に置き換えて実行し、カウンタを 0 にする**（＝11回目が全画面になる）。
- リフレッシュ方式は Nostos `panel.rs` の OTP 駆動に合わせる（DESIGN.md §2）。Nostos の `PARTIALS_BEFORE_FULL = 18` は 10 に変える。
- 各方式の所要時間を計測し、DESIGN.md §2 の表に追記する。
- ログ：`[Display] mode=Partial partial=7/10 took=123ms` のように毎回出力する。
- **完了条件**：RefreshTestApp（T7）で、11回目の描画が全画面リフレッシュになることを目視とログで確認できる。
- **2026-10-02 実機確認済み**（暫定の検証ファーム：B＝部分更新・A＝全面更新）：部分更新 10 回の後の要求が
  `mode=Partial->MonoFull` に置き換わることをログと目視で確認。所要時間は DESIGN.md §2 の表に記載。
  T7 の RefreshTestApp に置き換えた後も同じ確認を行う。

### T4. Input

- ボタン A（G2）・B（G3）：active-low・内部プルアップの GPIO 入力（Nostos と同じ）。短押しのみ検出し、チャタリング対策を入れる。
- タッチ（FT6336G）：**Tap のみ**。押下から離すまでが 300ms 以内、移動量が 20px 以内を Tap とする。座標は X:5〜475、Y:5〜795 にクランプする。papermono-rs の `touch.rs` と Nostos `main.rs` のタッチ処理（50ms 周期・指離れ判定）を流用する。
- 画面の向き：ポートレート（480×800）。タッチ座標と描画座標の向きが一致していることを確認する。
- **完了条件**：画面の四隅と中央のタップ座標が、描画座標と ±10px 以内で一致する（RefreshTestApp で確認）。
- **2026-10-03 実機確認済み**（暫定のタッチ検証ファーム・5 点×3 回）：座標の向き・倍率は正しく、全体が右下へ約 (+9, +9) px
  ずれていたため `services/input.rs` の `TOUCH_OFFSET` で一律補正。補正後は上端・中央が ±8px 以内、下端 2 点は最大 26px。
  詳細と UI 設計ルールは DESIGN.md §9「共通」。ボタンは GPIO 直読み（デバウンス 40ms）。描画中の操作は捨てる（`resync`）。

### T5. Storage

- `Storage::begin()`：TF_EN（M5IOE1 PYG14）を ON → TF_DET（PYG1）で挿入確認 → マウント。まず Nostos と同じ SDHOST **1bit**（`sdio`＋`embedded-fatfs`）で実装し、4bit 化は bench の結果を見て判断する。
- API：`available() -> bool`、`exists(path) -> bool`、`read_all(path, buf) -> Result<usize, _>`（ファイルハンドル API は必要になったら追加）
- SD が無い、またはマウントに失敗しても **起動を止めない**。`available() == false` として続行する。
- 計測：`/alea/bench.bin`（100KB 程度、無ければ作成）の読込時間をログに出す（DESIGN.md の読込速度見積もりの検証用）。
- **完了条件**：SdCheckApp（T8）でマウント方式、空き容量、`/alea/` 配下の一覧、読込時間が表示される。SD を抜いた状態でもクラッシュしない。
- **2026-10-03 実機確認済み**（暫定の SD 確認画面）：1bit でマウント、`/alea/bench.bin`（100KB）を作成して 61ms で読込、
  一覧・`exists`・`read_all` も動作。SD を抜いた状態でも起動し `available()==false` で続行。
  ディレクトリ作成が失敗した件の原因（DMA がフラッシュ上のゼロ埋めデータを読めない）と対策は DESIGN.md §6.5。

### T6. App / AppManager / Launcher

- `app.rs` は DESIGN.md §5 のとおり（Shake 系イベントの定義は M1 でも置いておく）。
- `AppManager`：
  - アプリは静的に登録する（`requires_sd()` で SD 必須かを返す）。
  - ボタン A は **常に AppManager が横取り** し、Launcher 以外で押された場合は Launcher に戻る。Launcher 上では無視する。
  - アプリ切替時は `on_exit()` → `display.full_refresh()` → `on_enter()` の順で呼ぶ。
    - **実装では `on_exit()` → `on_enter()`（描画のみ）→ 全面更新 → `on_ready()` に変更した**（古い画面を全面更新で描き直す無駄を省くため。DESIGN.md §5 に反映）。
- `LauncherApp`：
  - 見出し帯（高さ 56px）に「Alea」を表示する。
  - 本体は 2列×5段のタイル（DESIGN.md §9）。M1 では登録済みのアプリだけを並べ、残りの枠は空にする。
  - `requires_sd() == true` のアプリは、SD が無い場合に明灰色で表示しタップを無効にする。
  - タイルのアイコンは M1 では不要（名称テキストのみ）。
  - 描画は `Refresh::Partial`。
- **完了条件**：ランチャー → 各ダミーアプリ → ボタン A でランチャーへ戻る、を10往復しても表示崩れやクラッシュがない。
- **2026-10-03 実機確認済み**：10 往復（Refresh test・SD check の両方）で表示崩れ・クラッシュなし。SD 無しでは SD check が網点表示になりタップ無効。
  明灰色は部分更新（モノクロ）では黒になるため、網点（4 画素に 1 つ黒）で表した。

### T7. RefreshTestApp（ダミー1：`debug_refresh`）

- 画面中央にカウンタ（描画回数）と `partialCount()` を表示する。
- タップするたびに `Refresh::Partial` で再描画する（連打は 500ms 間隔に制限し、R-2 を守る）。
- タップ位置に小さな十字を描き、座標も表示する（T4 の検証を兼ねる）。
- ボタン B で `full_refresh()` を実行する。
- **2026-10-03 実機確認済み**（タップ 8 回はすべて部分更新・B で全面更新）。

### T8. SdCheckApp（ダミー2：`debug_sd`、requiresSd=true）

- マウント方式（4bit / 1bit）、総容量と空き容量、`/alea/` 配下の一覧（最大20件）、bench.bin の読込時間を表示する。
- ボタン B で再スキャンする。
- **2026-10-03 実機確認済み**。結果が毎回同じだと再スキャンしたか分からないため、走査回数「scan #N」を表示する。

---

## 5. 受け入れ基準（M1 完了の定義）

- [x] ビルドが警告なしで通る（ライブラリ由来の警告は除く。残るのはリンカの RWX 通知のみ）
- [x] 起動からランチャー表示まで、全初期化段が OK でログに出る
- [x] **11回目の部分描画が自動で全画面リフレッシュになる**（ログと目視で確認・2026-10-02 暫定ファームで確認）
- [x] カスタム LUT・差分高速モードがコード上に存在しない
- [x] LoRa / NFC / ブザーが動作していない
- [x] タッチ座標と描画座標が ±10px 以内で一致する（上端・中央で合格。下端は ±10px を超える打点があるため、DESIGN.md §9 の設計ルールで吸収する。2026-10-03 総司さん了承）
- [x] SD あり：SdCheckApp に情報が表示される／SD なし：起動でき、SdCheckApp がグレーアウトする（2026-10-03 確認）
- [x] ランチャーとアプリ間の往復を10回行っても問題がない
- [x] PaperMono-Lite でも動作する設計になっている（実機なし・コードレビューで確認：依存は `m5stack-papermono-lite` のみ、SKU は `0x50` の ACK で判定、Lite では PM1 G2 に触らない、LoRa/NFC は初期化しない）

---

## 6. CLAUDE.md に書くこと（作業ルール）

- 仕様の正は `DESIGN.md`。仕様を変更したら DESIGN.md も更新し、変更履歴に1行追記する。
- §2 の厳守事項 R-1〜R-6 を転記する。
- アプリは `services/` の公開 API だけを使い、HAL を直接呼ばない。
- 新しいアプリは `apps/<id>.rs`（大きくなれば `apps/<id>/`）に置き、`App` トレイトを実装して `app_manager.rs` の `AnyApp`・`dispatch!`・登録表に加える。
- ログの接頭辞は `[Board] [Display] [Input] [Storage] [AppMgr] [<AppId>]` に統一する。
- 実機でしか確認できない事項は、推測で「完了」にせず、確認手順を書いて総司さんに依頼する。

---

## 7. 報告してほしいこと（M1 完了時）

1. 受け入れ基準のチェック結果
2. 各リフレッシュ方式（Mono / Partial / Gray）の所要時間の実測値
3. Nostos・papermono-rs から流用したファイルと、変更点
4. SD のマウント方式（4bit / 1bit）と bench.bin の読込時間の実測値
5. LoRa 電源の既定状態と、R-5 の対応内容
6. M2（Shake・Rng・yesno / coin / dice）に着手する前に決めておくべき事項

## 8. M1 完了報告（2026-10-03）

1. **受け入れ基準**：§5 のとおり全項目を満たした（Lite はコードレビューのみ）。
2. **リフレッシュ所要時間**（C153・USB 給電）：モノクロ全面 約 4.65 s／部分 約 1.10 s／4 階調 約 4.41 s（DESIGN.md §2）。
3. **流用元と変更点**
   - `board/ioe.rs` ← Nostos `ioe.rs`：LoRa・LED・RTC RAM・フロントライトを除外。SKU 判定、LoRa 電源遮断、IOE1 入力読み、FT6336G 読み出しを追加。
   - `board/panel.rs` ← Nostos `panel.rs`：部分更新の上限 18 → 10、実際の方式（`Painted`）を返す。診断出力を削減。
   - `board/sd.rs` ← Nostos `sdlog.rs`：読み込み・一覧・容量・ファイル作成を追加。`lfn` を有効化。`RamBounce`（DMA がフラッシュを読めない問題の対策）。
   - `services/input.rs`：Nostos `main.rs` のタッチ判定に倣い、デバウンスとタッチ補正 (−9, −9) を追加。
4. **SD**：1bit、`bench.bin`（100KB）の読込 60〜62 ms（約 1.6 MB/s）。4bit 化は不要と判断。
5. **LoRa 電源**：既定状態は不明（Nostos が ON にした状態が PM1 に残っており、出力 HIGH だった）。起動時に C153 なら OFF にする（R-5）。一度 OFF にすると以後は OFF のまま保持される。
6. **M2 の前に決めておくべき事項**
   - **乱数源（D-4）**：ESP32-S3 のハードウェア乱数は、無線を使わない場合のエントロピー源の扱いを esp-hal のドキュメントで確認する必要がある（Alea は無線を使わない）。
   - **シェイク検出**：papermono-rs に BMI270 の設定（`imu.rs`・`bmi270_config.rs`）がある。実機で加速度が読めるかを M2 の最初に確認する。
   - **部分更新 1.1 s の短縮**：パネル側の書き換え時間か、フレームバッファの変換時間かを計測して判断する（推測では変換の比率が大きい）。振ってから結果が出るまでの体感に直結する。
   - **日本語フォント（D-3）**：yes/no・coin・dice は英数字で足りるが、表示を日本語にするなら M2 で形式を決める。
   - **ダイスの選択チップ**：10 種のチップは DESIGN.md §9 の「最小 64px 角」を守る配置にする（例：5 列×2 段で 1 個 96×64px 以上）。

### 参考

- Nostos（実機知見の出典）：`g:\dev\Nostos\firmware\nostos-fw\README.md`
- papermono-rs（BSP）：https://github.com/canardleteer/papermono-rs（`docs/SAFETY.md` は書き込み前に必読）
- PaperMono 公式ドキュメント（PinMap・注意事項）：https://docs.m5stack.com/en/core/PaperMono
- M5PM1 / M5IOE1 電源管理：https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1
- PaperMono OTP サンプル：https://github.com/m5stack/M5PaperMono-OTP-Demo
- 工場出荷ファーム（初期化手順の参考）：https://github.com/m5stack/M5PaperMono-UserDemo

---

## 9. 次回の作業（2026-10-03 時点・M2 の続き）

**状況**：シェイク検出・真性乱数・yesno / coin / dice は実機で動作確認済み（英数字のみの仮画面）。
ランチャーは Claude Design の B 案（書物調・4 階調画像）で確定。アプリ画面のデザインも確定し、
Developer ページの「Design preview」で画像として実機確認済み。

**デザイン**：Claude Design「Alea ランチャー」 https://claude.ai/artifact/2oF1T17tXXRsjEz3YymGcm
（1 段目＝ランチャー案、2 段目＝アプリ画面案）。実機で決めた文字の大きさは `tools/render_apps.py` が正（旧 `render_app_mockups.py` は削除）。

| 要素 | 大きさ | 備考 |
|---|---|---|
| チップの文字（1D3 など） | 26 | 輪郭を足して太く（`CHIP_STROKE = 2`） |
| 最下部の案内「A　戻る」「振って決める」 | 20 | 太くしない |
| 見出しのローマ数字・「合計」・「表」「裏」 | 24 | 太くしない |
| ランチャーのタイルのローマ数字 | 15 | 輪郭を足して太く |

**次にやること**
1. キャンバス（2 段目）の文字の大きさを上の表に揃える。
2. ダイス・コイン・是か非かを確定デザインで作り直す。
   - 枠・見出し・案内・「合計」「表」「裏」「是」「非」など変わらない部分は PC で画像化（`render_app_mockups.py` を発展）。
   - 結果の数字と YES / NO / HEADS / TAILS は、Cormorant Garamond Bold をビットマップ書体に変換して実機で描く（オールドスタイル数字のまま）。
   - 結果の更新はモノクロの部分更新（灰色は使わない）。待機画面は結果の欄を「?」にする。
3. 確認後、`debug_preview` と仮画面のコードを整理する。

**進捗（2026-10-03）**
- 1. 済：キャンバス 2 段目の 4 枚（AppDice・AppDice100・AppCoin・AppYesNo）の文字の大きさを表に揃えた（チップは `-webkit-text-stroke: 0.5px` で太さを近似）。
- 2. 実装済・**実機の目視確認待ち**：`tools/render_apps.py` → `assets/app_art.{1bpp,rs}`、`ui/art.rs`、`apps/{dice,coin,yesno}.rs`。
  書き込みと起動（ランチャー表示まで）は確認済み。確認用の合成画像は `tools/out/app_*.png`。
  デザインに無かった画面（D6 以外の多面体・1D6 / 3D6・コインの裏・待機の「?」）は同じ意匠で補った。
  - **確認手順**：ダイスで 10 種のチップを順にタップし、それぞれ振る（選択中のチップが黒地に白抜き・出目・合計）。
    コイン・是か非かを表裏・YES / NO が両方出るまで振る。気になる点（数字の大きさ・位置・裏面の意匠）を指摘する。
  - 実機確認（総司さん）：図形の中の数字（1D3・1D4・1D10・1D12・1D20・1D100）が下寄りに見えた → 字の範囲で上下を合わせるよう修正（`art::text_ink_centered`）。**修正後の目視確認待ち**。他は問題なし。
  - 3D6 の合計「11」が「II」に似る件は、気にならないとのこと（そのままにする）。
- 2. の修正後の表示も総司さんが実機で確認済み（2026-10-03）。
- 3. 済：デザイン確認用の `debug_preview`・`mock_*.2bpp`・`render_app_mockups.py` を削除。仮画面のコード（拡大文字の `big_text` など）は 2. で置き換えて削除済み。

## 10. M3（タロット）の進捗（2026-10-03）

- M2 は完了（DESIGN.md §12 の 3 条件を確認済み）。
- デザイン：キャンバス 3 段目の **B 案（額装）** で確定。文字の大きさはキャンバスのまま。
- 実装済み：`alea-core::tarot`（78 枚の一様抽選・正逆・分布テスト 3 件）、`tools/convert_cards.py`（360×540）、
  `tools/render_tarot.py`（名前の帯・キーワード画面）、`render_apps.py` にタロットの台紙と札、`apps/tarot.rs`、`ui/image.rs`。
- **実機確認済み（2026-10-03）**：表示・絵柄・逆位置・キーワードの切替とも正常（総司さん）。初回は「microSD: not found」→ パス先頭の `/` が原因で、Storage で取り除くよう修正。
- 確認の手順（記録）：
  1. PC で `python tools/convert_cards.py` と `python tools/render_tarot.py` を実行し、`sd/alea/tarot/` の `img/`・`cap/`・`word/` を microSD の `/alea/tarot/` にコピーする。
  2. ランチャーで I（タロット）を開く → 裏面の待機画面。振る → カード・名前・正逆（4 階調）。B → キーワード（今の向きの札が黒地）。B → カードに戻る。A → ランチャー。
  3. 確認したいこと：4 階調の濃さ（D-2）、逆位置の回転、正逆が偏らないか（シリアルの `[tarot]` ログ）、表示までの時間。

## 11. M4（棒倒し・あみだくじ・おみくじ）（2026-10-03）

- デザイン：キャンバス 4 段目で確定（あみだくじの案内は「振って引く」）。
- 実装：`alea-core` の `stick`・`amida`・`omikuji`（テスト 8 件）、`tools/render_m4.py`（部品）、`tools/data/omikuji.json`（文）、
  `apps/{stick,amida,omikuji}.rs`。3 本とも microSD 不要。
- 実機で直した点：あみだくじの本数の案内 20→22px、おみくじの番号 18→20px・11 以降は縦中横・2 値化の境目 170。
- **実機確認済み（総司さん）**。おみくじの文 30 文も確認済み。

## 12. M5（易・ルーン）（2026-10-03）

- デザイン：キャンバス 5 段目で確定。文（易 64 卦・ルーン 24 文字）は Alea のオリジナルで、`tools/data/iching.json`・`runes.json`。
- 実装：`alea-core` の `iching`（三枚硬貨法・文王の順の表）・`rune`（点対称は逆位置なし）、`tools/render_m5.py`、`apps/{iching,rune}.rs`。2 本とも microSD 不要。
- D-6：表=3・裏=2、確率 6・9＝1/8、7・8＝3/8（Wikipedia と照合・テスト済み）。D-3：実機に日本語フォントを持たない。
- 実機で直した点：易の結果で変爻の印と矢印が重なる → 列を広げ矢印を卦名の高さへ、6 本目の画面を 2.5 秒見せてから結果へ、
  途中の上下の案内を 2 行・1 段大きく・太く、18px 以下の明朝は 2 値化の境目 170。
- **実機確認済み（総司さん）**。これでランチャーの 9 本すべてが動く。次は M6（仕上げ：SD なし時の挙動・config.json・README・省電力）。

## 13. M6（仕上げ）（2026-10-03）

- config.json：`alea-core::config`（キー名を探して整数を読む・範囲外は無視・テスト 4 件）、起動時に `services/shake.rs` が読む。見本は `sd/alea/config.json`。
- 電源・LED：`board/power.rs`（BTN_STATUS・PWR_SRC・LED・シャットダウン）、AppManager の `power_off`（終了画面 `tools/render_sleep.py`）、
  Ctx の振りの合図（緑）。詳細は DESIGN.md §6.6。M5PM1 の仕様は公式データシート v1.9 を調査（`sw_rev=0x54`）。
- README を現状に更新（使い方・microSD はタロットだけ・config.json）。
- **実機確認済み（総司さん）**：振りの合図・USB 接続中の待機と復帰・USB を抜いたときの電源オフ・電源オン。
- **全アプリの通し操作を実機で確認（総司さん・2026-10-03）**：M6 の完了条件を満たした。
- 省電力：自動電源オフのみ（操作が無いまま 3 分で終了画面を描いて切る・USB 給電中は切らない・`config.json` の `auto_off_min`）。
  振って起動は採らない（電源オフ中も加速度センサーに給電が要るため）。5 分で切れることを実機で確認後、既定を 3 分に変更。

## 14. v1.0.0 リリース（2026-10-03）

- M1〜M6 完了。総司さんの判断で正式リリースとする。バージョン 1.0.0（`firmware/alea-fw/Cargo.toml`・`crates/alea-core/Cargo.toml`）。
- 配布物：`alea-fw-v1.0.0-merged.bin`（`espflash save-image --chip esp32s3 --flash-size 16mb --merge --skip-padding` で作成・0x0 に書き込む）。
- 未確認：PaperMono-Lite（C153-Lite）の実機（在庫切れ・入荷後に確認予定）。

## 15. 設定画面（v1.0.0 の後・2026-10-03）

- ランチャーの開発用ページ（B）を設定画面に置き換えた。デザインはキャンバス 6 段目。
- バックライト（消灯・弱・強）、自動電源オフ（しない・1・3・5・10 分）、電池（電圧・残量の目安）、開発用（Refresh test・SD check）。
- 設定は M5PM1 の RTC RAM に保存し、起動時に復元（ライトと札）。電源オフの直前にライトを消す。USB 給電中の待機でも消す。
- 実装：`alea-core::settings`（テスト 4 件）、`board/power.rs`（フロントライト・VBAT・RTC RAM）、`apps/settings.rs`、`tools/render_m6.py`、`Action::OpenId`・`Action::Close`。
- **実機確認済み（総司さん）**。
- 記事用のスクリーンショットは `tools/screenshots.py`（未コミット・`tools/out/screenshots/`）。
