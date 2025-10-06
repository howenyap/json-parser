use std::fmt::Display;
use std::path::PathBuf;
use std::time::Instant;
use std::{fs, thread};

use clap::Parser as ClapParser;
use humansize::{DECIMAL, format_size};
use json_parser::{Lexer, Parser};
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;

#[derive(ClapParser)]
#[command(name = "json-parser")]
#[command(about = "A JSON parser written in Rust")]
#[command(version)]
struct Args {
    #[arg(required = true)]
    files: Vec<PathBuf>,

    #[arg(short, long, help = "Show detailed statistics and timing information")]
    verbose: bool,

    #[arg(short, long, help = "Writes tokens to <filename>-tokens.txt")]
    tokens: bool,

    #[arg(short, long, help = "Processes files sequentially using 1 thread")]
    sequential: bool,
}

fn main() {
    let args = Args::parse();

    let num_threads = if args.sequential {
        1
    } else {
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
            .clamp(1, args.files.len())
    };

    let results: Vec<Result<ParseResult, String>> = if num_threads == 1 {
        println!("Processing {} files sequentially", args.files.len());

        args.files
            .iter()
            .map(|file| parse_file(file, args.tokens))
            .collect()
    } else {
        println!(
            "Processing {} files in parallel using {} threads",
            args.files.len(),
            num_threads
        );

        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .expect("Failed to build thread pool");

        pool.install(|| {
            args.files
                .par_iter()
                .map(|file| parse_file(file, args.tokens))
                .collect()
        })
    };

    results
        .iter()
        .zip(args.files.iter())
        .filter_map(|(result, file)| {
            result
                .as_ref()
                .err()
                .map(|e| (file.display().to_string(), e.to_string()))
        })
        .for_each(|(file, error)| {
            eprintln!("Failed to process {file}: {error}");
        });

    let results: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();

    results.iter().for_each(|result| {
        result.print(args.verbose);
    });

    let valid_files = {
        let s = results
            .iter()
            .filter(|r| r.outcome.is_valid())
            .map(|r| r.file_path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        if s.is_empty() { "none".to_string() } else { s }
    };

    let invalid_files = {
        let s = results
            .iter()
            .filter(|r| !r.outcome.is_valid())
            .map(|r| r.file_path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        if s.is_empty() { "none".to_string() } else { s }
    };

    println!("Total files: {}", results.len());
    println!("Valid files: {valid_files}");
    println!("Invalid files: {invalid_files}");
}

#[derive(Debug, PartialEq)]
enum Outcome {
    Valid,
    Invalid { error_message: String },
}

impl Outcome {
    pub fn is_valid(&self) -> bool {
        matches!(self, Outcome::Valid)
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Outcome::Valid => "valid".to_string(),
            Outcome::Invalid { error_message } => {
                format!("invalid\nError message: {error_message}")
            }
        };

        write!(f, "{s}")
    }
}

#[derive(Debug)]
struct ParseResult {
    outcome: Outcome,
    file_path: PathBuf,
    file_size: usize,
    token_count: usize,
    lex_duration: f64,
    parse_duration: f64,
}

impl ParseResult {
    pub fn new(
        file_path: PathBuf,
        file_size: usize,
        token_count: usize,
        outcome: Outcome,
        lex_duration: f64,
        parse_duration: f64,
    ) -> Self {
        Self {
            file_path,
            file_size,
            token_count,
            outcome,
            lex_duration,
            parse_duration,
        }
    }

    pub fn print(&self, verbose: bool) {
        let file_name = self.file_path.display();

        println!("File: {file_name}");
        println!("Outcome: {}", self.outcome);

        if verbose {
            let size = format_size(self.file_size, DECIMAL);
            let count = self.token_count.to_formatted_string(&Locale::en);

            println!("File size: {size}");
            println!("Tokens: {count}");
            println!("Time spent:");
            println!("Lexing: {:.6}s", self.lex_duration);
            println!("Parsing: {:.6}s", self.parse_duration);
            println!("Total: {:.6}s", self.lex_duration + self.parse_duration);
        }

        println!();
    }
}

fn parse_file(file_path: &PathBuf, output_tokens: bool) -> Result<ParseResult, String> {
    let input = fs::read(file_path).map_err(|e| format!("Error reading file: {e}"))?;
    let mut lexer = Lexer::new(&input);

    let lex_start = Instant::now();
    let tokens = match lexer.lex() {
        Ok(tokens) => tokens,
        Err(e) => {
            return Ok(ParseResult::new(
                file_path.clone(),
                input.len(),
                0,
                Outcome::Invalid {
                    error_message: e.to_string(),
                },
                lex_start.elapsed().as_secs_f64(),
                0.0,
            ));
        }
    };
    let lex_duration = lex_start.elapsed().as_secs_f64();

    if output_tokens {
        let tokens_filename = format!(
            "{}-tokens.txt",
            file_path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("Failed to get file stem")
        );

        let s = tokens
            .iter()
            .map(|t| t.to_string(&input))
            .collect::<Vec<_>>()
            .join("\n");

        if let Err(e) = fs::write(&tokens_filename, s) {
            return Err(format!("Error writing tokens to {tokens_filename}: {e}"));
        }
    }

    let token_count = tokens.len();
    let mut parser = Parser::new(tokens, &input);
    let parse_start = Instant::now();
    let value = parser.parse();
    let parse_duration = parse_start.elapsed().as_secs_f64();

    let Some(value) = value else {
        return Ok(ParseResult::new(
            file_path.clone(),
            input.len(),
            token_count,
            Outcome::Valid,
            lex_duration,
            parse_duration,
        ));
    };

    let outcome = match value {
        Ok(_) => Outcome::Valid,
        Err(e) => Outcome::Invalid {
            error_message: e.to_string(),
        },
    };

    Ok(ParseResult::new(
        file_path.clone(),
        input.len(),
        token_count,
        outcome,
        lex_duration,
        parse_duration,
    ))
}
