pub mod error;
pub mod lexer;

pub use error::{LexerError, LexerErrorKind, NumberError, Result, StringError};
pub use lexer::{Lexer, Token};

#[cfg(test)]
mod test;