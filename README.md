# json-parser

Test cases are from [coding challenges](https://codingchallenges.fyi/challenges/challenge-json-parser).

The largest JSON file that I could find is from [here](https://www.kaggle.com/datasets/iyadelwy/the-500mb-tv-show-dataset).

## Usage

Install rust following the official instructions [here](https://www.rust-lang.org/tools/install).

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/howenyap/json-parser.git && cd json-parser
```

Build the project:

```bash
cargo build --release
```

Run the project:

```bash
cargo run --release <file>
```

If you'd like to see the lexed tokens, use the `--verbose / -v` flag to write them to `tokens.txt`:

```bash
cargo run --release <file> --verbose
```

> Note: When running the program on large files with the verbose flag, it may take a long time to write the tokens to file

Run the tests:

```bash
cargo test
```
