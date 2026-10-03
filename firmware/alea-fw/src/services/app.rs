//! アプリの共通インターフェース（DESIGN.md §5）。
//!
//! - アプリは [`Ctx`] の公開 API（描画・反映・ストレージ）だけを使い、HAL を直接呼ばない。
//! - ボタン A は [`crate::app_manager::AppManager`] が横取りし、アプリには配送しない。
//! - アプリ切替の手順：`on_exit` → 画面を消して `on_enter`（描画のみ）→ 全面更新 → `on_ready`。
//!   HANDOFF の「`on_exit` → 全面更新 → `on_enter`」では古い画面を全面更新で描き直してしまうため、
//!   新しい画面を描いてから全面更新する順にした。

use crate::board::ioe::SysI2c;
use crate::board::power;
use crate::services::display::{Canvas, Display, Refresh};
use crate::services::input::{Input, InputEvent};
use crate::services::rng::Rng;
use crate::services::shake::Shake;
use crate::services::storage::Storage;
use alea_core::shake::ShakeEvent;

/// アプリに配送するイベント。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    /// 振り始め（画面は変えない・アニメーションなし方針）。
    ShakeStart,
    /// 振り終わり（ここで結果を確定する）。
    ShakeEnd,
    /// タップ（ページ座標）。
    Tap {
        /// x [px]。
        x: i16,
        /// y [px]。
        y: i16,
    },
    /// ボタン A（AppManager が横取りするため、アプリには届かない）。
    ButtonA,
    /// ボタン B。
    ButtonB,
    /// 電源ボタン（AppManager が横取りして電源オフの手順に入る。アプリには届かない）。
    PowerButton,
}

impl From<InputEvent> for Event {
    fn from(e: InputEvent) -> Self {
        match e {
            InputEvent::ButtonA => Event::ButtonA,
            InputEvent::ButtonB => Event::ButtonB,
            InputEvent::Tap { x, y } => Event::Tap { x, y },
        }
    }
}

/// イベント処理の結果、AppManager に求めること。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// 何もしない。
    None,
    /// 登録順 `index` のアプリを開く（ランチャーだけが使う）。
    Open(usize),
}

/// ミニアプリ。
#[allow(async_fn_in_trait)] // 単一タスク・静的ディスパッチでのみ使う。
pub trait App {
    /// ID（`"tarot"` など）。
    fn id(&self) -> &'static str;
    /// ランチャーに出す名称。
    fn title(&self) -> &'static str;
    /// microSD が必要か（SD が無いとランチャーで選べない）。
    fn requires_sd(&self) -> bool {
        false
    }
    /// 初期画面を 4 階調（`Refresh::Gray`）で出すか。false ならモノクロの全面更新。
    fn enter_gray(&self) -> bool {
        false
    }
    /// 起動時：状態を初期化し、初期画面を描く（反映は AppManager が全面更新で行う）。
    async fn on_enter(&mut self, ctx: &mut Ctx);
    /// 初期画面の全面更新が終わった後の処理（SD の読み込みなど）。
    async fn on_ready(&mut self, _ctx: &mut Ctx) {}
    /// イベント処理。描き直す場合は自分で `ctx.present` を呼ぶ。
    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action;
    /// 終了時。
    async fn on_exit(&mut self, _ctx: &mut Ctx) {}
}

/// アプリに渡す Core Services 一式。
pub struct Ctx {
    display: Display,
    i2c: SysI2c,
    storage: Storage,
    input: Input,
    shake: Shake,
    rng: Rng,
    /// メインループの周期を数える（入力は 2 周期に 1 回、電源ボタンは 10 周期に 1 回調べる）。
    tick: u32,
    /// 振りの合図の LED（緑）を点けているか。
    shake_led: bool,
}

impl Ctx {
    /// 初期化済みのサービスから作る。
    pub fn new(
        display: Display,
        i2c: SysI2c,
        storage: Storage,
        input: Input,
        shake: Shake,
        rng: Rng,
    ) -> Self {
        Self {
            display,
            i2c,
            storage,
            input,
            shake,
            rng,
            tick: 0,
            shake_led: false,
        }
    }

    /// フレームバッファを白で埋める。
    pub fn clear(&mut self) {
        self.display.clear();
    }

    /// 描画先。
    pub fn canvas(&mut self) -> Canvas<'_> {
        self.display.canvas()
    }

    /// 前回の全面更新からの部分更新回数。
    pub fn partial_count(&self) -> u8 {
        self.display.partial_count()
    }

    /// 画面に反映する（部分更新の上限に達していれば自動で全面）。描画中の操作は捨てる。
    pub async fn present(&mut self, mode: Refresh) {
        self.display.present(&mut self.i2c, mode).await;
        self.resync();
    }

    /// 明示的な全面更新。描画中の操作は捨てる。
    pub async fn full_refresh(&mut self) {
        self.display.full_refresh(&mut self.i2c).await;
        self.resync();
    }

    /// 描画中に起きた操作・揺れを捨てる。振りの合図の LED も消す。
    fn resync(&mut self) {
        self.input.resync();
        self.shake.resync();
        self.set_shake_led(false);
    }

    /// 振りの合図の LED（緑）。振り始めで点け、結果を描き終えたら消す（2026-10-03 決定）。
    pub(crate) fn set_shake_led(&mut self, on: bool) {
        if self.shake_led != on {
            self.shake_led = on;
            power::set_led(&mut self.i2c, on, false);
        }
    }

    /// USB から給電されているか（AppManager の電源オフの手順で使う）。
    pub(crate) fn usb_present(&mut self) -> bool {
        power::usb_present(&mut self.i2c)
    }

    /// 前回から電源ボタンが押されたか（AppManager の電源オフの手順で使う）。
    pub(crate) fn power_button(&mut self) -> bool {
        power::button_pressed(&mut self.i2c)
    }

    /// 電源を切る（USB 給電中は切れずに起動し直す）。
    pub(crate) fn shutdown(&mut self) {
        power::shutdown(&mut self.i2c);
    }

    /// 乱数源。
    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// microSD。
    pub fn storage(&mut self) -> &mut Storage {
        &mut self.storage
    }

    /// 入力と揺れを 1 周期分処理する（メインループ専用・[`POLL_MS`] ごと。アプリからは呼ばない）。
    /// 揺れは毎周期（約 100Hz）、ボタンとタッチは 2 周期に 1 回（入力側の周期 20ms）調べる。
    pub(crate) fn poll(&mut self) -> Option<Event> {
        self.tick = self.tick.wrapping_add(1);
        if let Some(e) = self.shake.poll(&mut self.i2c) {
            return Some(match e {
                ShakeEvent::Start => {
                    self.set_shake_led(true);
                    Event::ShakeStart
                }
                ShakeEvent::End => Event::ShakeEnd,
            });
        }
        if self.tick % 10 == 0 && power::button_pressed(&mut self.i2c) {
            return Some(Event::PowerButton);
        }
        if self.tick % 2 == 0 {
            return self.input.poll(&mut self.i2c).map(Event::from);
        }
        None
    }
}

/// メインループの周期 [ms]（BMI270 の 100Hz に合わせる）。
pub const POLL_MS: u64 = 10;
