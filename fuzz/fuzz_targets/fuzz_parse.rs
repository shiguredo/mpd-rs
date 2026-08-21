#![no_main]
use libfuzzer_sys::fuzz_target;

// 任意のバイト列に対してパーサーがパニックしないことを検証する
fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data)
        && let Ok(mpd) = shiguredo_mpd::parse(s)
    {
        // パース成功時はラウンドトリップも検証する
        let xml = shiguredo_mpd::write(&mpd);
        let _ = shiguredo_mpd::parse(&xml);
    }
});
