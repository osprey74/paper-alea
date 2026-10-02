# HANDOFF.md — paperAlea / M1：土台とランチャー

> 作成日：2026-10-02 ／ 対象マイルストーン：**M1**
> 前提資料：`DESIGN.md`（v0.1）。本書と DESIGN.md が食い違う場合は **DESIGN.md を正** とし、食い違いを報告すること。

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
2. **Nostos の実装を確認する。** 以下は Nostos で実機実績があるため、可能な限り流用する。
   - e-paper の描画ドライバと初期化手順（DESIGN.md D-1）
   - M5PM1 / M5IOE1 の初期化手順
   - microSD の初期化（TF_EN の有効化を含む）
   - 日本語フォントの扱い（DESIGN.md D-3）
   > Nostos リポジトリの場所は総司さんに確認すること。確認できない場合は公式サンプル（§7 参考）で進め、その旨を報告する。
3. 不明点が実装の方向性を左右する場合は、推測で進めずに質問を返す。

---

## 2. 厳守事項（パネル保護・互換性）

| # | ルール |
|---|---|
| R-1 | 部分リフレッシュ **10回ごとに全画面リフレッシュを1回**。Display 層で強制し、アプリからはバイパスできないようにする |
| R-2 | 部分高速リフレッシュをループで連続実行しない。描画はイベント起点のみとし、`loop()` 内で定期描画しない |
| R-3 | カスタム波形を使わない（内蔵 OTP 波形のみ） |
| R-4 | `epd_fastest` は使用しない |
| R-5 | LoRa / NFC は初期化しない（PaperMono-Lite 互換）。C153 で LoRa 電源（M5PM1 の LoRa_EN）が既定で ON の場合は OFF にする（既定状態は要確認） |
| R-6 | ブザーは鳴らさない |

---

## 3. 作成するファイル

```
paper-alea/
├─ platformio.ini               # DESIGN.md §3 を基に作成（LoRa/NFC 依存は除く）
├─ CLAUDE.md                    # 本プロジェクトの作業ルール（§6）
├─ src/
│  ├─ main.cpp
│  ├─ core/
│  │  ├─ App.h
│  │  ├─ AppManager.h / .cpp
│  │  ├─ Display.h / .cpp
│  │  ├─ Input.h / .cpp
│  │  ├─ Storage.h / .cpp
│  │  └─ Board.h / .cpp         # HAL 初期化を集約（M5PM1・M5IOE1・電源系統）
│  ├─ ui/
│  │  ├─ Layout.h               # 座標定数
│  │  └─ Widgets.h / .cpp       # 見出し帯・タイル・区切り線
│  └─ apps/
│     ├─ launcher/LauncherApp.h / .cpp
│     ├─ debug_refresh/RefreshTestApp.h / .cpp
│     └─ debug_sd/SdCheckApp.h / .cpp
└─ sd/alea/                     # 空ディレクトリ構成のみ（DESIGN.md §7）＋ README.txt
```

Shake.* / Rng.* / Assets.* は **作成しない**（M2 以降）。

---

## 4. タスク詳細

### T1. プロジェクト雛形

- `platformio.ini` を DESIGN.md §3 のとおり作成する。
- `lib_deps` に M5GFX を明示するかどうかは、M5Unified の依存解決に従う。ビルドが通る最小構成にする。
- **完了条件**：空の `setup()/loop()` がビルド・書き込みでき、シリアルに起動ログが出る。

### T2. Board（HAL 初期化）

- `Board::begin()` に初期化をまとめる。順序の例は以下（Nostos の順序があればそちらを優先）。
  1. M5Unified 初期化
  2. M5PM1 / M5IOE1 初期化
  3. e-paper 電源（M5IOE1 PYG3：EPD_3V3_EN）ON → リセット（PYG5）
  4. タッチ電源（PYG13：TP_VDD_EN）ON → リセット（PYG6）
  5. R-5 の LoRa 電源確認
- 充電 IC（IP2315）は I2C バスに常時接続しない（公式注意事項）。M1 では触らない。
- 各段の成否をシリアルに出力する。
- **完了条件**：起動ログに全段の OK が出る。失敗した段があっても停止せず、ログを残して続行する。

### T3. Display

```cpp
enum class Refresh { Quality, Text, Fast };   // Fastest は定義しない（R-4）

class Display {
public:
  void begin();
  LGFX_Sprite& canvas();         // PSRAM 上のフレームバッファ（480x800, 4階調）
  void present(Refresh mode);    // 描画反映。必要なら自動で全画面化
  void fullRefresh();            // 明示的な全画面リフレッシュ（カウンタを0に戻す）
  uint8_t partialCount() const;
private:
  uint8_t partialCount_ = 0;
  static constexpr uint8_t kMaxPartial = 10;
};
```

- `present()` の規則：
  - `Quality` は全画面扱いとし、カウンタを 0 にする。
  - `Text` / `Fast` は部分扱いとし、カウンタを +1 する。
  - **カウンタが `kMaxPartial` に達している状態で部分描画が要求されたら、その描画を全画面リフレッシュに置き換えて実行し、カウンタを 0 にする**（＝11回目が全画面になる）。
- 実際のリフレッシュ方式（M5GFX の `setEpdMode` か OTP 系か）は T0 で確認した Nostos の方式に合わせる。
- ログ：`[Display] mode=Text partial=7/10` のように毎回出力する。
- **完了条件**：RefreshTestApp（T7）で、11回目の描画が全画面リフレッシュになることを目視とログで確認できる。

### T4. Input

- ボタン A（G2）・B（G3）：短押しのみ検出し、チャタリング対策を入れる。M5Unified の `BtnA/BtnB` が PaperMono で正しく対応するか確認し、対応しない場合は GPIO を直接読む。
- タッチ（FT6336G）：**Tap のみ**。押下から離すまでが 300ms 以内、移動量が 20px 以内を Tap とする。座標は X:5〜475、Y:5〜795 にクランプする。M5Unified の `M5.Touch` が使えるか確認し、使えない場合は FT6336G（I2C 0x38、INT=G4）を直接読む。
- 画面の向き：ポートレート（480×800）。タッチ座標と描画座標の向きが一致していることを確認する。
- **完了条件**：画面の四隅と中央のタップ座標が、描画座標と ±10px 以内で一致する（RefreshTestApp で確認）。

### T5. Storage

- `Storage::begin()`：TF_EN（M5IOE1 PYG14）を ON → TF_DET（PYG1）で挿入確認 → SDMMC 4bit でマウント。4bit で失敗した場合は 1bit で再試行し、どちらで成功したかをログに出す。
- API：`bool available()`、`bool exists(path)`、`File open(path)`、`size_t readAll(path, buf, max)`
- SD が無い、またはマウントに失敗しても **起動を止めない**。`available()==false` として続行する。
- 計測：`/alea/bench.bin`（100KB 程度、無ければ作成）の読込時間をログに出す（DESIGN.md の読込速度見積もりの検証用）。
- **完了条件**：SdCheckApp（T8）でマウント方式、空き容量、`/alea/` 配下の一覧、読込時間が表示される。SD を抜いた状態でもクラッシュしない。

### T6. App / AppManager / Launcher

- `App.h` は DESIGN.md §5 のとおり（Shake 系イベントの定義は M1 でも置いておく）。
- `AppManager`：
  - `registerApp(App*, bool requiresSd)` でアプリを登録する。
  - ボタン A は **常に AppManager が横取り** し、Launcher 以外で押された場合は Launcher に戻る。Launcher 上では無視する。
  - アプリ切替時は `onExit()` → `display.fullRefresh()` → `onEnter()` の順で呼ぶ。
- `LauncherApp`：
  - 見出し帯（高さ 56px）に「Alea」を表示する。
  - 本体は 2列×5段のタイル（DESIGN.md §9）。M1 では登録済みのアプリだけを並べ、残りの枠は空にする。
  - `requiresSd==true` のアプリは、SD が無い場合に明灰色で表示しタップを無効にする。
  - タイルのアイコンは M1 では不要（名称テキストのみ）。
  - 描画は `Refresh::Text`。
- **完了条件**：ランチャー → 各ダミーアプリ → ボタン A でランチャーへ戻る、を10往復しても表示崩れやクラッシュがない。

### T7. RefreshTestApp（ダミー1：`debug_refresh`）

- 画面中央にカウンタ（描画回数）と `partialCount()` を表示する。
- タップするたびに `Refresh::Text` で再描画する（連打は 500ms 間隔に制限し、R-2 を守る）。
- タップ位置に小さな十字を描き、座標も表示する（T4 の検証を兼ねる）。
- ボタン B で `fullRefresh()` を実行する。

### T8. SdCheckApp（ダミー2：`debug_sd`、requiresSd=true）

- マウント方式（4bit / 1bit）、総容量と空き容量、`/alea/` 配下の一覧（最大20件）、bench.bin の読込時間を表示する。
- ボタン B で再スキャンする。

---

## 5. 受け入れ基準（M1 完了の定義）

- [ ] ビルドが警告なしで通る（ライブラリ由来の警告は除く）
- [ ] 起動からランチャー表示まで、全初期化段が OK でログに出る
- [ ] **11回目の部分描画が自動で全画面リフレッシュになる**（ログと目視で確認）
- [ ] `epd_fastest` 相当のモードがコード上に存在しない
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
- 新しいアプリは `apps/<id>/` に置き、`App` を継承して `main.cpp` で登録する。
- ログの接頭辞は `[Board] [Display] [Input] [Storage] [AppMgr] [<AppId>]` に統一する。
- 実機でしか確認できない事項は、推測で「完了」にせず、確認手順を書いて総司さんに依頼する。

---

## 7. 報告してほしいこと（M1 完了時）

1. 受け入れ基準のチェック結果
2. 採用した描画方式（D-1 の結論）と、その理由
3. タッチ・ボタンを M5Unified 経由で扱えたか、直接制御になったか
4. SD のマウント方式（4bit / 1bit）と bench.bin の読込時間の実測値
5. LoRa 電源の既定状態と、R-5 の対応内容
6. M2（Shake・Rng・yesno / coin / dice）に着手する前に決めておくべき事項

### 参考

- PaperMono 公式ドキュメント（PinMap・注意事項）：https://docs.m5stack.com/en/core/PaperMono
- PaperMono Arduino クイックスタート：https://docs.m5stack.com/en/arduino/papermono/program
- M5PM1 / M5IOE1 電源管理：https://docs.m5stack.com/en/arduino/papermono/m5pm1_m5ioe1
- PaperMono OTP サンプル：https://github.com/m5stack/M5PaperMono-OTP-Demo
- 工場出荷ファーム（初期化手順の参考）：https://github.com/m5stack/M5PaperMono-UserDemo
