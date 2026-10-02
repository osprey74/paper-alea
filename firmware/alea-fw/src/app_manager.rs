//! AppManager — アプリの登録と切替（DESIGN.md §4〜§5、HANDOFF T6）。
//!
//! - アプリは [`AnyApp`] に静的に登録する（`no_std`・ヒープ無しのため `dyn` を使わない）。
//! - ボタン A は常にここで横取りする。ランチャー以外なら戻り、ランチャー上では無視する。
//! - 切替手順：`on_exit` → `on_enter`（描画のみ）→ 全面更新 → `on_ready`。

use esp_println::println;

use crate::apps::debug_refresh::RefreshTestApp;
use crate::apps::debug_sd::SdCheckApp;
use crate::apps::launcher::{LauncherApp, TileInfo};
use crate::services::app::{Action, App, Ctx, Event};

/// 登録アプリ（ランチャーを除く）。新しいアプリはここに足す。
enum AnyApp {
    RefreshTest(RefreshTestApp),
    SdCheck(SdCheckApp),
}

/// `AnyApp` の各アプリへ同じ呼び出しを振り分ける。
macro_rules! dispatch {
    ($self:expr, $app:ident => $body:expr) => {
        match $self {
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
const APP_COUNT: usize = 2;

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
            AnyApp::RefreshTest(RefreshTestApp::new()),
            AnyApp::SdCheck(SdCheckApp::new()),
        ];
        let infos = [apps[0].info(), apps[1].info()];
        Self {
            launcher: LauncherApp::new(&infos),
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
        ctx.full_refresh().await;
        match to {
            None => self.launcher.on_ready(ctx).await,
            Some(i) => self.apps[i].on_ready(ctx).await,
        }
    }
}
