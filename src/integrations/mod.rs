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