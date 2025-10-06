use thiserror::Error;

#[derive(Debug)]
pub struct LexerError {
    pub kind: LexerErrorKind,
    pub line: usize,
    pub col: usize,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Error at line {}, col {}: {}",
            self.line, self.col, self.kind
        )
    }
}

#[derive(Debug, Error)]
pub enum LexerErrorKind {
    #[error("[invalid string] {0}")]
    InvalidString(StringError),
    #[error("[invalid number] {0}")]
    InvalidNumber(NumberError),
    #[error("[invalid literal] {0}")]
    InvalidLiteral(String),
    #[error("[invalid token] {0}")]
    InvalidToken(u8),
    #[error("eof")]
    Eof,
}

#[derive(Error, Debug)]
pub enum StringError {
    #[error("string not terminated, missing \"")]
    Unterminated,
    #[error("string ends with an incomplete escape sequence")]
    IncompleteEscape,
    #[error("string contains invalid escape sequence \\{escape}")]
    InvalidEscape { escape: u8 },
    #[error("string contains control character 0x{code:02X} that must be escaped")]
    UnescapedControlCharacter { code: u8 },
    #[error("unicode escape must be followed by four hexadecimal digits, found '{digits}'")]
    InvalidUnicodeEscape { digits: String },
}

#[derive(Error, Debug)]
pub enum NumberError {
    #[error("invalid decimal: {reason}")]
    InvalidDecimal { reason: &'static str },

    #[error("invalid exponent: {reason}")]
    InvalidExponent { reason: &'static str },

    #[error("numbers cannot have leading zeros")]
    LeadingZero,

    #[error("invalid negative: {reason}")]
    InvalidNegative { reason: &'static str },
}

pub type Result<T> = std::result::Result<T, LexerErrorKind>;
