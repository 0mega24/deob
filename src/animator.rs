use std::io::Write;
use std::time::Duration;

use crossterm::{
    cursor,
    style::{Color, Print, SetForegroundColor, ResetColor},
    ExecutableCommand,
};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::charset::{random_char, ResolvedCharSet};
use crate::cli::AnsiColor;

pub use crate::cli::RevealOrder;

pub struct AnimConfig {
    pub speed: Duration,
    pub color: AnsiColor,
    pub charset: ResolvedCharSet,
    pub order: RevealOrder,
    pub scrambles_min: u32,
    pub scrambles_max: u32,
}

fn to_crossterm_color(color: &AnsiColor) -> Color {
    match color {
        AnsiColor::Black => Color::Black,
        AnsiColor::Red => Color::DarkRed,
        AnsiColor::Green => Color::DarkGreen,
        AnsiColor::Yellow => Color::DarkYellow,
        AnsiColor::Blue => Color::DarkBlue,
        AnsiColor::Magenta => Color::DarkMagenta,
        AnsiColor::Cyan => Color::DarkCyan,
        AnsiColor::White => Color::White,
    }
}

#[allow(dead_code)]
pub fn build_schedule(candidate_indices: Vec<usize>, order: RevealOrder, rng: &mut impl Rng) -> Vec<usize> {
    let mut indices = candidate_indices;
    if order == RevealOrder::Random {
        indices.shuffle(rng);
    }
    indices
}

pub fn animate(text: &str, config: &AnimConfig, stdout: &mut impl Write) {
    if text.is_empty() {
        return;
    }

    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut rng = rand::thread_rng();
    let color = to_crossterm_color(&config.color);

    // Build non-whitespace indices for reveal schedule
    let non_ws_indices: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, &c)| !c.is_whitespace())
        .map(|(i, _)| i)
        .collect();
    let schedule = build_schedule(non_ws_indices, config.order.clone(), &mut rng);
    let mut locked = vec![false; len];

    // Hide cursor
    stdout.execute(cursor::Hide).ok();

    // Initial frame: all noise (no chars locked yet)
    stdout.execute(cursor::MoveToColumn(0)).ok();
    stdout.execute(SetForegroundColor(color)).ok();
    for (i, &real_char) in chars.iter().enumerate() {
        if real_char.is_whitespace() || locked[i] {
            stdout.execute(Print(real_char)).ok();
        } else {
            stdout.execute(Print(random_char(config.charset, &mut rng))).ok();
        }
    }
    stdout.flush().ok();
    std::thread::sleep(config.speed);

    // For each char in reveal schedule: pick random frame count, churn, then lock
    for &idx in &schedule {
        let n = rng.gen_range(config.scrambles_min..=config.scrambles_max);
        for _ in 0..n {
            stdout.execute(cursor::MoveToColumn(0)).ok();
            stdout.execute(SetForegroundColor(color)).ok();
            for (i, &real_char) in chars.iter().enumerate() {
                if real_char.is_whitespace() || locked[i] {
                    stdout.execute(Print(real_char)).ok();
                } else {
                    stdout.execute(Print(random_char(config.charset, &mut rng))).ok();
                }
            }
            stdout.flush().ok();
            std::thread::sleep(config.speed);
        }
        locked[idx] = true;
    }

    // Final frame: all real chars
    stdout.execute(cursor::MoveToColumn(0)).ok();
    stdout.execute(SetForegroundColor(color)).ok();
    for &real_char in &chars {
        stdout.execute(Print(real_char)).ok();
    }
    stdout.flush().ok();

    stdout.execute(ResetColor).ok();
    stdout.execute(cursor::Show).ok();
    writeln!(stdout).ok();
    stdout.flush().ok();
}
