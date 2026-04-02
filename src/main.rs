mod cli;
mod charset;
mod animator;
mod integrations;

use std::io::{self, BufRead};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use clap::Parser;
use crossterm::{cursor, ExecutableCommand};

use cli::Args;
use charset::resolve;
use animator::{animate, AnimConfig};

fn read_file_lines(path: &std::path::PathBuf) -> Result<Vec<String>, String> {
    std::fs::read_to_string(path)
        .map(|s| s.lines().map(String::from).collect())
        .map_err(|e| format!("deob: failed to read {}: {}", path.display(), e))
}

fn main() {
    let args = Args::parse();

    let _interrupted = Arc::new(AtomicBool::new(false));
    ctrlc::set_handler(move || {
        let mut stdout = io::stdout();
        stdout.execute(cursor::Show).ok();
        std::process::exit(0);
    })
    .expect("failed to set Ctrl+C handler");

    let mut stdout = io::stdout();

    // Side-by-side mode
    if args.left.is_some() || args.right.is_some() {
        let (left_path, right_path) = match (args.left, args.right) {
            (Some(l), Some(r)) => (l, r),
            _ => {
                eprintln!("deob: --left and --right must both be provided together");
                std::process::exit(1);
            }
        };

        let left_lines = match read_file_lines(&left_path) {
            Ok(l) => l,
            Err(e) => { eprintln!("{e}"); std::process::exit(1); }
        };
        let right_lines = match read_file_lines(&right_path) {
            Ok(r) => r,
            Err(e) => { eprintln!("{e}"); std::process::exit(1); }
        };

        let config = AnimConfig {
            speed: Duration::from_millis(args.speed),
            color: args.color,
            charset: resolve(args.charset, &right_lines.join("\n")),
            order: args.order,
            scrambles_min: args.scrambles_min,
            scrambles_max: args.scrambles_max,
        };

        integrations::animate_side_by_side(
            &left_lines,
            &right_lines,
            args.gap,
            args.marker,
            &config,
            &mut stdout,
        );
        return;
    }

    // Single-string mode (unchanged)
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

    let resolved_charset = resolve(args.charset, &text);

    let config = AnimConfig {
        speed: Duration::from_millis(args.speed),
        color: args.color,
        charset: resolved_charset,
        order: args.order,
        scrambles_min: args.scrambles_min,
        scrambles_max: args.scrambles_max,
    };

    animate(&text, &config, &mut stdout);
}
