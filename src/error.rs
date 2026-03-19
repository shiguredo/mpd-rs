use std::fmt;

/// MPD のパース時のエラー
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// エラーの種別を返す
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl std::error::Error for Error {}

/// エラーの種別
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// XML パースエラー
    Xml,
    /// 必須属性が欠けている
    MissingAttribute {
        element: &'static str,
        attribute: &'static str,
    },
    /// 必須要素が欠けている
    MissingElement {
        parent: &'static str,
        element: &'static str,
    },
    /// 属性の値が不正
    InvalidAttributeValue { attribute: &'static str },
    /// Duration 文字列が不正
    InvalidDuration,
    /// 予期しない構造
    UnexpectedStructure,
    /// XPath セレクタが不正
    InvalidXPath,
    /// Patch 操作の適用に失敗
    PatchFailed,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml => write!(f, "XML parse error"),
            Self::MissingAttribute { element, attribute } => {
                write!(f, "missing required attribute '{attribute}' on <{element}>")
            }
            Self::MissingElement { parent, element } => {
                write!(f, "missing required element <{element}> in <{parent}>")
            }
            Self::InvalidAttributeValue { attribute } => {
                write!(f, "invalid value for attribute '{attribute}'")
            }
            Self::InvalidDuration => write!(f, "invalid ISO 8601 duration"),
            Self::UnexpectedStructure => write!(f, "unexpected MPD structure"),
            Self::InvalidXPath => write!(f, "invalid XPath selector"),
            Self::PatchFailed => write!(f, "patch operation failed"),
        }
    }
}

/// `Result` 型エイリアス
pub type Result<T> = std::result::Result<T, Error>;
