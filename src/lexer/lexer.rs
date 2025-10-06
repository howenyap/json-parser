use std::ops::Range;

use super::error::{LexerError, LexerErrorKind, NumberError, Result, StringError};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    Colon,
    Comma,
    Lcurl,
    Rcurl,
    Lsquare,
    Rsquare,
    Eof,

    String(Range<usize>),
    Number(Range<usize>),
    True,
    False,
    Null,
}

impl Token {
    pub fn from_byte(b: u8) -> Result<Self> {
        let token = match b {
            b':' => Self::Colon,
            b',' => Self::Comma,
            b'{' => Self::Lcurl,
            b'}' => Self::Rcurl,
            b'[' => Self::Lsquare,
            b']' => Self::Rsquare,
            _ => {
                return Err(LexerErrorKind::InvalidToken(b));
            }
        };

        Ok(token)
    }

    pub fn to_string(&self, input: &[u8]) -> String {
        match self {
            Token::Colon
            | Token::Comma
            | Token::Lcurl
            | Token::Rcurl
            | Token::Lsquare
            | Token::Rsquare
            | Token::Eof
            | Token::True
            | Token::False
            | Token::Null => format!("{self:?}"),
            Token::String(range) | Token::Number(range) => {
                let bytes = &input[range.clone()];
                let s = str::from_utf8(bytes).unwrap_or("invalid utf8");
                let ident = match self {
                    Token::String(_) => "String",
                    Token::Number(_) => "Number",
                    _ => unreachable!(),
                };
                format!("{ident}({s})")
            }
        }
    }
}

pub struct Lexer<'a> {
    input: &'a [u8],
    pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    pub fn lex(&mut self) -> std::result::Result<Vec<Token>, LexerError> {
        let mut tokens = vec![];

        loop {
            match self.next_token() {
                Err(e) => {
                    return Err(LexerError {
                        kind: e,
                        line: self.line,
                        col: self.col,
                    });
                }
                Ok(Token::Eof) => return Ok(tokens),
                Ok(token) => tokens.push(token),
            }
        }
    }

    fn next_token(&mut self) -> Result<Token> {
        self.skip_whitespace();

        let Some(b) = self.curr() else {
            return Ok(Token::Eof);
        };

        if let Ok(token) = Token::from_byte(b) {
            self.advance()?;
            return Ok(token);
        }

        let token = match b {
            b'"' => self.read_string()?,
            b if b.is_ascii_alphabetic() => self.read_literal()?,
            b if b == b'-' || b.is_ascii_digit() => self.read_number()?,
            b => return Err(LexerErrorKind::InvalidToken(b)),
        };

        Ok(token)
    }

    fn read_literal(&mut self) -> Result<Token> {
        let start = self.pos;

        while let Some(b) = self.curr()
            && b.is_ascii_alphabetic()
        {
            self.advance()?;
        }

        let end = self.pos;
        let bytes = &self.input[start..end];

        let literal = match bytes {
            b"true" => Token::True,
            b"false" => Token::False,
            b"null" => Token::Null,
            _ => {
                return Err(LexerErrorKind::InvalidLiteral(
                    String::from_utf8_lossy(bytes).to_string(),
                ));
            }
        };

        Ok(literal)
    }

    fn read_number(&mut self) -> Result<Token> {
        let start = self.pos;

        if self.curr() == Some(b'-') {
            self.advance()?;

            match self.curr() {
                Some(byte) if byte.is_ascii_digit() => (),
                Some(b'0') => {
                    return Err(LexerErrorKind::InvalidNumber(NumberError::LeadingZero));
                }
                _ => {
                    return Err(LexerErrorKind::InvalidNumber(
                        NumberError::InvalidNegative {
                            reason: "'-' must be followed by a digit",
                        },
                    ));
                }
            }
        }

        if self.curr() == Some(b'0') {
            match self.peek() {
                Some(b'.') | Some(b'e') | Some(b'E') => (),
                Some(byte) if byte.is_ascii_digit() => {
                    self.advance()?;
                    return Err(LexerErrorKind::InvalidNumber(NumberError::LeadingZero));
                }
                _ => {
                    self.advance()?;
                    let end = self.pos;
                    return Ok(Token::Number(start..end));
                }
            }
        }

        let mut found_decimal = false;
        let mut found_exponent = false;

        while let Some(b) = self.curr() {
            match b {
                b'.' => {
                    if found_decimal {
                        return Err(LexerErrorKind::InvalidNumber(NumberError::InvalidDecimal {
                            reason: "multiple decimal points found",
                        }));
                    }

                    found_decimal = true;
                    self.advance()?;

                    match self.curr() {
                        Some(byte) if byte.is_ascii_digit() => (),
                        _ => {
                            return Err(LexerErrorKind::InvalidNumber(
                                NumberError::InvalidDecimal {
                                    reason: "decimal point must be followed by a digit",
                                },
                            ));
                        }
                    }
                }
                b'e' | b'E' => {
                    if found_exponent {
                        return Err(LexerErrorKind::InvalidNumber(
                            NumberError::InvalidExponent {
                                reason: "multiple exponents found",
                            },
                        ));
                    }

                    found_exponent = true;
                    self.advance()?;

                    match self.curr() {
                        Some(b'+') | Some(b'-') => (),
                        Some(byte) if byte.is_ascii_digit() => (),
                        _ => {
                            return Err(LexerErrorKind::InvalidNumber(
                                NumberError::InvalidExponent {
                                    reason: "exponent must be followed by '+' or '-' or a digit",
                                },
                            ));
                        }
                    }
                }
                byte if byte.is_ascii_digit() => (),
                _ => break,
            };

            self.advance()?;
        }

        let end = self.pos;
        Ok(Token::Number(start..end))
    }

    fn read_string(&mut self) -> Result<Token> {
        self.advance()?;

        let start = self.pos;

        while let Some(b) = self.curr() {
            match b {
                b'"' => {
                    let end = self.pos;
                    self.advance()?;
                    return Ok(Token::String(start..end));
                }
                b'\\' => {
                    self.advance()?;

                    let Some(escape) = self.curr() else {
                        return Err(LexerErrorKind::InvalidString(StringError::IncompleteEscape));
                    };

                    match escape {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => self.advance()?,
                        b'u' => self.read_unicode_escape()?,
                        other => {
                            return Err(LexerErrorKind::InvalidString(
                                StringError::InvalidEscape {
                                    escape: other,
                                },  
                            ));
                        }
                    }
                }
                control if control < 0x20 => {
                    return Err(LexerErrorKind::InvalidString(
                        StringError::UnescapedControlCharacter { code: control },
                    ));
                }
                _ => self.advance()?,
            }
        }

        Err(LexerErrorKind::InvalidString(StringError::Unterminated))
    }

    fn read_unicode_escape(&mut self) -> Result<()> {
        self.advance()?;
        let start = self.pos;

        for _ in 0..4 {
            match self.curr() {
                Some(b) if b.is_ascii_hexdigit() => self.advance()?,
                Some(_) => {
                    let end = self.pos + 1;
                    return Err(LexerErrorKind::InvalidString(
                        StringError::InvalidUnicodeEscape {
                            digits: String::from_utf8_lossy(&self.input[start..end]).to_string(),
                        },
                    ));
                }
                None => {
                    return Err(LexerErrorKind::InvalidString(
                        StringError::InvalidUnicodeEscape {
                            digits: String::from_utf8_lossy(&self.input[start..self.pos])
                                .to_string(),
                        },
                    ));
                }
            }
        }

        Ok(())
    }

    fn advance(&mut self) -> Result<()> {
        let Some(b) = self.curr() else {
            return Err(LexerErrorKind::Eof);
        };

        self.pos += 1;

        if b == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        Ok(())
    }

    fn curr(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos + 1).copied()
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.curr().is_some_and(|b| b.is_ascii_whitespace()) {
            self.advance().unwrap();
        }
    }
}
