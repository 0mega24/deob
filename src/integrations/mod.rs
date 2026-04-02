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

/// Visual display width: strip ANSI codes and marker characters.
fn visual_width(s: &str, marker: char) -> usize {
    strip_ansi(s).chars().filter(|&c| c != marker).count()
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