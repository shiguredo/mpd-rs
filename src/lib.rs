//! MPEG-DASH MPD (Media Presentation Description) パーサー + ライターライブラリ
//!
//! MPD XML をパースして構造体に変換し、構造体から XML に再シリアライズできる。
//! パース → シリアライズ → パースのラウンドトリップが同じ結果になることを保証している。

mod duration;
mod error;
pub(crate) mod parser;
mod patch;
pub mod template;
pub mod types;
pub(crate) mod writer;

pub use duration::{format_duration, parse_duration};
pub use error::{Error, ErrorKind, Result};
pub use patch::{apply_patch, parse_patch};
pub use template::resolve_template;
pub use types::*;

/// MPD (XML 文字列) をパースする
pub fn parse(input: &str) -> Result<Mpd> {
    parser::parse(input)
}

/// MPD 構造体を XML 文字列にシリアライズする
pub fn write(mpd: &Mpd) -> String {
    writer::write(mpd)
}
