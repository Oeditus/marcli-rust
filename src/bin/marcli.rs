//! Command-line frontend for marcli.
//!
//! Reads CommonMark Markdown from stdin and writes the rendered result to
//! stdout.  ANSI color sequences are emitted when stdout is a TTY and the
//! `NO_COLOR` environment variable is absent; otherwise plain text is written.
//!
//! # Usage
//!
//!     cat README.md | marcli           # auto-detect (color when TTY)
//!     cat README.md | marcli --color   # force color
//!     cat README.md | marcli --no-color # force plain text
//!
//! The `NO_COLOR` environment variable (https://no-color.org/) is respected
//! when no explicit flag is supplied.

use std::io::{self, IsTerminal, Read, Write};

use marcli::{render, RenderOptions};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Explicit flag wins; otherwise fall back to TTY + NO_COLOR heuristic.
    let use_color = match args.iter().find_map(|a| match a.as_str() {
        "--color" => Some(true),
        "--no-color" => Some(false),
        _ => None,
    }) {
        Some(forced) => forced,
        None => io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    };

    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("marcli: error reading stdin: {e}");
        std::process::exit(1);
    }

    let opts = RenderOptions {
        escape_sequences: use_color,
        ..Default::default()
    };

    let output = render(&input, &opts);

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    if let Err(e) = handle.write_all(output.as_bytes()) {
        if e.kind() != io::ErrorKind::BrokenPipe {
            eprintln!("marcli: error writing output: {e}");
            std::process::exit(1);
        }
    }
    // Trailing newline so the shell prompt appears on its own line.
    let _ = handle.write_all(b"\n");
}
