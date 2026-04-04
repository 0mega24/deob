mod cli;
mod charset;
mod animator;
mod layout;

use std::io::{self, BufRead};
use std::time::Duration;

use clap::Parser;
use crossterm::{cursor, ExecutableCommand};

use cli::Args;
use charset::resolve;
use animator::{animate, animate_columns, animate_marked, AnimConfig};

fn read_file_lines(path: &std::path::PathBuf) -> Result<Vec<String>, String> {
    std::fs::read_to_string(path)
        .map(|s| s.lines().map(String::from).collect())
        .map_err(|e| format!("deob: failed to read {}: {}", path.display(), e))
}

fn main() {
    let args = Args::parse();

    ctrlc::set_handler(move || {
        let mut stdout = io::stdout();
        stdout.execute(cursor::Show).ok();
        std::process::exit(0);
    })
    .expect("failed to set Ctrl+C handler");

    let mut stdout = io::stdout();

    // Column mode: --col <file> --col <file> [--col <file> ...]
    if !args.cols.is_empty() {
        if args.cols.len() < 2 {
            eprintln!("deob: --col requires at least 2 columns");
            std::process::exit(1);
        }
        let mut col_lines: Vec<Vec<String>> = Vec::new();
        for path in &args.cols {
            match read_file_lines(path) {
                Ok(lines) => col_lines.push(lines),
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
        }
        let combined =
            col_lines.iter().map(|c| c.join("\n")).collect::<Vec<_>>().join("\n");
        let config = AnimConfig {
            speed: Duration::from_millis(args.speed),
            color: args.color,
            charset: resolve(args.charset, &combined),
            order: args.order,
            scrambles_min: args.scrambles_min,
            scrambles_max: args.scrambles_max,
            valign: args.valign,
        };
        animate_columns(&col_lines, args.gap, args.marker, &config, &mut stdout);
        return;
    }

    // Single-string mode
    let text = match args.text {
        Some(t) => t,
        None => {
            let stdin = io::stdin();
            let mut lines = Vec::new();
            for line in stdin.lock().lines() {
                match line {
                    Ok(l) => lines.push(l),
                    Err(e) => {
                        eprintln!("deob: failed to read stdin: {e}");
                        std::process::exit(1);
                    }
                }
            }
            lines.join("\n")
        }
    };

    if text.trim().is_empty() {
        return;
    }

    // When stdout is not a terminal (e.g. piped), skip animation and pass text through.
    use std::io::IsTerminal;
    if !std::io::stdout().is_terminal() {
        print!("{text}");
        return;
    }

    let resolved_charset = resolve(args.charset, &text);

    let config = AnimConfig {
        speed: Duration::from_millis(args.speed),
        color: args.color,
        charset: resolved_charset,
        order: args.order,
        scrambles_min: args.scrambles_min,
        scrambles_max: args.scrambles_max,
        valign: crate::cli::VAlign::Top,
    };

    if args.markers {
        animate_marked(&text, args.marker, &config, &mut stdout);
    } else {
        animate(&text, &config, &mut stdout);
    }
}
