//! Integration extension point.
//!
//! Side-by-side layout mode for ricing integrations (fastfetch, starship, etc.).
//! The `animate_side_by_side` function accepts any `std::io::Write` impl,
//! so integrations can provide their own output targets.

#[derive(Debug, PartialEq)]
pub enum Segment {
    Static(String),
    Scrambled(String),
}

/// Parse a line into static and scrambled segments.
/// Text enclosed in `marker` characters is scrambled; the rest is static.
/// Marker characters are stripped from the output.
pub fn parse_markers(line: &str, marker: char) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_scramble = false;

    for ch in line.chars() {
        if ch == marker {
            if !current.is_empty() {
                if in_scramble {
                    segments.push(Segment::Scrambled(std::mem::take(&mut current)));
                } else {
                    segments.push(Segment::Static(std::mem::take(&mut current)));
                }
            }
            in_scramble = !in_scramble;
        } else {
            current.push(ch);
        }
    }

    if !current.is_empty() {
        if in_scramble {
            segments.push(Segment::Scrambled(current));
        } else {
            segments.push(Segment::Static(current));
        }
    }

    segments
}

/// Strip ANSI escape sequences (e.g. `\x1b[32m`) for width measurement.
fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            while let Some(&c) = chars.peek() {
                chars.next();
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Strip cursor-movement ANSI codes (A-H, J-l, n-z) but keep SGR color codes (ending in 'm').
/// fastfetch logo output embeds cursor-positioning sequences to set up side-by-side rendering;
/// those must be removed before deob prints the lines or they corrupt cursor positioning.
fn strip_cursor_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            let mut params = String::new();
            while let Some(&c) = chars.peek() {
                chars.next();
                if c.is_ascii_alphabetic() {
                    if c == 'm' {
                        // SGR color code — preserve it
                        result.push('\x1b');
                        result.push('[');
                        result.push_str(&params);
                        result.push('m');
                    }
                    // Any other letter (A-H = cursor move, J/K = erase, etc.) — discard
                    break;
                }
                params.push(c);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Visual display width: strip ANSI codes and marker characters.
fn visual_width(s: &str, marker: char) -> usize {
    strip_ansi(s).chars().filter(|&c| c != marker).count()
}

/// Trim trailing lines that have no visible content (empty or ANSI-only).
fn trim_trailing_empty<'a>(lines: &'a [String], marker: char) -> &'a [String] {
    let end = lines
        .iter()
        .rposition(|l| visual_width(l, marker) > 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    &lines[..end]
}

/// Pair left and right column lines with computed per-row padding.
/// Returns `Vec<(left_raw, padding_spaces, right_raw)>`.
/// The shorter column is padded with empty strings to match the taller column.
pub fn compose_layout(
    left: &[String],
    right: &[String],
    gap: usize,
    marker: char,
) -> Vec<(String, usize, String)> {
    let max_left_width = left.iter().map(|l| visual_width(l, marker)).max().unwrap_or(0);
    let height = left.len().max(right.len());
    let empty = String::new();

    (0..height)
        .map(|i| {
            let left_line = left.get(i).unwrap_or(&empty);
            let right_line = right.get(i).unwrap_or(&empty);
            let w = visual_width(left_line, marker);
            let padding = max_left_width + gap - w;
            (left_line.clone(), padding, right_line.clone())
        })
        .collect()
}

use std::io::Write;

use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use rand::Rng;

use crate::animator::{AnimConfig, RevealOrder};
use crate::charset::{random_char, ResolvedCharSet};
use crate::cli::AnsiColor;

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

struct ScrambleChar {
    real: char,
    lock_frame: usize,
}

enum ReadySegment {
    Static(String),
    Scrambled(Vec<ScrambleChar>),
}

fn build_ready(
    segs: Vec<Vec<Segment>>,
    total_frames: usize,
    order: &RevealOrder,
    rng: &mut impl Rng,
) -> Vec<Vec<ReadySegment>> {
    segs.into_iter()
        .map(|line| {
            line.into_iter()
                .map(|seg| match seg {
                    Segment::Static(s) => ReadySegment::Static(s),
                    Segment::Scrambled(s) => {
                        let chars: Vec<char> = s.chars().collect();
                        let n = chars.len();
                        let mut frames: Vec<usize> = (0..n)
                            .map(|_| rng.gen_range(1..=total_frames))
                            .collect();
                        if *order == RevealOrder::Ordered {
                            frames.sort_unstable();
                        }
                        ReadySegment::Scrambled(
                            chars
                                .into_iter()
                                .zip(frames)
                                .map(|(c, f)| ScrambleChar { real: c, lock_frame: f })
                                .collect(),
                        )
                    }
                })
                .collect()
        })
        .collect()
}

fn render_segs(
    stdout: &mut impl Write,
    segs: &[ReadySegment],
    frame: usize,
    charset: ResolvedCharSet,
    anim_color: Color,
    in_anim: &mut bool,
    rng: &mut impl Rng,
) {
    for seg in segs {
        match seg {
            ReadySegment::Static(s) => {
                if *in_anim {
                    stdout.execute(ResetColor).ok();
                    *in_anim = false;
                }
                stdout.execute(Print(s)).ok();
            }
            ReadySegment::Scrambled(chars) => {
                for sc in chars {
                    if sc.lock_frame <= frame || sc.real.is_whitespace() {
                        if *in_anim {
                            stdout.execute(ResetColor).ok();
                            *in_anim = false;
                        }
                        stdout.execute(Print(sc.real)).ok();
                    } else {
                        if !*in_anim {
                            stdout.execute(SetForegroundColor(anim_color)).ok();
                            *in_anim = true;
                        }
                        stdout.execute(Print(random_char(charset, rng))).ok();
                    }
                }
            }
        }
    }
}

fn render_row(
    stdout: &mut impl Write,
    left_segs: &[ReadySegment],
    padding: usize,
    right_segs: &[ReadySegment],
    frame: usize,
    charset: ResolvedCharSet,
    anim_color: Color,
    rng: &mut impl Rng,
) {
    let mut in_anim = false;
    render_segs(stdout, left_segs, frame, charset, anim_color, &mut in_anim, rng);
    if in_anim {
        stdout.execute(ResetColor).ok();
        in_anim = false;
    }
    stdout.execute(Print(" ".repeat(padding))).ok();
    render_segs(stdout, right_segs, frame, charset, anim_color, &mut in_anim, rng);
    if in_anim {
        stdout.execute(ResetColor).ok();
    }
    stdout.execute(Print('\n')).ok();
}

pub fn animate_side_by_side(
    left_lines: &[String],
    right_lines: &[String],
    gap: usize,
    marker: char,
    config: &AnimConfig,
    stdout: &mut impl Write,
) {
    let left_lines = trim_trailing_empty(left_lines, marker);
    let right_lines = trim_trailing_empty(right_lines, marker);
    let layout = compose_layout(left_lines, right_lines, gap, marker);
    let n_lines = layout.len();
    if n_lines == 0 {
        return;
    }

    let mut rng = rand::thread_rng();

    let left_segs: Vec<Vec<Segment>> =
        layout.iter().map(|(l, _, _)| parse_markers(&strip_cursor_codes(l), marker)).collect();
    let right_segs: Vec<Vec<Segment>> =
        layout.iter().map(|(_, _, r)| parse_markers(&strip_cursor_codes(r), marker)).collect();
    let paddings: Vec<usize> = layout.iter().map(|(_, p, _)| *p).collect();

    let max_chars = left_segs
        .iter()
        .chain(right_segs.iter())
        .flat_map(|line| line.iter())
        .filter_map(|seg| {
            if let Segment::Scrambled(s) = seg { Some(s.chars().count()) } else { None }
        })
        .max()
        .unwrap_or(0);

    let color = to_crossterm_color(&config.color);
    stdout.execute(cursor::Hide).ok();

    let per_char = if max_chars == 0 {
        1
    } else {
        rng.gen_range(config.scrambles_min..=config.scrambles_max) as usize
    };
    let total_frames = max_chars * per_char;

    let left_ready = build_ready(left_segs, total_frames.max(1), &config.order, &mut rng);
    let right_ready = build_ready(right_segs, total_frames.max(1), &config.order, &mut rng);

    // Reserve vertical space so \n on the last line never causes a scroll that
    // breaks MoveUp tracking.
    for _ in 0..n_lines {
        stdout.execute(Print('\n')).ok();
    }
    stdout.execute(cursor::MoveUp(n_lines as u16)).ok();

    if max_chars == 0 {
        for (i, left_row) in left_ready.iter().enumerate() {
            stdout.execute(cursor::MoveToColumn(0)).ok();
            render_row(stdout, left_row, paddings[i], &right_ready[i], 1, config.charset, color, &mut rng);
        }
        stdout.execute(cursor::Show).ok();
        stdout.flush().ok();
        return;
    }

    // Initial frame — all scrambled chars show noise (lock_frames all >= 1)
    for (i, left_row) in left_ready.iter().enumerate() {
        stdout.execute(cursor::MoveToColumn(0)).ok();
        render_row(stdout, left_row, paddings[i], &right_ready[i], 0, config.charset, color, &mut rng);
    }
    stdout.flush().ok();
    std::thread::sleep(config.speed);

    // Frames 1..=total_frames
    for frame in 1..=total_frames {
        stdout.execute(cursor::MoveUp(n_lines as u16)).ok();
        for (i, left_row) in left_ready.iter().enumerate() {
            stdout.execute(cursor::MoveToColumn(0)).ok();
            render_row(stdout, left_row, paddings[i], &right_ready[i], frame, config.charset, color, &mut rng);
        }
        stdout.flush().ok();
        if frame < total_frames {
            std::thread::sleep(config.speed);
        }
    }

    stdout.execute(cursor::Show).ok();
    stdout.flush().ok();
}
