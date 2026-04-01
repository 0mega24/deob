mod cli;
mod charset;
mod animator;
mod integrations;

use std::io::{self, BufRead};
use std::time::Duration;

use clap::Parser;
use crossterm::{cursor, ExecutableCommand};

use cli::Args;
use charset::resolve;
use animator::{animate, AnimConfig};

fn main() {
    let args = Args::parse();

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

    if args.scrambles_min > args.scrambles_max {
        eprintln!("deob: --scrambles-min ({}) cannot exceed --scrambles-max ({})", args.scrambles_min, args.scrambles_max);
        std::process::exit(1);
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

    // Ctrl+C: restore cursor before exit
    ctrlc::set_handler(move || {
        let mut stdout = io::stdout();
        stdout.execute(crossterm::style::ResetColor).ok();
        stdout.execute(cursor::Show).ok();
        std::process::exit(0);
    })
    .expect("failed to set Ctrl+C handler");

    let mut stdout = io::stdout();
    animate(&text, &config, &mut stdout);
}
