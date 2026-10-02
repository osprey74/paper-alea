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
├─ core/
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
- **完了条件**：書き込みでき、シリアルに `[Board] alea-fw 0.1.0 boot` が出る（**実機確認待ち**）。

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

### T4. Input

- ボタン A（G2）・B（G3）：active-low・内部プルアップの GPIO 入力（Nostos と同じ）。短押しのみ検出し、チャタリング対策を入れる。
- タッチ（FT6336G）：**Tap のみ**。押下から離すまでが 300ms 以内、移動量が 20px 以内を Tap とする。座標は X:5〜475、Y:5〜795 にクランプする。papermono-rs の `touch.rs` と Nostos `main.rs` のタッチ処理（50ms 周期・指離れ判定）を流用する。
- 画面の向き：ポートレート（480×800）。タッチ座標と描画座標の向きが一致していることを確認する。
- **完了条件**：画面の四隅と中央のタップ座標が、描画座標と ±10px 以内で一致する（RefreshTestApp で確認）。

### T5. Storage

- `Storage::begin()`：TF_EN（M5IOE1 PYG14）を ON → TF_DET（PYG1）で挿入確認 → マウント。まず Nostos と同じ SDHOST **1bit**（`sdio`＋`embedded-fatfs`）で実装し、4bit 化は bench の結果を見て判断する。
- API：`available() -> bool`、`exists(path) -> bool`、`read_all(path, buf) -> Result<usize, _>`（ファイルハンドル API は必要になったら追加）
- SD が無い、またはマウントに失敗しても **起動を止めない**。`available() == false` として続行する。
- 計測：`/alea/bench.bin`（100KB 程度、無ければ作成）の読込時間をログに出す（DESIGN.md の読込速度見積もりの検証用）。
- **完了条件**：SdCheckApp（T8）でマウント方式、空き容量、`/alea/` 配下の一覧、読込時間が表示される。SD を抜いた状態でもクラッシュしない。

### T6. App / AppManager / Launcher

- `app.rs` は DESIGN.md §5 のとおり（Shake 系イベントの定義は M1 でも置いておく）。
- `AppManager`：
  - アプリは静的に登録する（`requires_sd()` で SD 必須かを返す）。
  - ボタン A は **常に AppManager が横取り** し、Launcher 以外で押された場合は Launcher に戻る。Launcher 上では無視する。
  - アプリ切替時は `on_exit()` → `display.full_refresh()` → `on_enter()` の順で呼ぶ。
- `LauncherApp`：
  - 見出し帯（高さ 56px）に「Alea」を表示する。
  - 本体は 2列×5段のタイル（DESIGN.md §9）。M1 では登録済みのアプリだけを並べ、残りの枠は空にする。
  - `requires_sd() == true` のアプリは、SD が無い場合に明灰色で表示しタップを無効にする。
  - タイルのアイコンは M1 では不要（名称テキストのみ）。
  - 描画は `Refresh::Partial`。
- **完了条件**：ランチャー → 各ダミーアプリ → ボタン A でランチャーへ戻る、を10往復しても表示崩れやクラッシュがない。

### T7. RefreshTestApp（ダミー1：`debug_refresh`）

- 画面中央にカウンタ（描画回数）と `partialCount()` を表示する。
- タップするたびに `Refresh::Partial` で再描画する（連打は 500ms 間隔に制限し、R-2 を守る）。
- タップ位置に小さな十字を描き、座標も表示する（T4 の検証を兼ねる）。
- ボタン B で `full_refresh()` を実行する。

### T8. SdCheckApp（ダミー2：`debug_sd`、requiresSd=true）

- マウント方式（4bit / 1bit）、総容量と空き容量、`/alea/` 配下の一覧（最大20件）、bench.bin の読込時間を表示する。
- ボタン B で再スキャンする。

---

## 5. 受け入れ基準（M1 完了の定義）

- [ ] ビルドが警告なしで通る（ライブラリ由来の警告は除く）
- [ ] 起動からランチャー表示まで、全初期化段が OK でログに出る
- [ ] **11回目の部分描画が自動で全画面リフレッシュになる**（ログと目視で確認）
- [ ] カスタム LUT・差分高速モードがコード上に存在しない
- [ ] LoRa / NFC / ブザーが動作していない
- [ ] タッチ座標と描画座標が ±10px 以内で一致する
- [ ] SD あり：SdCheckApp に情報が表示される／SD なし：起動でき、SdCheckApp がグレーアウトする
- [ ] ランチャーとアプリ間の往復を10回行っても問題がない
- [ ] PaperMono-Lite でも動作する設計になっている（実機がない場合はコードレビューで確認）

---

## 6. CLAUDE.md に書くこと（作業ルール）

- 仕様の正は `DESIGN.md`。仕様を変更したら DESIGN.md も更新し、変更履歴に1行追記する。
- §2 の厳守事項 R-1〜R-6 を転記する。
- アプリは `core/` の公開 API だけを使い、HAL を直接呼ばない。
- 新しいアプリは `apps/<id>.rs`（大きくなれば `apps/<id>/`）に置き、`App` トレイトを実装して `main.rs` で登録する。
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

### 参考

- Nostos（実機知見の出典）：`g:\dev\Nostos\firmware\nostos-fw\README.md`
- papermono-rs（BSP）：https://github.com/canardleteer/papermono-rs（`docs/SAFETY.md` は書き込み前に必読）
- PaperMono 公式ドキュメント（PinMap・注意事項）：https://docs.m5stack.com/en/core/PaperMono
- M5PM1 / M5IOE1 電源管理：https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1
- PaperMono OTP サンプル：https://github.com/m5stack/M5PaperMono-OTP-Demo
- 工場出荷ファーム（初期化手順の参考）：https://github.com/m5stack/M5PaperMono-UserDemo
