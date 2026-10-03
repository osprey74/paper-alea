//! 設定（DESIGN.md §6.6・§9 設定画面）：バックライトの段階・自動電源オフまでの時間・電池残量の目安。
//!
//! 設定は M5PM1 の RTC RAM（電池で保持・電源オフをまたいで残る）に 3 バイトで保存する。
//! `[マジック, バックライト段階, 自動電源オフ（分）]`。マジックが違えば（初回・他のファームの後）保存なしとみなす。

/// バックライトの段階（0=消灯・1=弱・2=強）。
pub const BACKLIGHT_LEVELS: u8 = 3;

/// 自動電源オフの選択肢 [分]（0 = しない）。設定画面の札の並び。
pub const AUTO_OFF_CHOICES: [u8; 5] = [0, 1, 3, 5, 10];

/// RTC RAM に置く設定のマジック（Nostos の 0x5A と区別する）。
pub const MAGIC: u8 = 0xA1;

/// 保存する設定。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// バックライトの段階（0〜2）。
    pub backlight: u8,
    /// 自動電源オフまでの時間 [分]（0 = しない）。
    pub auto_off_min: u8,
}

impl Settings {
    /// RTC RAM に書く 3 バイト。
    pub fn encode(self) -> [u8; 3] {
        [MAGIC, self.backlight, self.auto_off_min]
    }

    /// RTC RAM から読んだ 3 バイト。マジックが違う・値が範囲外なら `None`。
    pub fn decode(b: [u8; 3]) -> Option<Self> {
        if b[0] != MAGIC || b[1] >= BACKLIGHT_LEVELS || b[2] > 120 {
            return None;
        }
        Some(Self {
            backlight: b[1],
            auto_off_min: b[2],
        })
    }
}

/// バックライトの段階 → PWM のデューティ（最大 `duty_max`）。弱は最大の 1/8（Nostos の段階 1 と同じ）。
pub fn backlight_duty(level: u8, duty_max: u16) -> u16 {
    match level {
        0 => 0,
        1 => duty_max / 8,
        _ => duty_max,
    }
}

/// 電池電圧 [mV] → 残量の目安 [%]（5% 刻み）。リチウムイオン電池の放電の目安の表を直線でつなぐ。
/// 負荷や温度で変わるため「約」として表示する。
pub fn battery_percent(mv: u16) -> u8 {
    // (mV, %)。4.15V 以上を満充電、3.40V 以下を空とする。
    const CURVE: [(u16, u8); 9] = [
        (3400, 0),
        (3500, 5),
        (3600, 15),
        (3700, 30),
        (3800, 50),
        (3900, 65),
        (4000, 80),
        (4100, 92),
        (4150, 100),
    ];
    if mv <= CURVE[0].0 {
        return 0;
    }
    for w in CURVE.windows(2) {
        let ((v0, p0), (v1, p1)) = (w[0], w[1]);
        if mv <= v1 {
            let p = u32::from(p0) + (u32::from(mv - v0) * u32::from(p1 - p0)) / u32::from(v1 - v0);
            return ((p + 2) / 5 * 5) as u8;
        }
    }
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let s = Settings {
            backlight: 2,
            auto_off_min: 10,
        };
        assert_eq!(Settings::decode(s.encode()), Some(s));
    }

    #[test]
    fn decode_rejects_foreign_or_bad_data() {
        assert_eq!(Settings::decode([0x5A, 1, 1]), None); // Nostos のマジック
        assert_eq!(Settings::decode([0x00, 0, 0]), None);
        assert_eq!(Settings::decode([MAGIC, 3, 3]), None); // 段階が範囲外
        assert_eq!(Settings::decode([MAGIC, 0, 200]), None);
    }

    #[test]
    fn backlight_steps() {
        assert_eq!(backlight_duty(0, 4095), 0);
        assert_eq!(backlight_duty(1, 4095), 511);
        assert_eq!(backlight_duty(2, 4095), 4095);
    }

    #[test]
    fn battery_curve() {
        assert_eq!(battery_percent(3300), 0);
        assert_eq!(battery_percent(3400), 0);
        assert_eq!(battery_percent(3800), 50);
        assert_eq!(battery_percent(3920), 70);
        assert_eq!(battery_percent(4150), 100);
        assert_eq!(battery_percent(4250), 100);
        // 電圧が上がれば残量は減らない。
        let mut last = 0;
        for mv in (3300..4300).step_by(5) {
            let p = battery_percent(mv);
            assert!(p >= last && p <= 100 && p % 5 == 0, "mv={mv} p={p}");
            last = p;
        }
    }
}
