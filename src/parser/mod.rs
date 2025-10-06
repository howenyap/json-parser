pub mod error;

pub use error::{ParserError, Result};

use std::{collections::HashMap, ops::Range};

use crate::lexer::Token;

#[allow(unused)]
#[derive(Clone, Debug)]
pub enum Value<'a> {
    Object(HashMap<&'a str, Value<'a>>),
    Array(Vec<Value<'a>>),
    String(&'a str),
    Number(&'a str),
    Boolean(bool),
    Null,
}

pub struct Parser<'a> {
    input: &'a [u8],
    tokens: Vec<Token>,
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, input: &'a [u8]) -> Self {
        Self {
            input,
            tokens,
            pos: 0,
        }
    }

    pub fn parse(&mut self) -> Option<std::result::Result<Value<'a>, ParserError>> {
        self.curr()?;

        Some(self.parse_value())
    }

    fn parse_value(&mut self) -> std::result::Result<Value<'a>, ParserError> {
        let Some(token) = self.curr() else {
            return Err(ParserError::UnexpectedEof);
        };

        match token {
            Token::Lcurl => self.parse_object(),
            Token::Lsquare => self.parse_array(),
            Token::String(range) => {
                let s = Self::read_str(self.input, range)?;
                self.next()?;
                Ok(Value::String(s))
            }
            Token::Number(range) => {
                let s = Self::read_str(self.input, range)?;
                self.next()?;
                Ok(Value::Number(s))
            }
            Token::True => {
                self.next()?;
                Ok(Value::Boolean(true))
            }
            Token::False => {
                self.next()?;
                Ok(Value::Boolean(false))
            }
            Token::Null => {
                self.next()?;
                Ok(Value::Null)
            }
            _ => Err(ParserError::InvalidValue {
                found: token.clone(),
            }),
        }
    }

    fn parse_object(&mut self) -> Result<'a, Value<'a>> {
        let mut object: HashMap<&str, Value<'a>> = HashMap::new();
        self.next()?;

        while let Some(token) = self.curr() {
            if token == &Token::Rcurl {
                self.next()?;
                return Ok(Value::Object(object));
            }

            let Some(Token::String(range)) = self.curr() else {
                return Err(ParserError::InvalidKey);
            };

            let key = Self::read_str(self.input, range)?;

            if object.contains_key(&key) {
                return Err(ParserError::DuplicateKey);
            };

            self.next()?;

            let Ok(Token::Colon) = self.next() else {
                return Err(ParserError::MissingColon);
            };

            let value = self.parse_value()?;
            object.insert(key, value);

            match self.curr() {
                Some(Token::Comma) => {
                    if self.peek() == Some(&Token::Rcurl) {
                        return Err(ParserError::TrailingComma);
                    }

                    self.next()?;
                }
                Some(Token::Rcurl) => {
                    self.next()?;
                    return Ok(Value::Object(object));
                }
                other => {
                    return Err(ParserError::InvalidValue {
                        found: other.cloned().unwrap_or(Token::Null),
                    });
                }
            }
        }

        Err(ParserError::UnexpectedEof)
    }

    fn parse_array(&mut self) -> Result<'a, Value<'a>> {
        let mut array = Vec::new();
        self.next()?;

        if self.curr() == Some(&Token::Rsquare) {
            self.next()?;
            return Ok(Value::Array(array));
        }

        while let Some(token) = self.curr() {
            if token == &Token::Rsquare {
                self.next()?;
                return Ok(Value::Array(array));
            }

            let value = self.parse_value()?;
            array.push(value);

            match self.curr() {
                Some(Token::Comma) => {
                    self.next()?;
                }
                Some(Token::Rsquare) => {
                    self.next()?;
                    return Ok(Value::Array(array));
                }
                _ => {
                    return Err(ParserError::InvalidArray);
                }
            }
        }

        Err(ParserError::UnexpectedEof)
    }

    fn next(&mut self) -> std::result::Result<&Token, ParserError> {
        if self.curr().is_some() {
            let token = self
                .tokens
                .get(self.pos)
                .ok_or(ParserError::UnexpectedEof)?;

            self.pos += 1;

            Ok(token)
        } else {
            Err(ParserError::UnexpectedEof)
        }
    }

    fn curr(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos + 1)
    }

    fn read_str<'b>(
        input: &'b [u8],
        range: &Range<usize>,
    ) -> std::result::Result<&'b str, ParserError> {
        std::str::from_utf8(&input[range.clone()]).map_err(|_| ParserError::NonUTF8String)
    }
}

#[cfg(test)]
mod test;
