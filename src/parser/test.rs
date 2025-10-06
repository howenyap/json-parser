use super::*;
use ParserError::*;

fn expect_success(input: &str) -> Value {
    let bytes = input.as_bytes();
    let mut lexer = crate::lexer::Lexer::new(bytes);
    let tokens = lexer.lex().expect("Lexing failed");

    let mut parser = Parser::new(tokens, bytes);
    let result = parser.parse();

    match result {
        Some(Ok(value)) => value,
        Some(Err(e)) => panic!("Expected success, but failed: {e}"),
        None => panic!("Missing result"),
    }
}

fn expect_failure(input: &str) -> ParserError {
    let bytes = input.as_bytes();
    let mut lexer = crate::lexer::Lexer::new(bytes);
    let tokens = lexer.lex().expect("Lexing failed");

    let mut parser = Parser::new(tokens, bytes);
    let result = parser.parse();

    match result {
        Some(Ok(_)) => panic!("Expected failure, but succeeded"),
        Some(Err(e)) => e,
        None => panic!("Missing result"),
    }
}

#[test]
fn empty_object() {
    let value = expect_success("{}");

    let Value::Object(obj) = value else {
        panic!("Expected object, got {value:?}");
    };

    assert!(obj.is_empty());
}

#[test]
fn empty_array() {
    let value = expect_success("[]");

    let Value::Array(arr) = value else {
        panic!("Expected array, got {value:?}");
    };

    assert!(arr.is_empty());
}

#[test]
fn string_value() {
    let value = expect_success(r#"{"key": "value"}"#);

    let Value::Object(obj) = value else {
        panic!("Expected object, got {value:?}");
    };

    let Some(Value::String(s)) = obj.get("key") else {
        panic!("Expected string value");
    };

    assert_eq!(*s, "value");
}

#[test]
fn number_value() {
    let value = expect_success(r#"{"num": 42}"#);

    let Value::Object(obj) = value else {
        panic!("Expected object, got {value:?}");
    };

    let Some(Value::Number(n)) = obj.get("num") else {
        panic!("Expected number value");
    };

    assert_eq!(*n, "42");
}

#[test]
fn boolean_values() {
    [("true", true), ("false", false)]
        .iter()
        .for_each(|(input, expected)| {
            let input = format!(r#"{{"flag": {input}}}"#);
            let value = expect_success(&input);

            let Value::Object(obj) = value else {
                panic!("Expected object, got {value:?}");
            };

            let Some(Value::Boolean(b)) = obj.get("flag") else {
                panic!("Expected boolean value");
            };

            assert_eq!(*b, *expected);
        });
}

#[test]
fn null_value() {
    let value = expect_success(r#"{"empty": null}"#);
    let Value::Object(obj) = value else {
        panic!("Expected object, got {value:?}");
    };

    let Some(Value::Null) = obj.get("empty") else {
        panic!("Expected null value");
    };
}

#[test]
fn invalid_start() {
    let error = expect_failure("]");
    assert!(matches!(error, InvalidValue { .. }));
}

#[test]
fn duplicate_key() {
    let error = expect_failure(r#"{"key": 1, "key": 2}"#);
    assert!(matches!(error, DuplicateKey));
}

#[test]
fn missing_colon() {
    let error = expect_failure(r#"{"key" "value"}"#);
    assert!(matches!(error, MissingColon));
}

#[test]
fn invalid_key() {
    let error = expect_failure(r#"{123: "value"}"#);
    assert!(matches!(error, InvalidKey));
}

#[test]
fn trailing_comma() {
    let error = expect_failure(r#"{"key": "value",}"#);
    assert!(matches!(error, TrailingComma));
}
