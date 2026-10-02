//! システム I2C 上の M5PM1 / M5IOE1 制御と電源レール bring-up。
//!
//! Nostos `firmware/nostos-fw/src/ioe.rs`（MIT・元は papermono-rs `embassy-debug` の
//! `ioe.rs` / `touch_bus.rs`）から Alea に必要な部分を移植：
//! - M5IOE1 の発見（`0x4F` → `0x6F` フォールバック）と wake プロトコル
//! - push-pull 出力の設定と **IN 読み戻しによる追従確認**（コールドブート固着対策）
//! - M5PM1 の電源保持・電源ボタン誤操作対策
//! - LoRa 電源（PM1 G2）の遮断（R-5）
//!
//! LoRa 無線・RTC RAM の UI 設定・LED・フロントライトは扱わない。

use core::sync::atomic::{AtomicU8, Ordering};

use embassy_time::{Duration, Timer};
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_println::println;
use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::ioe1;
use m5stack_papermono_lite::m5pm1::M5pm1;
use m5stack_papermono_lite::pmic;

/// システムバス用ブロッキング I2C ドライバの別名。
pub type SysI2c = I2c<'static, esp_hal::Blocking>;

/// 実行時に発見された M5IOE1 のアドレス（`0x4F` または `0x6F`）。
static IOE_ADDR: AtomicU8 = AtomicU8::new(addresses::M5IOE1);

/// 電源レール投入から I2C ポーリング開始までの整定待ち。
/// ⚠️ PM1 への書き込みはこの整定後に行う（整定前の書き込みは化けて別レジスタを壊し得る）。
const POWER_SETTLE_MS: u64 = 500;

/// M5IOE1 wake 信号送出からレジスタ読み出しまでの整定待ち。
const WAKE_SETTLE_MS: u64 = 10;

/// 100 kHz バス初期化リトライの間隔。
const INIT_RETRY_MS: u64 = 800;

/// LoRa 電源（`3V3_L2_LoRa`）を制御する PM1 の GPIO 番号（C153 のみ配線）。
const PM1_LORA_EN_GPIO: u8 = 2;

/// [`bring_up`] の結果。
pub struct BringUp {
    /// 発見した M5IOE1 アドレス（`None` なら表示・タッチの電源制御不可）。
    pub ioe_addr: Option<u8>,
    /// PM1 の電源保持設定に成功したか。
    pub power_held: bool,
    /// フル版 PaperMono（`C153`）か。`0x50`（ST25R3916）が ACK すれば C153、NAK なら Lite。
    pub is_c153: bool,
}

/// レジスタポインタ書き込み＋1 バイト読み出しでデバイス存在を確認する。
fn probe_read(i2c: &mut SysI2c, addr: u8, reg: u8) -> bool {
    write_then_read(i2c, addr, reg).is_ok()
}

fn write_then_read(i2c: &mut SysI2c, addr: u8, reg: u8) -> Result<u8, esp_hal::i2c::master::Error> {
    i2c.write(addr, &[reg])?;
    let mut val = [0u8];
    i2c.read(addr, &mut val)?;
    Ok(val[0])
}

/// UID/REV レジスタ読み出しで M5IOE1 の実在を確認する。
fn ident(i2c: &mut SysI2c, addr: u8) -> bool {
    let mut uid = [0u8; 2];
    if i2c.write(addr, &[ioe1::UID_L]).is_err() {
        return false;
    }
    if i2c.read(addr, &mut uid).is_err() {
        return false;
    }
    write_then_read(i2c, addr, ioe1::REV).is_ok()
}

/// エキスパンダ MCU へ wake トランザクションを送る。
fn wake(i2c: &mut SysI2c, addr: u8) {
    let _ = i2c.write(addr, &[ioe1::UID_L]);
}

/// 指定アドレスで M5IOE1 の初期化を試みる（100 kHz → 400 kHz フォールバック）。
async fn try_init_at(i2c: &mut SysI2c, addr: u8) -> bool {
    let hz100 = Config::default();
    let hz400 = Config::default().with_frequency(Rate::from_khz(400));
    let _ = i2c.apply_config(&hz100);

    wake(i2c, addr);
    Timer::after(Duration::from_millis(WAKE_SETTLE_MS)).await;
    if ident(i2c, addr) {
        return true;
    }

    Timer::after(Duration::from_millis(INIT_RETRY_MS)).await;
    wake(i2c, addr);
    Timer::after(Duration::from_millis(WAKE_SETTLE_MS)).await;
    if ident(i2c, addr) {
        return true;
    }

    let _ = i2c.apply_config(&hz400);
    wake(i2c, addr);
    Timer::after(Duration::from_millis(WAKE_SETTLE_MS)).await;
    let ok = ident(i2c, addr);
    let _ = i2c.apply_config(&hz100);
    ok
}

/// 対応アドレス（`0x4F`, `0x6F`）を走査して M5IOE1 を発見・初期化する。
async fn begin_ioe(i2c: &mut SysI2c) -> Option<u8> {
    for &addr in &[addresses::M5IOE1, addresses::M5IOE1_UM] {
        if try_init_at(i2c, addr).await {
            IOE_ADDR.store(addr, Ordering::Relaxed);
            return Some(addr);
        }
    }
    None
}

/// エキスパンダピンを push-pull デジタル出力に設定しレベルを与える（5 回リトライ）。
pub fn set_push_pull_output(
    i2c: &mut SysI2c,
    pyg: u8,
    high: bool,
) -> Result<(), esp_hal::i2c::master::Error> {
    let mut last_err = None;
    for _ in 0..5 {
        match m5stack_papermono_lite::m5ioe1::set_push_pull_output(
            i2c,
            IOE_ADDR.load(Ordering::Relaxed),
            pyg,
            high,
        ) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = Some(e);
                embassy_time::block_for(Duration::from_millis(5));
            }
        }
    }
    match last_err {
        Some(err) => Err(err),
        None => Ok(()),
    }
}

/// M5IOE1 の生レジスタを 1 バイト読む。
fn ioe_read_reg(i2c: &mut SysI2c, reg: u8) -> Option<u8> {
    let mut b = [0u8];
    i2c.write_read(IOE_ADDR.load(Ordering::Relaxed), &[reg], &mut b)
        .ok()
        .map(|_| b[0])
}

/// M5IOE1 の GPIO 関連レジスタを一括ダンプする（コールドブート診断）。
/// MODE(0x03/04)=1 出力 / OUT(0x05/06) / IN(0x07/08)=実ピンレベル / PU(0x09/0A) / PD(0x0B/0C) /
/// DRV(0x13/14)=1 オープンドレイン。IN の bit2=io3(EPD_VDD) / bit4=io5(EPD_RST) を見る。
pub fn dump_ioe_gpio(i2c: &mut SysI2c, tag: &str) {
    let r = |i2c: &mut SysI2c, reg: u8| ioe_read_reg(i2c, reg).unwrap_or(0xFF);
    let (m_l, m_h) = (r(i2c, 0x03), r(i2c, 0x04));
    let (o_l, o_h) = (r(i2c, 0x05), r(i2c, 0x06));
    let (i_l, i_h) = (r(i2c, 0x07), r(i2c, 0x08));
    let (pu_l, pu_h) = (r(i2c, 0x09), r(i2c, 0x0A));
    let (pd_l, pd_h) = (r(i2c, 0x0B), r(i2c, 0x0C));
    let (d_l, d_h) = (r(i2c, 0x13), r(i2c, 0x14));
    println!(
        "[Board] ioe1[{}] mode={:02x}{:02x} out={:02x}{:02x} in={:02x}{:02x} pu={:02x}{:02x} pd={:02x}{:02x} drv={:02x}{:02x}",
        tag, m_h, m_l, o_h, o_l, i_h, i_l, pu_h, pu_l, pd_h, pd_l, d_h, d_l
    );
}

fn ioe_write_reg(i2c: &mut SysI2c, reg: u8, v: u8) -> bool {
    i2c.write(IOE_ADDR.load(Ordering::Relaxed), &[reg, v]).is_ok()
}

fn ioe_in_bit(i2c: &mut SysI2c, pyg: u8) -> u8 {
    let (reg, bit) = if pyg <= 8 { (0x07, pyg - 1) } else { (0x08, pyg - 9) };
    (ioe_read_reg(i2c, reg).unwrap_or(0) >> bit) & 1
}

/// MODE を入力（＋プルアップ）へ一度落としてから元の設定に戻す「ピンキック」。
/// IOE1 はコールド起動直後、MODE=1/OUT=1/DRV=push-pull と登録済みでも**出力ドライバが
/// 有効化されない**ことがある（Nostos 2026-09-16 実機: io3=EPD_VDD_EN が実ピン LOW のまま→
/// パネル無電源→コールドブート固着の真因）。MODE を一度切り替えると駆動が始まる。
fn kick_pin(i2c: &mut SysI2c, pyg: u8) {
    let (m_reg, pu_reg, bit) = if pyg <= 8 {
        (0x03u8, 0x09u8, pyg - 1)
    } else {
        (0x04u8, 0x0Au8, pyg - 9)
    };
    let Some(m) = ioe_read_reg(i2c, m_reg) else { return };
    let Some(pu) = ioe_read_reg(i2c, pu_reg) else { return };
    let _ = ioe_write_reg(i2c, pu_reg, pu | (1 << bit));
    let _ = ioe_write_reg(i2c, m_reg, m & !(1 << bit));
    embassy_time::block_for(Duration::from_millis(5));
    let _ = ioe_write_reg(i2c, pu_reg, pu);
    let _ = ioe_write_reg(i2c, m_reg, m | (1 << bit));
    embassy_time::block_for(Duration::from_millis(2));
}

/// push-pull 出力を設定し、**IN レジスタ（実ピンレベル）で追従を確認**する。追従しなければ
/// [`kick_pin`] で MODE を振り直して再試行（最大 3 回）。電源イネーブル／リセット系のピンに使う。
/// 戻り値は最終的にピンが指定レベルになったか。
pub fn set_output_verified(i2c: &mut SysI2c, pyg: u8, high: bool) -> bool {
    for attempt in 0..3u8 {
        let _ = set_push_pull_output(i2c, pyg, high);
        embassy_time::block_for(Duration::from_millis(2));
        if ioe_in_bit(i2c, pyg) == high as u8 {
            if attempt > 0 {
                println!(
                    "[Board] ioe1 pin{} recovered by kick x{} (want {})",
                    pyg, attempt, high as u8
                );
            }
            return true;
        }
        kick_pin(i2c, pyg);
    }
    println!(
        "[Board] ioe1 pin{} does NOT follow output (want {})",
        pyg, high as u8
    );
    false
}

/// M5IOE1 の入力ピンのレベルを読む（`true` = HIGH）。読み出し失敗は `None`。
pub fn input_level(i2c: &mut SysI2c, pyg: u8) -> Option<bool> {
    let (reg, bit) = if pyg <= 8 { (0x07, pyg - 1) } else { (0x08, pyg - 9) };
    ioe_read_reg(i2c, reg).map(|v| (v >> bit) & 1 == 1)
}

/// FT6336G から第 1 接触点を読む（Nostos `ioe.rs` の `read_touch`）。
///
/// 戻り値は物理フレームバッファ座標（USB 下向き 480×800・M5GFX 準拠）。非接触・読み出し失敗は `None`。
pub fn read_touch(i2c: &mut SysI2c) -> Option<(u16, u16)> {
    use m5stack_papermono_lite::touch;
    const LEN: usize = 1 + (touch::MAX_POINTS as usize) * touch::M5GFX_POINT_BYTES;
    let mut buf = [0u8; LEN];
    if i2c
        .write_read(addresses::FT6336G, &[touch::M5GFX_STATUS_REG], &mut buf)
        .is_err()
    {
        return None;
    }
    let (n, x, y, _x2, _y2) = touch::decode_m5gfx(&buf)?;
    if n == 0 {
        return None;
    }
    Some((x, y))
}

/// M5PM1 の「I2C アイドルスリープ」レジスタ（M5Unified `M5PM1_REG_I2C_CFG`）。
const PM1_REG_I2C_CFG: u8 = 0x09;
/// M5PM1 のウォッチドッグカウンタ（M5Unified `M5PM1_REG_WDT_CNT`）。
const PM1_REG_WDT_CNT: u8 = 0x0A;
/// M5PM1 ボタン設定1。[7]DL_LOCK [6:5]DBL_DLY [4:3]LONG_DLY [2:1]CLK_DLY [0]SINGLE_RST_DIS。
const PM1_REG_BTN_CFG_1: u8 = 0x49;
/// M5PM1 ボタン設定2。[0]DOUBLE_OFF_DIS。
const PM1_REG_BTN_CFG_2: u8 = 0x4A;
/// `BTN_CFG_1` で読み書きするビット群: DL_LOCK(7)+LONG_DLY(4:3)+SINGLE_RST_DIS(0)。
const PM1_BTN_CFG1_MASK: u8 = 0x99;
/// `LONG_DLY=11`（長押し 4 秒）。
const PM1_BTN_LONG_DLY_4S: u8 = 0x18;
/// `SINGLE_RST_DIS`（単クリック・リセット無効）。
const PM1_BTN_SINGLE_RST_DIS: u8 = 1 << 0;
/// `DOUBLE_OFF_DIS`（ダブルクリック電源オフ無効）。
const PM1_BTN_DOUBLE_OFF_DIS: u8 = 1 << 0;
/// M5PM1 ボタン割り込み状態。
const PM1_REG_IRQ_STATUS3: u8 = 0x42;
/// M5PM1 ボタン割り込みマスク（bit=1 で禁止）。
const PM1_REG_IRQ_MASK3: u8 = 0x45;
/// ボタン割り込み全ビット（[2:0]）。
const PM1_BTN_IRQ_ALL: u8 = 0x07;
/// `PWR_CFG` bit1: 5V DCDC 有効（バッテリ駆動時の表示系の電源）。
const PM1_PWR_CFG_DCDC_EN: u8 = 1 << 1;
/// `PWR_CFG` bit2: 3.3V LDO 有効（バッテリ→システムの給電経路）。
const PM1_PWR_CFG_LDO_EN: u8 = 1 << 2;

/// M5PM1 を「生かし続ける」設定（LDO/DCDC 有効・LDO hold・WDT と I2C スリープ無効）。
///
/// バッテリ駆動時の必須処理（USB 給電中は落ちないため気づきにくい）。
/// ⚠️ `PWR_CFG` の BOOST(bit3) は触らない（VIN あり時に ON で 5V レール競合・Nostos 実測）。
fn hold_power(i2c: &mut SysI2c) -> bool {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    let a = pm1.write_at(PM1_REG_I2C_CFG, 0x00).is_ok();
    let b = pm1.write_at(PM1_REG_WDT_CNT, 0x00).is_ok();
    // read-modify-write。読み出しに失敗したら書かない（他ビットを消すと PM1 に壊れた設定が残る）。
    let mut c = false;
    let mut d = false;
    let mut pwr_v = 0xFFu8;
    let mut hold_v = 0xFFu8;
    if let Ok(pwr) = pm1.read_at(pmic::PWR_CFG) {
        pwr_v = pwr | PM1_PWR_CFG_LDO_EN | PM1_PWR_CFG_DCDC_EN;
        c = pm1.write_at(pmic::PWR_CFG, pwr_v).is_ok();
    }
    if let Ok(hold) = pm1.read_at(pmic::HOLD_CFG) {
        hold_v = hold | pmic::HOLD_LDO;
        d = pm1.write_at(pmic::HOLD_CFG, hold_v).is_ok();
    }
    println!(
        "[Board] pm1 pwr_cfg=0x{:02x} hold_cfg=0x{:02x} (i2c_cfg={} wdt={} pwr={} hold={})",
        pwr_v, hold_v, a as u8, b as u8, c as u8, d as u8
    );
    a && b && c && d
}

/// 電源ボタンの破壊的アクション（単クリック・リセット／ダブルクリック電源オフ）を無効化し、
/// 長押し判定を 4 秒に延ばす（誤操作→コールドブート固着の引き金を封じる）。
fn configure_power_button(i2c: &mut SysI2c) -> bool {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    let mut ok = true;
    let mut cfg1_v = 0xFFu8;
    let mut cfg2_v = 0xFFu8;
    if let Ok(cfg1) = pm1.read_at(PM1_REG_BTN_CFG_1) {
        cfg1_v = (cfg1 & !PM1_BTN_CFG1_MASK) | PM1_BTN_LONG_DLY_4S | PM1_BTN_SINGLE_RST_DIS;
        ok &= pm1.write_at(PM1_REG_BTN_CFG_1, cfg1_v).is_ok();
    } else {
        ok = false;
    }
    if let Ok(cfg2) = pm1.read_at(PM1_REG_BTN_CFG_2) {
        cfg2_v = cfg2 | PM1_BTN_DOUBLE_OFF_DIS;
        ok &= pm1.write_at(PM1_REG_BTN_CFG_2, cfg2_v).is_ok();
    } else {
        ok = false;
    }
    let _ = pm1.write_at(PM1_REG_IRQ_MASK3, PM1_BTN_IRQ_ALL);
    let _ = pm1.write_at(PM1_REG_IRQ_STATUS3, 0x00);
    println!(
        "[Board] pm1 btn_cfg1=0x{:02x} btn_cfg2=0x{:02x} (double-off dis, long=4s) ok={}",
        cfg1_v, cfg2_v, ok as u8
    );
    ok
}

/// R-5: C153 で LoRa 電源（PM1 G2）が出力 HIGH なら LOW にする。Lite では G2 に触らない。
fn ensure_lora_off(i2c: &mut SysI2c, is_c153: bool) {
    let mask = 1u8 << PM1_LORA_EN_GPIO;
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    let mode = pm1.read_at(pmic::GPIO_MODE).ok();
    let out = pm1.read_at(m5stack_papermono_lite::m5pm1::GPIO_OUT).ok();
    let (Some(mode), Some(out)) = (mode, out) else {
        println!("[Board] lora_en read FAILED");
        return;
    };
    let is_output = mode & mask != 0;
    let is_high = out & mask != 0;
    println!(
        "[Board] lora_en(pm1 g2) before: output={} high={} (c153={})",
        is_output as u8, is_high as u8, is_c153 as u8
    );
    if is_c153 && is_output && is_high {
        let ok = pm1.set_gpio_output(PM1_LORA_EN_GPIO, false).is_ok();
        println!("[Board] lora_en -> off {}", if ok { "OK" } else { "FAILED" });
    }
}

/// 電源レールと M5IOE1 を立ち上げる。
///
/// 1. バス整定待ち → PM1 電源保持・電源ボタン設定 → SKU 判定 → LoRa 電源遮断（R-5）。
/// 2. M5IOE1 発見 → IP2315 を I2C バスから隔離（`PYG11_PWM3` LOW）・PDM マイク電源 OFF・EPD_VDD ON。
/// 3. FT6336G タッチを電源サイクルして起動（T4 で使う）。
///
/// microSD 電源は Storage（T5）で投入する。各段の成否をログに出し、失敗しても続行する。
pub async fn bring_up(i2c: &mut SysI2c) -> BringUp {
    Timer::after(Duration::from_millis(POWER_SETTLE_MS)).await;

    let pm1_ok = probe_read(i2c, addresses::M5PM1, pmic::DEVICE_ID);
    println!("[Board] pm1 probe {}", if pm1_ok { "OK" } else { "FAILED" });
    let power_held = hold_power(i2c);
    println!("[Board] pm1 power hold {}", if power_held { "OK" } else { "FAILED" });
    let _ = configure_power_button(i2c);

    // SKU 判定は読み出しプローブのみ（ST25R3916 は初期化しない）。
    let is_c153 = probe_read(
        i2c,
        addresses::ST25R3916_LEFTOVER,
        addresses::ST25R3916_LEFTOVER_DEVICE_ID,
    );
    println!("[Board] sku {}", if is_c153 { "C153" } else { "C153-Lite" });
    ensure_lora_off(i2c, is_c153);

    let ioe_addr = begin_ioe(i2c).await;
    match ioe_addr {
        Some(addr) => println!("[Board] ioe1 found at 0x{:02x} OK", addr),
        None => println!("[Board] ioe1 not found FAILED"),
    }

    if ioe_addr.is_some() {
        let gate = set_output_verified(i2c, ioe1::IP2315_I2C_GATE, false);
        let pdm = set_output_verified(i2c, ioe1::PDM_VDD_ENABLE, false);
        let epd = set_output_verified(i2c, ioe1::EPD_VDD_ENABLE, true);
        println!(
            "[Board] rails ip2315_isolated={} pdm_off={} epd_vdd_on={}",
            gate as u8, pdm as u8, epd as u8
        );

        let _ = set_output_verified(i2c, ioe1::TOUCH_RST, false);
        let _ = set_output_verified(i2c, ioe1::TOUCH_VDD_ENABLE, false);
        Timer::after(Duration::from_millis(30)).await;
        let vdd = set_output_verified(i2c, ioe1::TOUCH_VDD_ENABLE, true);
        Timer::after(Duration::from_millis(20)).await;
        let rst = set_output_verified(i2c, ioe1::TOUCH_RST, true);
        Timer::after(Duration::from_millis(100)).await;
        println!("[Board] touch power vdd={} rst={}", vdd as u8, rst as u8);
    }

    BringUp {
        ioe_addr,
        power_held,
        is_c153,
    }
}
