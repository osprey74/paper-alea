//! microSD（SDHOST 1bit + FAT32）。
//!
//! Nostos `firmware/nostos-fw/src/sdlog.rs`（MIT）のカード初期化とマウント手順を流用し、
//! 読み込み・一覧・容量取得・ベンチ用ファイル作成を加えた。
//! - `esp-hal::sdmmc` SDHOST スロット 1・1bit（CLK=GPIO13 / CMD=GPIO12 / DAT0=GPIO11）
//! - `sdio` がカード初期化、`embedded-fatfs`（`lfn` 有効）が FAT、`embedded-partitions` が MBR
//!
//! 操作のたびに FAT をマウント／アンマウントする（Nostos と同じ。カードを抜かれても状態が残らない）。

use block_device_adapters::BufStream;
use embassy_time::Delay;
use embedded_fatfs::{FileSystem, FsOptions, ReadWriteSeek};
use embedded_io_async::{Read, Write};
use embedded_partitions::mbr::Scheme;
use esp_hal::sdmmc::{Config, DelayPhase, SdHostController, SlotConfig};
use esp_hal::Async;
use sdio::{sd::Card, BlockDevice};
use static_cell::StaticCell;

/// カードクロック（Nostos と同じ 20MHz。初期化は sdio が低速で行う）。
const CARD_HZ: u32 = 20_000_000;
/// S3 高速パスの入力サンプリング位相。
const INPUT_DELAY_PHASE: DelayPhase = DelayPhase::_0;

/// 一覧の 1 項目に保持するファイル名の最大バイト数（超えた分は切り捨て）。
pub const NAME_MAX: usize = 24;

type RawCard = BlockDevice<Card, esp_hal::sdmmc::Slot<'static, 1, Async>, Delay, 512>;
type SdCard = RamBounce<RawCard>;

/// 書き込むデータを必ず RAM 上のバッファに写してから SD に渡すラッパー。
///
/// embedded-fatfs はクラスタのゼロ埋めに `const ZEROS: [u8; 512]`（フラッシュ上の定数）を使い、
/// BufStream はアラインメントの合ったバッファをそのまま SD に渡す。SDHOST の DMA はフラッシュを
/// 読めないため、ディレクトリ作成などのゼロ埋めが `Io` エラーで失敗していた（2026-10-03 実機で特定）。
/// Nostos はゼロ埋めの起きない追記しかしていなかったため表面化しなかった。
pub struct RamBounce<D>(D);

impl<D: block_device_driver::BlockDevice<512>> block_device_driver::BlockDevice<512>
    for RamBounce<D>
{
    type Error = D::Error;
    type Align = D::Align;

    async fn read(
        &mut self,
        block_address: u32,
        data: &mut [aligned::Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        self.0.read(block_address, data).await
    }

    async fn write(
        &mut self,
        block_address: u32,
        data: &[aligned::Aligned<Self::Align, [u8; 512]>],
    ) -> Result<(), Self::Error> {
        let mut bounce = [aligned::Aligned::<Self::Align, _>([0u8; 512])];
        for (i, block) in data.iter().enumerate() {
            bounce[0].copy_from_slice(&block[..]);
            self.0.write(block_address + i as u32, &bounce).await?;
        }
        Ok(())
    }

    async fn size(&mut self) -> Result<u64, Self::Error> {
        self.0.size().await
    }
}

/// ディレクトリ一覧の 1 項目。
#[derive(Clone, Copy)]
pub struct DirItem {
    name: [u8; NAME_MAX],
    name_len: u8,
    /// ファイルサイズ [バイト]（ディレクトリは 0）。
    pub size: u64,
    /// ディレクトリか。
    pub is_dir: bool,
}

impl DirItem {
    /// 空の項目。
    pub const EMPTY: Self = Self {
        name: [0; NAME_MAX],
        name_len: 0,
        size: 0,
        is_dir: false,
    };

    /// ファイル名（ASCII 以外は `?` に置き換え済み）。
    pub fn name(&self) -> &str {
        core::str::from_utf8(&self.name[..usize::from(self.name_len)]).unwrap_or("?")
    }
}

/// 容量 [バイト]。
#[derive(Clone, Copy)]
pub struct Capacity {
    /// 総容量。
    pub total: u64,
    /// 空き容量。
    pub free: u64,
}

/// 1 回のマウントで行う操作と、その結果の書き込み先。
enum Op<'a> {
    Capacity(&'a mut Option<Capacity>),
    Exists(&'a str, &'a mut bool),
    Read(&'a str, &'a mut [u8], &'a mut Option<usize>),
    /// 中身を捨てながら全体を読み、読めたバイト数を返す（ベンチ用）。
    ReadDiscard(&'a str, &'a mut Option<u64>),
    List(&'a str, &'a mut [DirItem], &'a mut Option<usize>),
    /// `dir` が無ければ作り、`path` が `size` バイトでなければ作り直す。
    Ensure(&'a str, &'a str, u64, &'a mut bool),
}

/// 初期化済みの microSD カード。
pub struct Sd {
    card: SdCard,
}

impl Sd {
    /// SDHOST スロット 1（1bit）でカードを初期化する。カード無し・失敗は `None`。
    /// SD 電源（IOE1 PYG14）は呼び出し側で投入しておく。起動時に 1 回だけ呼ぶ。
    pub async fn init(
        sdhost: esp_hal::peripherals::SDHOST<'static>,
        clk: esp_hal::peripherals::GPIO13<'static>,
        cmd: esp_hal::peripherals::GPIO12<'static>,
        dat0: esp_hal::peripherals::GPIO11<'static>,
    ) -> Option<Self> {
        // slot は controller を借用するため、StaticCell で 'static 化する。
        static SDHOST_CTRL: StaticCell<SdHostController<'static>> = StaticCell::new();
        let controller = SDHOST_CTRL.init(SdHostController::new(sdhost, Config::default()).ok()?);
        let slot = controller
            .slot::<1>(SlotConfig::default().with_input_delay_phase(INPUT_DELAY_PHASE))
            .ok()?;
        let slot = slot.with_clk(clk).with_cmd(cmd).with_data0(dat0).into_async();
        let card = BlockDevice::new_sd_card(slot, CARD_HZ, Delay).await.ok()?;
        Some(Self {
            card: RamBounce(card),
        })
    }

    /// 総容量と空き容量（FAT 全体を走査するため大容量カードでは数秒かかる）。
    pub async fn capacity(&mut self) -> Option<Capacity> {
        let mut out = None;
        self.run(&mut Op::Capacity(&mut out)).await;
        out
    }

    /// `path` にファイルかディレクトリがあるか。
    pub async fn exists(&mut self, path: &str) -> bool {
        let mut out = false;
        self.run(&mut Op::Exists(path, &mut out)).await;
        out
    }

    /// ファイルを `buf` に読み込み、読んだバイト数を返す（`buf` より大きいファイルは先頭だけ）。
    pub async fn read(&mut self, path: &str, buf: &mut [u8]) -> Option<usize> {
        let mut out = None;
        self.run(&mut Op::Read(path, buf, &mut out)).await;
        out
    }

    /// ファイル全体を読み捨て、読めたバイト数を返す（読込速度の計測用）。
    pub async fn read_discard(&mut self, path: &str) -> Option<u64> {
        let mut out = None;
        self.run(&mut Op::ReadDiscard(path, &mut out)).await;
        out
    }

    /// ディレクトリの中身を `out` に詰め、項目数を返す（`.` と `..` は除く）。
    pub async fn list(&mut self, dir: &str, out: &mut [DirItem]) -> Option<usize> {
        let mut n = None;
        self.run(&mut Op::List(dir, out, &mut n)).await;
        n
    }

    /// `dir` を作り、`path` を `size` バイトのファイルにする（既に同じサイズなら何もしない）。
    pub async fn ensure_file(&mut self, dir: &str, path: &str, size: u64) -> bool {
        let mut ok = false;
        self.run(&mut Op::Ensure(dir, path, size, &mut ok)).await;
        ok
    }

    /// パーティションを開いて FAT をマウントし、操作を 1 つ行ってアンマウントする。
    async fn run(&mut self, op: &mut Op<'_>) {
        let stream = BufStream::<_, 512>::new(&mut self.card);
        match Scheme::open(stream).await {
            Ok(Scheme::Mbr(mut mbr)) => {
                let fat_idx = mbr.iter_used().find(|(_, p)| p.is_fat()).map(|(i, _)| i);
                if let Some(idx) = fat_idx {
                    if let Ok(slice) = mbr.open_partition(idx).await {
                        run_on_fs(slice, op).await;
                    }
                }
            }
            Ok(Scheme::Superfloppy(io)) => run_on_fs(io, op).await,
            _ => {}
        }
    }
}

async fn run_on_fs<IO: ReadWriteSeek>(io: IO, op: &mut Op<'_>)
where
    IO::Error: core::fmt::Debug,
{
    let fs = match FileSystem::new(io, FsOptions::new()).await {
        Ok(fs) => fs,
        Err(e) => {
            esp_println::println!("[Storage] mount err={:?}", e);
            return;
        }
    };
    // 操作は別関数にして、途中で抜けても必ず unmount まで到達させる。
    do_op(&fs, op).await;
    let _ = fs.unmount().await;
}

async fn do_op<IO: ReadWriteSeek, TP, OCC>(fs: &FileSystem<IO, TP, OCC>, op: &mut Op<'_>)
where
    IO::Error: core::fmt::Debug,
    TP: embedded_fatfs::TimeProvider,
    OCC: embedded_fatfs::OemCpConverter,
{
    {
        let root = fs.root_dir();
        match op {
            Op::Capacity(out) => {
                if let Ok(st) = fs.stats().await {
                    let cluster = u64::from(st.cluster_size());
                    **out = Some(Capacity {
                        total: u64::from(st.total_clusters()) * cluster,
                        free: u64::from(st.free_clusters()) * cluster,
                    });
                }
            }
            Op::Exists(path, out) => {
                **out = root.open_file(path).await.is_ok() || root.open_dir(path).await.is_ok();
            }
            Op::Read(path, buf, out) => {
                if let Ok(mut f) = root.open_file(path).await {
                    let mut n = 0;
                    while n < buf.len() {
                        match f.read(&mut buf[n..]).await {
                            Ok(0) => break,
                            Ok(k) => n += k,
                            Err(_) => return,
                        }
                    }
                    **out = Some(n);
                }
            }
            Op::ReadDiscard(path, out) => {
                if let Ok(mut f) = root.open_file(path).await {
                    let mut chunk = [0u8; 4096];
                    let mut n: u64 = 0;
                    loop {
                        match f.read(&mut chunk).await {
                            Ok(0) => break,
                            Ok(k) => n += k as u64,
                            Err(_) => return,
                        }
                    }
                    **out = Some(n);
                }
            }
            Op::List(dir, items, out) => {
                let Ok(d) = root.open_dir(dir).await else {
                    return;
                };
                let mut iter = d.iter();
                let mut n = 0;
                while let Some(entry) = iter.next().await {
                    let Ok(e) = entry else { return };
                    let short = e.short_file_name_as_bytes();
                    if short == b"." || short == b".." {
                        continue;
                    }
                    if n >= items.len() {
                        break;
                    }
                    let mut item = DirItem::EMPTY;
                    let mut len = 0;
                    match e.long_file_name_as_ucs2_units() {
                        Some(units) => {
                            for &u in units.iter().take(NAME_MAX) {
                                item.name[len] = if u < 0x80 { u as u8 } else { b'?' };
                                len += 1;
                            }
                        }
                        None => {
                            for &b in short.iter().take(NAME_MAX) {
                                item.name[len] = if b < 0x80 { b } else { b'?' };
                                len += 1;
                            }
                        }
                    }
                    item.name_len = len as u8;
                    item.is_dir = e.is_dir();
                    item.size = if item.is_dir { 0 } else { e.len() };
                    items[n] = item;
                    n += 1;
                }
                **out = Some(n);
            }
            Op::Ensure(dir, path, size, out) => {
                if let Err(e) = root.open_dir(dir).await {
                    esp_println::println!("[Storage] open_dir({}) err={:?}", dir, e);
                    if let Err(e) = root.create_dir(dir).await {
                        esp_println::println!("[Storage] create_dir({}) err={:?}", dir, e);
                        return;
                    }
                }
                let current = match root.open_file(path).await {
                    Ok(mut f) => {
                        let mut chunk = [0u8; 4096];
                        let mut n: u64 = 0;
                        while let Ok(k) = f.read(&mut chunk).await {
                            if k == 0 {
                                break;
                            }
                            n += k as u64;
                        }
                        Some(n)
                    }
                    Err(_) => None,
                };
                if current == Some(*size) {
                    **out = true;
                } else {
                    **out = write_pattern(&root, path, *size).await;
                }
            }
        }
    }
}

/// `path` を作り直して `size` バイトの決まったパターンを書く。
async fn write_pattern<IO: ReadWriteSeek, TP, OCC>(
    root: &embedded_fatfs::Dir<'_, IO, TP, OCC>,
    path: &str,
    size: u64,
) -> bool
where
    TP: embedded_fatfs::TimeProvider,
    OCC: embedded_fatfs::OemCpConverter,
{
    let Ok(mut f) = root.create_file(path).await else {
        return false;
    };
    if f.truncate().await.is_err() {
        return false;
    }
    let mut chunk = [0u8; 512];
    for (i, b) in chunk.iter_mut().enumerate() {
        *b = i as u8;
    }
    let mut left = size;
    while left > 0 {
        let k = left.min(chunk.len() as u64) as usize;
        if f.write_all(&chunk[..k]).await.is_err() {
            return false;
        }
        left -= k as u64;
    }
    f.flush().await.is_ok()
}
