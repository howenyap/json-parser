pub mod error;
pub mod parser;

pub use error::{ParserError, Result};
pub use parser::{Parser, Value};

#[cfg(test)]
mod test;