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
pub fn build_schedule(len: usize, order: RevealOrder, rng: &mut impl Rng) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..len).collect();
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

    let non_ws_indices: Vec<usize> = chars
        .iter()
        .enumerate()
        .filter(|(_, &c)| !c.is_whitespace())
        .map(|(i, _)| i)
        .collect();
    let schedule = if config.order == RevealOrder::Random {
        let mut s = non_ws_indices.clone();
        s.shuffle(&mut rng);
        s
    } else {
        non_ws_indices
    };
    let schedule_len = schedule.len();
    let mut locked = vec![false; len];

    // Hide cursor
    stdout.execute(cursor::Hide).ok();

    for step in 0..=schedule_len {
        // Mark next character as locked
        if step > 0 {
            locked[schedule[step - 1]] = true;
        }

        // Move to start of line
        stdout.execute(cursor::MoveToColumn(0)).ok();
        stdout.execute(SetForegroundColor(color)).ok();

        for (i, &real_char) in chars.iter().enumerate() {
            if real_char.is_whitespace() || locked[i] {
                stdout.execute(Print(real_char)).ok();
            } else {
                let noise = random_char(config.charset, &mut rng);
                stdout.execute(Print(noise)).ok();
            }
        }

        stdout.flush().ok();

        if step < schedule_len {
            std::thread::sleep(config.speed);
        }
    }

    stdout.execute(ResetColor).ok();
    stdout.execute(cursor::Show).ok();
    writeln!(stdout).ok();
    stdout.flush().ok();
}
