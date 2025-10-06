use std::fs;
use std::time::Instant;

use humansize::{DECIMAL, format_size};
use json_parser::{Lexer, Parser};
use num_format::{Locale, ToFormattedString};

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    let Some(path) = args.get(1) else {
        eprintln!("Usage: ./json-parser <file> --verbose");
        return;
    };

    let verbose = args
        .get(2)
        .is_some_and(|arg| arg == "--verbose" || arg == "-v");

    let input = match fs::read(path) {
        Ok(input) => input,
        Err(e) => {
            eprintln!("Error reading input: {e}");
            return;
        }
    };

    let mut lexer = Lexer::new(&input);

    let lex_start = Instant::now();
    let tokens = match lexer.lex() {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    let lex_end = lex_start.elapsed();

    if verbose {        
        let s = tokens.iter().map(|t| t.to_string(&input)).collect::<Vec<_>>().join("\n");
        if let Err(e) = fs::write("tokens.txt", s) {
            eprintln!("Error writing tokens to file: {e}");
        }
    }

    let size = format_size(input.len(), DECIMAL);
    let lex_duration = lex_end.as_secs_f64();
    let count = tokens.len().to_formatted_string(&Locale::en);

    let mut parser = Parser::new(tokens, &input);
    let parse_start = Instant::now();
    let value = parser.parse();
    let parse_end = parse_start.elapsed();

    let Some(value) = value else {
        return;
    };

    #[allow(unused_variables)]
    let parsed = match value {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Outcome: invalid");
            eprintln!("{e}");
            return;
        }
    };

    let parse_duration = parse_end.as_secs_f64();
    eprintln!("File size: {size}");
    eprintln!("Tokens: {count}");
    eprintln!("Outcome: valid");
    eprintln!("Time spent:");
    eprintln!("Lexing: {lex_duration}s");
    eprintln!("Parsing: {parse_duration}s");
    eprintln!("Total: {}s", lex_duration + parse_duration);
}
