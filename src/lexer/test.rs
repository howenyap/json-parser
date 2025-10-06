use super::*;
use LexerErrorKind::*;

fn expect_success(input: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(input.as_bytes());
    lexer.lex().expect("expected lexer to succeed")
}

fn expect_error(input: &str) -> LexerError {
    let mut lexer = Lexer::new(input.as_bytes());
    lexer.lex().expect_err("expected lexer to error")
}

#[test]
fn punctuation() {
    let tokens = expect_success(":,{}[]");
    let expected = [
        Token::Colon,
        Token::Comma,
        Token::Lcurl,
        Token::Rcurl,
        Token::Lsquare,
        Token::Rsquare,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn string() {
    let input = "\"hello world\"";
    let tokens = expect_success(input);

    let range = 1..input.len() - 1;
    let expected = [Token::String(range)];

    assert_eq!(tokens, expected);
}

#[test]
fn string_with_valid_escapes() {
    let input = "\"\\\"\\\\\\/\\b\\f\\n\\r\\t\\u0041\"";
    let tokens = expect_success(input);

    let expected = [Token::String(1..input.len() - 1)];
    assert_eq!(tokens, expected);
}

#[test]
fn number() {
    let input = "-1.2e+3";
    let tokens = expect_success(input);

    let range = 0..input.len();
    let expected = [Token::Number(range)];

    assert_eq!(tokens, expected)
}

#[test]
fn negative_zero() {
    let input = "-0";
    let tokens = expect_success(input);

    let expected = [Token::Number(0..input.len())];

    assert_eq!(tokens, expected);
}

#[test]
fn literals() {
    [
        ("true", [Token::True]),
        ("false", [Token::False]),
        ("null", [Token::Null]),
    ]
    .iter()
    .for_each(|(input, expected)| {
        let tokens = expect_success(input);

        assert_eq!(tokens, expected);
    });
}

#[test]
fn empty_input() {
    let tokens = expect_success("");

    assert_eq!(tokens, []);
}

#[test]
fn rejects_unterminated_string() {
    let LexerError { kind, line, col } = expect_error("\"hello");

    assert!(matches!(kind, InvalidString(StringError::Unterminated)));
    assert_eq!(line, 1);
    assert_eq!(col, 7);
}

#[test]
fn rejects_string_with_control_character() {
    let LexerError { kind, line, col } = expect_error("\"hello\nworld\"");

    assert!(matches!(
        kind,
        InvalidString(StringError::UnescapedControlCharacter { code: b'\n' })
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 7);
}

#[test]
fn rejects_string_with_other_control_character() {
    let LexerError { kind, .. } = expect_error("\"hello\x07world\"");

    assert!(matches!(
        kind,
        InvalidString(StringError::UnescapedControlCharacter { code: 0x07 })
    ));
}

#[test]
fn rejects_string_with_invalid_escape() {
    let LexerError { kind, line, col } = expect_error("\"\\x\"");

    assert!(matches!(
        kind,
        InvalidString(StringError::InvalidEscape { escape: b'x' })
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn rejects_string_with_incomplete_escape() {
    let LexerError { kind, line, col } = expect_error("\"\\");

    assert!(matches!(kind, InvalidString(StringError::IncompleteEscape)));
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn rejects_string_with_invalid_unicode_escape() {
    let LexerError { kind, line, col } = expect_error("\"\\u12G4\"");

    assert!(matches!(
        kind,
        InvalidString(StringError::InvalidUnicodeEscape { digits }) if digits == "12G"
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 6);
}

#[test]
fn rejects_string_with_short_unicode_escape() {
    let LexerError { kind, line, col } = expect_error("\"\\u12\"");

    assert!(matches!(
        kind,
        InvalidString(StringError::InvalidUnicodeEscape { digits }) if digits == "12\""
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 6);
}

#[test]
fn rejects_number_with_leading_zero() {
    let LexerError { kind, line, col } = expect_error("01");

    assert!(matches!(kind, InvalidNumber(NumberError::LeadingZero)));
    assert_eq!(line, 1);
    assert_eq!(col, 2);
}

#[test]
fn rejects_negative_number_with_leading_zero() {
    let LexerError { kind, line, col } = expect_error("-01");

    assert!(matches!(kind, InvalidNumber(NumberError::LeadingZero)));
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn rejects_decimal_with_multiple_decimal_points() {
    let LexerError { kind, line, col } = expect_error("1.2.3");

    assert!(matches!(
        kind,
        InvalidNumber(NumberError::InvalidDecimal {
            reason: "multiple decimal points found"
        })
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 4);
}

#[test]
fn rejects_decimal_with_invalid_postfix() {
    let LexerError { kind, line, col } = expect_error("1.");

    assert!(matches!(
        kind,
        InvalidNumber(NumberError::InvalidDecimal {
            reason: "decimal point must be followed by a digit"
        })
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn rejects_exponent_with_invalid_postfix() {
    let LexerError { kind, line, col } = expect_error("1e");

    assert!(matches!(
        kind,
        InvalidNumber(NumberError::InvalidExponent {
            reason: "exponent must be followed by '+' or '-' or a digit"
        })
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 3);
}

#[test]
fn rejects_negative_without_following_digit() {
    let LexerError { kind, line, col } = expect_error("-a");

    assert!(matches!(
        kind,
        InvalidNumber(NumberError::InvalidNegative {
            reason: "'-' must be followed by a digit"
        })
    ));
    assert_eq!(line, 1);
    assert_eq!(col, 2);
}
