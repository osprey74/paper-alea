//! AppManager — アプリの登録と切替（DESIGN.md §4〜§5、HANDOFF T6）。
//!
//! - アプリは [`AnyApp`] に静的に登録する（`no_std`・ヒープ無しのため `dyn` を使わない）。
//! - ボタン A は常にここで横取りする。ランチャー以外なら戻り、ランチャー上では無視する。
//! - 切替手順：`on_exit` → `on_enter`（描画のみ）→ 全面更新 → `on_ready`。
//! - 電源ボタンも常にここで横取りし、終了画面を描いて電源を切る（USB 給電中は終了画面で待つ）。

use embassy_time::Timer;
use esp_println::println;

use crate::apps::amida::AmidaApp;
use crate::apps::coin::CoinApp;
use crate::apps::debug_refresh::RefreshTestApp;
use crate::apps::debug_sd::SdCheckApp;
use crate::apps::dice::DiceApp;
use crate::apps::iching::IchingApp;
use crate::apps::launcher::{LauncherApp, Slot, TileInfo};
use crate::apps::omikuji::OmikujiApp;
use crate::apps::rune::RuneApp;
use crate::apps::stick::StickApp;
use crate::apps::tarot::TarotApp;
use crate::apps::yesno::YesNoApp;
use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::ui::layout::SCREEN_W;

/// 登録アプリ（ランチャーを除く）。新しいアプリはここに足す。
enum AnyApp {
    Tarot(TarotApp),
    Iching(IchingApp),
    Rune(RuneApp),
    Stick(StickApp),
    Amida(AmidaApp),
    Omikuji(OmikujiApp),
    Dice(DiceApp),
    Coin(CoinApp),
    YesNo(YesNoApp),
    RefreshTest(RefreshTestApp),
    SdCheck(SdCheckApp),
}

/// `AnyApp` の各アプリへ同じ呼び出しを振り分ける。
macro_rules! dispatch {
    ($self:expr, $app:ident => $body:expr) => {
        match $self {
            AnyApp::Tarot($app) => $body,
            AnyApp::Iching($app) => $body,
            AnyApp::Rune($app) => $body,
            AnyApp::Stick($app) => $body,
            AnyApp::Amida($app) => $body,
            AnyApp::Omikuji($app) => $body,
            AnyApp::Dice($app) => $body,
            AnyApp::Coin($app) => $body,
            AnyApp::YesNo($app) => $body,
            AnyApp::RefreshTest($app) => $body,
            AnyApp::SdCheck($app) => $body,
        }
    };
}

impl AnyApp {
    fn info(&self) -> TileInfo {
        dispatch!(self, a => TileInfo { title: a.title(), requires_sd: a.requires_sd() })
    }

    fn id(&self) -> &'static str {
        dispatch!(self, a => a.id())
    }

    fn enter_gray(&self) -> bool {
        dispatch!(self, a => a.enter_gray())
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        dispatch!(self, a => a.on_enter(ctx).await)
    }

    async fn on_ready(&mut self, ctx: &mut Ctx) {
        dispatch!(self, a => a.on_ready(ctx).await)
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        dispatch!(self, a => a.on_event(ctx, e).await)
    }

    async fn on_exit(&mut self, ctx: &mut Ctx) {
        dispatch!(self, a => a.on_exit(ctx).await)
    }
}

/// 終了画面（`tools/render_sleep.py` が生成・480×800・2bit/画素）。電池のとき。
static SLEEP_OFF: &[u8] = include_bytes!("../assets/sleep_off.2bpp");
/// 終了画面。USB 給電中の待機。
static SLEEP_USB: &[u8] = include_bytes!("../assets/sleep_usb.2bpp");
/// USB 給電中の待機で、電源ボタンと USB を調べる周期 [ms]。
const STANDBY_POLL_MS: u64 = 100;
/// シャットダウンを指示してから、切れなかったと判断するまで [ms]。
const SHUTDOWN_WAIT_MS: u64 = 2000;

/// 登録アプリ数。
const APP_COUNT: usize = 11;

/// ランチャーのタイル I〜IX に置くアプリの ID（DESIGN.md §1 の収録順）。未登録の ID は未実装として薄く表示する。
const MAIN_TILE_IDS: [&str; 9] = [
    "tarot", "iching", "rune", "dice", "coin", "stick", "amida", "omikuji", "yesno",
];

/// アプリの登録と切替。
pub struct AppManager {
    launcher: LauncherApp,
    apps: [AnyApp; APP_COUNT],
    /// 表示中のアプリ（`None` = ランチャー）。
    current: Option<usize>,
}

impl AppManager {
    /// アプリを登録する。
    pub fn new() -> Self {
        let apps = [
            AnyApp::Tarot(TarotApp::new()),
            AnyApp::Iching(IchingApp::new()),
            AnyApp::Rune(RuneApp::new()),
            AnyApp::Stick(StickApp::new()),
            AnyApp::Amida(AmidaApp::new()),
            AnyApp::Omikuji(OmikujiApp::new()),
            AnyApp::Dice(DiceApp::new()),
            AnyApp::Coin(CoinApp::new()),
            AnyApp::YesNo(YesNoApp::new()),
            AnyApp::RefreshTest(RefreshTestApp::new()),
            AnyApp::SdCheck(SdCheckApp::new()),
        ];
        let slot = |k: usize| Slot {
            app: k,
            info: apps[k].info(),
        };
        let main = MAIN_TILE_IDS.map(|id| apps.iter().position(|a| a.id() == id).map(slot));
        // 開発用ページ：ID が debug_ で始まるアプリを登録順に並べる。
        let mut dev = [None; crate::ui::layout::TILE_COUNT];
        let mut n = 0;
        for (k, a) in apps.iter().enumerate() {
            if a.id().starts_with("debug_") && n < dev.len() {
                dev[n] = Some(slot(k));
                n += 1;
            }
        }
        Self {
            launcher: LauncherApp::new(main, dev),
            apps,
            current: None,
        }
    }

    /// ランチャーを表示する（起動時に 1 回）。
    pub async fn start(&mut self, ctx: &mut Ctx) {
        self.enter(ctx, None).await;
    }

    /// イベントを現在のアプリに配送する。ボタン A と電源ボタンは横取りする。
    pub async fn handle(&mut self, ctx: &mut Ctx, e: Event) {
        if e == Event::PowerButton {
            self.power_off(ctx).await;
            return;
        }
        if e == Event::ButtonA {
            if self.current.is_some() {
                self.switch(ctx, None).await;
            }
            return;
        }
        let action = match self.current {
            None => self.launcher.on_event(ctx, e).await,
            Some(i) => self.apps[i].on_event(ctx, e).await,
        };
        if e == Event::ShakeEnd {
            // 振りの合図の LED は、結果を描き終えたら（描かないアプリでも）消す。
            ctx.set_shake_led(false);
        }
        if let Action::Open(i) = action {
            if i < APP_COUNT {
                self.switch(ctx, Some(i)).await;
            }
        }
    }

    /// 電源ボタンが押された：終了画面（書物調の表紙・4 階調）を描いて電源を切る。
    /// USB 給電中は切っても起動し直すため、待機用の終了画面を出して待ち、
    /// もう一度押されたらランチャーに戻る。待機中に USB が抜かれたら電源オフの画面にして切る。
    async fn power_off(&mut self, ctx: &mut Ctx) {
        println!("[AppMgr] power button");
        match self.current {
            None => self.launcher.on_exit(ctx).await,
            Some(i) => self.apps[i].on_exit(ctx).await,
        }
        loop {
            let usb = ctx.usb_present();
            ctx.clear();
            let image = if usb { SLEEP_USB } else { SLEEP_OFF };
            ctx.canvas().blit_2bpp(0, 0, SCREEN_W, 800, image);
            ctx.present(Refresh::Gray).await;
            // 描画中の押下は捨てる。
            let _ = ctx.power_button();
            if !usb {
                println!("[AppMgr] shutdown");
                ctx.shutdown();
                // 切れなかった（直前に USB が挿された等）ときは、待機の画面からやり直す。
                Timer::after_millis(SHUTDOWN_WAIT_MS).await;
                continue;
            }
            println!("[AppMgr] usb present, standby");
            loop {
                Timer::after_millis(STANDBY_POLL_MS).await;
                if ctx.power_button() {
                    println!("[AppMgr] resume");
                    self.enter(ctx, None).await;
                    return;
                }
                if !ctx.usb_present() {
                    break;
                }
            }
        }
    }

    async fn switch(&mut self, ctx: &mut Ctx, to: Option<usize>) {
        match self.current {
            None => self.launcher.on_exit(ctx).await,
            Some(i) => self.apps[i].on_exit(ctx).await,
        }
        self.enter(ctx, to).await;
    }

    async fn enter(&mut self, ctx: &mut Ctx, to: Option<usize>) {
        let name = to.map_or(self.launcher.id(), |i| self.apps[i].id());
        println!("[AppMgr] enter {}", name);
        self.current = to;
        ctx.clear();
        match to {
            None => self.launcher.on_enter(ctx).await,
            Some(i) => self.apps[i].on_enter(ctx).await,
        }
        let gray = to.map_or(self.launcher.enter_gray(), |i| self.apps[i].enter_gray());
        if gray {
            ctx.present(Refresh::Gray).await;
        } else {
            ctx.full_refresh().await;
        }
        match to {
            None => self.launcher.on_ready(ctx).await,
            Some(i) => self.apps[i].on_ready(ctx).await,
        }
    }
}
