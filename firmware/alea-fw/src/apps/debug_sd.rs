//! SdCheckApp（`debug_sd`・HANDOFF T8・SD 必須）。
//!
//! マウント方式・総容量と空き容量・`/alea/` の一覧（最大 20 件）・100KB の読込時間を表示する。
//! ボタン B で再スキャン（走査回数「scan #N」を表示）。空き容量の取得は大容量カードで十数秒かかることがあるため、
//! 全面更新で「Scanning...」を出してから走査する（`on_ready`）。

use core::fmt::Write as _;

use embassy_time::Instant;
use esp_println::println;

use crate::services::app::{Action, App, Ctx, Event};
use crate::services::display::Refresh;
use crate::services::storage::{Capacity, DirItem};
use crate::ui::widgets::{self, Lines};
use crate::ui::FmtBuf;

/// `/alea` の一覧に表示する最大項目数。
const MAX_ITEMS: usize = 20;
/// Alea のデータディレクトリ。
const ALEA_DIR: &str = "alea";
/// 読込速度計測用のファイル（無ければ作る）。
const BENCH_PATH: &str = "alea/bench.bin";
/// 読込速度計測用ファイルの大きさ（100KB）。
const BENCH_BYTES: u64 = 100 * 1024;
const MB: u64 = 1024 * 1024;

/// SD 確認。
pub struct SdCheckApp {
    /// 走査した回数（再スキャンが効いたことを画面で分かるようにする）。
    scans: u32,
    scanned: bool,
    capacity: Option<Capacity>,
    /// ベンチ読込（バイト数, 所要 ms）。
    bench: Option<(u64, u64)>,
    items: [DirItem; MAX_ITEMS],
    n_items: Option<usize>,
}

impl SdCheckApp {
    /// 作る。
    pub const fn new() -> Self {
        Self {
            scans: 0,
            scanned: false,
            capacity: None,
            bench: None,
            items: [DirItem::EMPTY; MAX_ITEMS],
            n_items: None,
        }
    }

    async fn scan(&mut self, ctx: &mut Ctx) {
        let storage = ctx.storage();
        let t0 = Instant::now();
        self.capacity = storage.capacity().await;
        match self.capacity {
            Some(c) => println!(
                "[debug_sd] capacity total={}MB free={}MB (took {}ms)",
                c.total / MB,
                c.free / MB,
                t0.elapsed().as_millis()
            ),
            None => println!("[debug_sd] capacity FAILED"),
        }
        let ready = storage.ensure_file(ALEA_DIR, BENCH_PATH, BENCH_BYTES).await;
        println!("[debug_sd] {} ready={}", BENCH_PATH, ready as u8);
        let t0 = Instant::now();
        self.bench = storage
            .read_discard(BENCH_PATH)
            .await
            .map(|n| (n, t0.elapsed().as_millis()));
        match self.bench {
            Some((n, ms)) => println!("[debug_sd] bench read {} bytes in {}ms", n, ms),
            None => println!("[debug_sd] bench read FAILED"),
        }
        self.n_items = storage.list(ALEA_DIR, &mut self.items).await;
        // exists / read_all の動作確認（ベンチファイルの先頭は 0,1,2,3…）。
        let exists = storage.exists(BENCH_PATH).await;
        let mut head = [0u8; 4];
        let n = storage.read_all(BENCH_PATH, &mut head).await;
        println!("[debug_sd] exists={} read_all head={:?} n={:?}", exists as u8, head, n);
        self.scans += 1;
        self.scanned = true;
    }

    fn draw(&self, ctx: &mut Ctx) {
        let available = ctx.storage().available();
        let detected = ctx.storage().detected();
        let bus = ctx.storage().bus_width();
        ctx.clear();
        let mut canvas = ctx.canvas();
        widgets::header(&mut canvas, "SD check", true);
        let mut lines = Lines::below_header();
        let mut line = FmtBuf::<48>::new();
        let _ = write!(line, "B: rescan   (scan #{})", self.scans);
        lines.put(&mut canvas, line.as_str());
        line.clear();
        lines.gap(8);
        let det = match detected {
            Some(true) => "inserted",
            Some(false) => "empty",
            None => "unknown",
        };
        let _ = write!(line, "TF_DET: {det}");
        lines.put(&mut canvas, line.as_str());
        line.clear();
        let _ = write!(line, "mount: {} {}", bus, if available { "OK" } else { "FAILED" });
        lines.put(&mut canvas, line.as_str());
        line.clear();
        if !self.scanned {
            lines.gap(8);
            lines.put(&mut canvas, "Scanning...");
            return;
        }
        match self.capacity {
            Some(c) => {
                let _ = write!(line, "total {} MB / free {} MB", c.total / MB, c.free / MB);
            }
            None => {
                let _ = write!(line, "capacity: -");
            }
        }
        lines.put(&mut canvas, line.as_str());
        line.clear();
        match self.bench {
            Some((n, ms)) => {
                let kbs = if ms > 0 { n * 1000 / ms / 1024 } else { 0 };
                let _ = write!(line, "read {}KB: {} ms ({} KB/s)", n / 1024, ms, kbs);
            }
            None => {
                let _ = write!(line, "read: -");
            }
        }
        lines.put(&mut canvas, line.as_str());
        line.clear();
        lines.gap(8);
        lines.put(&mut canvas, "/alea/");
        match self.n_items {
            Some(0) => lines.put(&mut canvas, "  (empty)"),
            Some(n) => {
                for item in &self.items[..n] {
                    if item.is_dir {
                        let _ = write!(line, "  {}/", item.name());
                    } else {
                        let _ = write!(line, "  {}  {}", item.name(), item.size);
                    }
                    lines.put(&mut canvas, line.as_str());
                    line.clear();
                }
            }
            None => lines.put(&mut canvas, "  (not found)"),
        }
    }
}

impl App for SdCheckApp {
    fn id(&self) -> &'static str {
        "debug_sd"
    }

    fn title(&self) -> &'static str {
        "SD check"
    }

    fn requires_sd(&self) -> bool {
        true
    }

    async fn on_enter(&mut self, ctx: &mut Ctx) {
        self.scans = 0;
        self.scanned = false;
        self.draw(ctx);
    }

    async fn on_ready(&mut self, ctx: &mut Ctx) {
        self.scan(ctx).await;
        self.draw(ctx);
        ctx.present(Refresh::Partial).await;
    }

    async fn on_event(&mut self, ctx: &mut Ctx, e: Event) -> Action {
        if e == Event::ButtonB {
            println!("[debug_sd] rescan");
            self.scan(ctx).await;
            self.draw(ctx);
            ctx.present(Refresh::Partial).await;
        }
        Action::None
    }
}
