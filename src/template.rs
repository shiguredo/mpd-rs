use std::fmt::Write;

/// SegmentTemplate のテンプレート変数を展開する
///
/// 対応する変数:
/// - `$Number$` / `$Number%0Nd$` — セグメント番号 (ゼロ埋めフォーマット対応)
/// - `$Time$` — セグメント開始時刻
/// - `$Bandwidth$` — ビットレート
/// - `$RepresentationID$` — Representation ID
pub fn resolve_template(
    template: &str,
    number: u64,
    time: u64,
    bandwidth: u64,
    representation_id: &str,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            // $$ はリテラル $
            if chars.peek() == Some(&'$') {
                chars.next();
                result.push('$');
                continue;
            }

            // 変数名を読み取る
            let mut var = String::new();
            for vc in chars.by_ref() {
                if vc == '$' {
                    break;
                }
                var.push(vc);
            }

            // フォーマット指定子をパースする (例: Number%05d)
            if let Some(pct_pos) = var.find('%') {
                let name = &var[..pct_pos];
                let fmt_spec = &var[pct_pos + 1..];
                let value = match name {
                    "Number" => number,
                    "Time" => time,
                    "Bandwidth" => bandwidth,
                    _ => {
                        // 不明な変数はそのまま出力する
                        write!(result, "${var}$").unwrap();
                        continue;
                    }
                };
                // %0Nd 形式のパース
                if let Some(width) = parse_format_width(fmt_spec) {
                    write!(result, "{value:0>width$}").unwrap();
                } else {
                    write!(result, "{value}").unwrap();
                }
            } else {
                match var.as_str() {
                    "Number" => write!(result, "{number}").unwrap(),
                    "Time" => write!(result, "{time}").unwrap(),
                    "Bandwidth" => write!(result, "{bandwidth}").unwrap(),
                    "RepresentationID" => result.push_str(representation_id),
                    _ => {
                        // 不明な変数はそのまま出力する
                        write!(result, "${var}$").unwrap();
                    }
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

/// `0Nd` 形式のフォーマット指定子から幅を取得する
fn parse_format_width(spec: &str) -> Option<usize> {
    let spec = spec.strip_prefix('0')?;
    let spec = spec.strip_suffix('d')?;
    spec.parse::<usize>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number() {
        assert_eq!(
            resolve_template("seg_$Number$.m4s", 42, 0, 0, ""),
            "seg_42.m4s"
        );
    }

    #[test]
    fn test_number_with_format() {
        assert_eq!(
            resolve_template("seg_$Number%05d$.m4s", 1, 0, 0, ""),
            "seg_00001.m4s"
        );
    }

    #[test]
    fn test_time() {
        assert_eq!(
            resolve_template("seg_$Time$.m4s", 0, 90000, 0, ""),
            "seg_90000.m4s"
        );
    }

    #[test]
    fn test_bandwidth() {
        assert_eq!(
            resolve_template("$Bandwidth$/seg.m4s", 0, 0, 2000000, ""),
            "2000000/seg.m4s"
        );
    }

    #[test]
    fn test_representation_id() {
        assert_eq!(
            resolve_template("$RepresentationID$/init.mp4", 0, 0, 0, "video_1"),
            "video_1/init.mp4"
        );
    }

    #[test]
    fn test_multiple_variables() {
        assert_eq!(
            resolve_template("$RepresentationID$/seg_$Number%03d$.m4s", 7, 0, 0, "v1"),
            "v1/seg_007.m4s"
        );
    }

    #[test]
    fn test_dollar_escape() {
        assert_eq!(
            resolve_template("price_$$_$Number$.m4s", 1, 0, 0, ""),
            "price_$_1.m4s"
        );
    }

    #[test]
    fn test_no_variables() {
        assert_eq!(resolve_template("init.mp4", 0, 0, 0, ""), "init.mp4");
    }
}
