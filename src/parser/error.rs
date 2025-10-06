use crate::lexer::Token;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error(
        "Json values can only be an object, array, number, string, true, false, or null, found: {found:?}"
    )]
    InvalidValue { found: Token },

    #[error("Json keys must be followed by a colon")]
    MissingColon,

    #[error("Json strings must be valid UTF-8")]
    NonUTF8String,

    #[error("Unexpected end of input")]
    UnexpectedEof,

    #[error("Keys must be strings")]
    InvalidKey,

    #[error("Keys must be unique within an object")]
    DuplicateKey,

    #[error("Array value must either be terminated or followed by a comma")]
    InvalidArray,

    #[error("Trailing commas are not allowed")]
    TrailingComma,
}

pub type Result<'a, T> = std::result::Result<T, ParserError>;
