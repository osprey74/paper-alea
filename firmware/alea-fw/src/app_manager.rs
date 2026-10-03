//! AppManager — アプリの登録と切替（DESIGN.md §4〜§5、HANDOFF T6）。
//!
//! - アプリは [`AnyApp`] に静的に登録する（`no_std`・ヒープ無しのため `dyn` を使わない）。
//! - ボタン A は常にここで横取りする。ランチャー以外なら戻り、ランチャー上では無視する。
//! - 切替手順：`on_exit` → `on_enter`（描画のみ）→ 全面更新 → `on_ready`。

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

    /// イベントを現在のアプリに配送する。ボタン A は横取りする。
    pub async fn handle(&mut self, ctx: &mut Ctx, e: Event) {
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
        if let Action::Open(i) = action {
            if i < APP_COUNT {
                self.switch(ctx, Some(i)).await;
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
