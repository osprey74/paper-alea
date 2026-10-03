//! 電源ボタン・USB 給電・LED・シャットダウン（M5PM1 / M5IOE1）。
//!
//! 出典：M5PM1 データシート v1.9（m5stack/M5PM1 の `docs/M5PM1_Datasheet_EN.pdf`）、
//! M5Unified `LED_PaperMono_Class.inl`、M5PaperMono-UserDemo `app_shutdown.cpp`（2026-10-03 調査）。
//! - 電源ボタンの単クリックのリセットは [`super::ioe`] で無効化済み。押下は `BTN_STATUS`（0x48）の
//!   bit7（押された記録・読むと消える）で知る。PM1 のファームが `sw_rev=0x54`（C153 実機）で、このレジスタがある。
//! - 電源オフ後は電源ボタンを 1 回押すと起動する（PM1 の機能・設定不要）。
//! - USB 給電中は SYS_CMD のシャットダウンをしてもすぐ起動し直す（実測・Nostos と papermono-rs で確認済み）。
//! - LED：緑＝M5IOE1 PYG8・青＝PYG9（いずれも HIGH で点灯）、赤＝M5PM1 の LED_EN（PWR_CFG bit4）。
//! - フロントライト：M5PM1 の PWM0（G3）。設定は M5PM1 の RTC RAM（電池で保持）に置く（Nostos `ioe.rs` と同じ手順・MIT）。

use alea_core::settings::Settings;
use m5stack_papermono_lite::addresses;
use m5stack_papermono_lite::ioe1;
use m5stack_papermono_lite::pmic;
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

/// M5PM1 の RTC RAM（0xA0〜・32 バイト・電池で保持）の、設定の置き場所。
const PM1_RTC_RAM_SETTINGS: u8 = 0xA0;

/// フロントライトの PWM デューティを設定する（0 = 消灯）。
pub fn set_frontlight(i2c: &mut SysI2c, duty: u16) {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    if duty == 0 {
        let _ = pm1.set_pwm0_duty(0);
    } else {
        let _ = pm1.enable_pwm0(pmic::FRONTLIGHT_PWM);
        let _ = pm1.set_pwm0_duty(duty);
    }
}

/// 電池電圧 [mV]（M5PM1 の ADC）。
pub fn read_vbat_mv(i2c: &mut SysI2c) -> Option<u16> {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    if let Ok(v) = pm1.read_le16(pmic::VBAT_L) {
        return Some(v);
    }
    let lo = pm1.read_at(pmic::VBAT_L).ok()?;
    let hi = pm1.read_at(pmic::VBAT_L.wrapping_add(1)).ok()?;
    Some(pmic::adc_mv(lo, hi))
}

/// 保存した設定を読む。無い（初回・他のファームの後）・壊れていれば `None`。
pub fn load_settings(i2c: &mut SysI2c) -> Option<Settings> {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    let mut b = [0u8; 3];
    for (i, v) in b.iter_mut().enumerate() {
        *v = pm1.read_at(PM1_RTC_RAM_SETTINGS + i as u8).ok()?;
    }
    Settings::decode(b)
}

/// 設定を保存する。
pub fn save_settings(i2c: &mut SysI2c, s: Settings) {
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    for (i, v) in s.encode().iter().enumerate() {
        let _ = pm1.write_at(PM1_RTC_RAM_SETTINGS + i as u8, *v);
    }
}

/// 電源を切る。フロントライトを消し、LDO の保持を外してから M5PM1 にシャットダウンを指示する（UserDemo と同じ順）。
/// USB 給電中は切れずに起動し直す。
pub fn shutdown(i2c: &mut SysI2c) {
    leds_off(i2c);
    // PM1 は電源オフ中も動いているため、PWM を止めないとライトが点いたまま残る。
    set_frontlight(i2c, 0);
    let mut pm1 = M5pm1::new(&mut *i2c, addresses::M5PM1);
    if let Ok(hold) = pm1.read_at(HOLD_CFG) {
        let _ = pm1.write_at(HOLD_CFG, hold & !HOLD_LDO);
    }
    let _ = pm1.shutdown();
}
