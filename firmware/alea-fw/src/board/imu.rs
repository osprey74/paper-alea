//! BMI270 加速度センサー（システム I2C・0x68）。
//!
//! 初期化手順は papermono-rs `firmware/embassy-debug/src/ui.rs`（MIT）に倣う：
//! ソフトリセット → 30ms → 省電力解除 → 1ms → Bosch 標準設定（8KB）の書き込み → 50ms →
//! 状態確認 → 加速度サンプリング開始（100Hz）。
//! 変更点：測定範囲を ±2g から **±8g** にする（振ると 2g を簡単に超えて飽和するため）。

use embassy_time::{Duration, Timer};
use esp_println::println;
use m5stack_papermono_lite::imu;

use super::ioe::SysI2c;

/// `ACC_RANGE` の ±8g 設定値。
const ACC_RANGE_8G: u8 = 0x02;
/// ±8g のときの感度 [LSB/g]。
const LSB_PER_G_8G: i32 = 4096;
/// `INTERNAL_STATUS` の初期化完了（`message` = 0x01）。
const STATUS_INIT_OK: u8 = 0x01;

/// BMI270 を初期化する。成功で `true`（失敗しても起動は続ける）。
pub async fn begin(i2c: &mut SysI2c) -> bool {
    let mut id = [0u8];
    let id_ok = i2c
        .write_read(imu::ADDRESS, &[imu::CHIP_ID], &mut id)
        .is_ok()
        && id[0] == imu::CHIP_ID_VALUE;
    if !id_ok {
        println!("[Board] imu chip_id=0x{:02x} FAILED", id[0]);
        return false;
    }
    let _ = imu::soft_reset(i2c);
    Timer::after(Duration::from_millis(30)).await;
    let _ = imu::disable_adv_power_save(i2c);
    Timer::after(Duration::from_millis(1)).await;
    let cfg = imu::load_config(i2c).is_ok();
    Timer::after(Duration::from_millis(50)).await;
    let status = imu::read_internal_status(i2c).unwrap_or(0xFF);
    let _ = imu::enable_accel_sampling(i2c);
    let range = i2c.write(imu::ADDRESS, &[imu::ACC_RANGE, ACC_RANGE_8G]).is_ok();
    Timer::after(Duration::from_millis(20)).await;
    let ok = cfg && status & 0x0F == STATUS_INIT_OK && range;
    println!(
        "[Board] imu bmi270 config={} status=0x{:02x} range=8g {}",
        cfg as u8,
        status,
        if ok { "OK" } else { "FAILED" }
    );
    ok
}

/// 加速度 [mg]（センサー座標）を読む。
pub fn read_mg(i2c: &mut SysI2c) -> Option<(i32, i32, i32)> {
    let s = imu::read_accel(i2c).ok()?;
    let mg = |v: i16| i32::from(v) * 1000 / LSB_PER_G_8G;
    Some((mg(s.x), mg(s.y), mg(s.z)))
}
