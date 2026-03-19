use crate::error::{Error, ErrorKind};

/// ISO 8601 Duration 文字列を秒に変換する
///
/// 対応形式: `PT1H2M3.4S`, `PT30S`, `PT1M`, `P1DT12H` など
pub fn parse_duration(input: &str) -> crate::error::Result<f64> {
    let s = input.trim();
    if !s.starts_with('P') {
        return Err(Error::new(
            ErrorKind::InvalidDuration,
            format!("duration must start with 'P': {input}"),
        ));
    }

    let s = &s[1..];
    if s.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidDuration,
            format!("empty duration: {input}"),
        ));
    }

    let mut seconds: f64 = 0.0;
    let mut in_time = false;
    let mut num_start: Option<usize> = None;
    let mut has_component = false;

    for (i, c) in s.char_indices() {
        match c {
            'T' => {
                in_time = true;
                num_start = None;
            }
            '0'..='9' | '.' => {
                if num_start.is_none() {
                    num_start = Some(i);
                }
            }
            'Y' | 'W' => {
                // 年と週は MPD では使われないが、パースだけはする
                let val = parse_number(s, num_start, i, input)?;
                match c {
                    'Y' => seconds += val * 365.25 * 86400.0,
                    'W' => seconds += val * 7.0 * 86400.0,
                    _ => unreachable!(),
                }
                num_start = None;
                has_component = true;
            }
            'D' => {
                let val = parse_number(s, num_start, i, input)?;
                seconds += val * 86400.0;
                num_start = None;
                has_component = true;
            }
            'H' => {
                if !in_time {
                    return Err(Error::new(
                        ErrorKind::InvalidDuration,
                        format!("'H' without 'T': {input}"),
                    ));
                }
                let val = parse_number(s, num_start, i, input)?;
                seconds += val * 3600.0;
                num_start = None;
                has_component = true;
            }
            'M' => {
                let val = parse_number(s, num_start, i, input)?;
                if in_time {
                    seconds += val * 60.0;
                } else {
                    // 月 (MPD では通常使われないが、安全のため)
                    seconds += val * 30.44 * 86400.0;
                }
                num_start = None;
                has_component = true;
            }
            'S' => {
                if !in_time {
                    return Err(Error::new(
                        ErrorKind::InvalidDuration,
                        format!("'S' without 'T': {input}"),
                    ));
                }
                let val = parse_number(s, num_start, i, input)?;
                seconds += val;
                num_start = None;
                has_component = true;
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidDuration,
                    format!("unexpected character '{c}': {input}"),
                ));
            }
        }
    }

    // 数値が残っている場合はエラー
    if num_start.is_some() {
        return Err(Error::new(
            ErrorKind::InvalidDuration,
            format!("trailing number without unit: {input}"),
        ));
    }

    // 成分が 1 つもない場合はエラー (例: "PT", "P")
    if !has_component {
        return Err(Error::new(
            ErrorKind::InvalidDuration,
            format!("no duration components: {input}"),
        ));
    }

    Ok(seconds)
}

/// 部分文字列を f64 にパースする
fn parse_number(
    s: &str,
    num_start: Option<usize>,
    end: usize,
    original: &str,
) -> crate::error::Result<f64> {
    let start = num_start.ok_or_else(|| {
        Error::new(
            ErrorKind::InvalidDuration,
            format!("unit without number: {original}"),
        )
    })?;
    s[start..end].parse::<f64>().map_err(|_| {
        Error::new(
            ErrorKind::InvalidDuration,
            format!("invalid number '{}': {original}", &s[start..end]),
        )
    })
}

/// 秒数を ISO 8601 Duration 文字列に変換する
///
/// 出力形式: `PT1H2M3.4S`, `PT30S` など
pub fn format_duration(seconds: f64) -> String {
    if seconds == 0.0 {
        return "PT0S".to_string();
    }

    let mut s = String::from("PT");
    let mut remaining = seconds;

    let hours = (remaining / 3600.0).floor() as u64;
    if hours > 0 {
        s.push_str(&format!("{hours}H"));
        remaining -= hours as f64 * 3600.0;
    }

    let minutes = (remaining / 60.0).floor() as u64;
    if minutes > 0 {
        s.push_str(&format!("{minutes}M"));
        remaining -= minutes as f64 * 60.0;
    }

    if remaining > 0.0 || s == "PT" {
        // 整数秒なら小数点なしで出力する
        if remaining == remaining.floor() {
            s.push_str(&format!("{}S", remaining as u64));
        } else {
            // 不要な末尾ゼロを削除する
            let formatted = format!("{remaining:.3}");
            let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
            s.push_str(trimmed);
            s.push('S');
        }
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_durations() {
        assert_eq!(parse_duration("PT30S").unwrap(), 30.0);
        assert_eq!(parse_duration("PT0.5S").unwrap(), 0.5);
        assert_eq!(parse_duration("PT1M").unwrap(), 60.0);
        assert_eq!(parse_duration("PT1H").unwrap(), 3600.0);
        assert_eq!(parse_duration("PT1H2M3S").unwrap(), 3723.0);
        assert_eq!(parse_duration("PT1H2M3.4S").unwrap(), 3723.4);
    }

    #[test]
    fn test_days() {
        assert_eq!(parse_duration("P1D").unwrap(), 86400.0);
        assert_eq!(parse_duration("P1DT12H").unwrap(), 86400.0 + 43200.0);
    }

    #[test]
    fn test_fractional_seconds() {
        assert_eq!(parse_duration("PT0.010S").unwrap(), 0.010);
        assert_eq!(parse_duration("PT4.000S").unwrap(), 4.0);
    }

    #[test]
    fn test_whitespace_trimmed() {
        assert_eq!(parse_duration("  PT30S  ").unwrap(), 30.0);
    }

    #[test]
    fn test_invalid() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("30S").is_err());
        assert!(parse_duration("PT").is_err());
        assert!(parse_duration("PTS").is_err());
        assert!(parse_duration("PT1H2").is_err());
        assert!(parse_duration("PTxS").is_err());
    }
}
