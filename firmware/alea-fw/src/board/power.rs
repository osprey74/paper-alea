//! 電源ボタン・USB 給電・LED・シャットダウン（M5PM1 / M5IOE1）。
//!
//! 出典：M5PM1 データシート v1.9（m5stack/M5PM1 の `docs/M5PM1_Datasheet_EN.pdf`）、
//! M5Unified `LED_PaperMono_Class.inl`、M5PaperMono-UserDemo `app_shutdown.cpp`（2026-10-03 調査）。
//! - 電源ボタンの単クリックのリセットは [`super::ioe`] で無効化済み。押下は `BTN_STATUS`（0x48）の
//!   bit7（押された記録・読むと消える）で知る。PM1 のファームが `sw_rev=0x54`（C153 実機）で、このレジスタがある。
//! - 電源オフ後は電源ボタンを 1 回押すと起動する（PM1 の機能・設定不要）。
//! - USB 給電中は SYS_CMD のシャットダウンをしてもすぐ起動し直す（実測・Nostos と papermono-rs で確認済み）。
//! - LED：緑＝M5IOE1 PYG8・青＝PYG9（いずれも HIGH で点灯）、赤＝M5PM1 の LED_EN（PWR_CFG bit4）。

use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::ioe1;
use m5stack_papermono_lite::m5pm1::{
    M5pm1, HOLD_CFG, HOLD_LDO, PWR_SRC, PWR_SRC_VIN, PWR_SRC_VINOUT,
};

use super::ioe::{set_push_pull_output, SysI2c};

/// M5PM1 `BTN_STATUS`（読み出し専用）。bit7 = 押された記録（読むと消える）、bit0 = 今押されているか。
const PM1_REG_BTN_STATUS: u8 = 0x48;
const PM1_BTN_EVENT: u8 = 1 << 7;

/// 前回読んでから電源ボタンが押されたか。
pub fn button_pressed(i2c: &mut SysI2c) -> bool {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    pm1.read_at(PM1_REG_BTN_STATUS)
        .is_ok_and(|v| v & PM1_BTN_EVENT != 0)
}

/// USB（5VIN）から給電されているか。読めなければ給電中とみなす（電源を切らない側に倒す）。
pub fn usb_present(i2c: &mut SysI2c) -> bool {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    pm1.read_at(PWR_SRC)
        .map_or(true, |v| v & (PWR_SRC_VIN | PWR_SRC_VINOUT) != 0)
}

/// RGB LED の緑と青（M5IOE1）を設定する。
pub fn set_led(i2c: &mut SysI2c, green: bool, blue: bool) {
    let _ = set_push_pull_output(i2c, ioe1::RGB_GREEN, green);
    let _ = set_push_pull_output(i2c, ioe1::RGB_BLUE, blue);
}

/// 起動時に LED をすべて消す（Nostos が点けた青が残っていたため・2026-10-03）。
pub fn leds_off(i2c: &mut SysI2c) {
    set_led(i2c, false, false);
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    let _ = pm1.set_led(false);
}

/// 電源を切る。LDO の保持を外してから M5PM1 にシャットダウンを指示する（UserDemo と同じ順）。
/// USB 給電中は切れずに起動し直す。
pub fn shutdown(i2c: &mut SysI2c) {
    leds_off(i2c);
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    if let Ok(hold) = pm1.read_at(HOLD_CFG) {
        let _ = pm1.write_at(HOLD_CFG, hold & !HOLD_LDO);
    }
    let _ = pm1.shutdown();
}
