use proptest::prelude::*;

// 任意の時・分・秒から ISO 8601 Duration 文字列を生成し、パース結果が一致するか検証する
proptest! {
    #[test]
    fn roundtrip_hms(
        h in 0u32..100,
        m in 0u32..60,
        s in 0u32..60,
        ms in 0u32..1000,
    ) {
        let duration_str = if ms > 0 {
            format!("PT{h}H{m}M{s}.{ms:03}S")
        } else {
            format!("PT{h}H{m}M{s}S")
        };

        let expected = f64::from(h) * 3600.0
            + f64::from(m) * 60.0
            + f64::from(s)
            + f64::from(ms) / 1000.0;

        let parsed = shiguredo_mpd::parse_duration(&duration_str).unwrap();
        let diff = (parsed - expected).abs();
        prop_assert!(diff < 1e-9, "parsed={parsed}, expected={expected}, diff={diff}");
    }

    #[test]
    fn seconds_only(s in 0u32..100000, ms in 0u32..1000) {
        let duration_str = if ms > 0 {
            format!("PT{s}.{ms:03}S")
        } else {
            format!("PT{s}S")
        };

        let expected = f64::from(s) + f64::from(ms) / 1000.0;
        let parsed = shiguredo_mpd::parse_duration(&duration_str).unwrap();
        let diff = (parsed - expected).abs();
        prop_assert!(diff < 1e-9, "parsed={parsed}, expected={expected}");
    }

    #[test]
    fn days_and_time(d in 0u32..365, h in 0u32..24, m in 0u32..60, s in 0u32..60) {
        let duration_str = format!("P{d}DT{h}H{m}M{s}S");
        let expected = f64::from(d) * 86400.0
            + f64::from(h) * 3600.0
            + f64::from(m) * 60.0
            + f64::from(s);
        let parsed = shiguredo_mpd::parse_duration(&duration_str).unwrap();
        let diff = (parsed - expected).abs();
        prop_assert!(diff < 1e-9, "parsed={parsed}, expected={expected}");
    }

    // 不正な文字列でパニックしないことを検証する
    #[test]
    fn no_panic_on_arbitrary_input(s in "\\PC{0,50}") {
        let _ = shiguredo_mpd::parse_duration(&s);
    }
}
