//! Layout helpers: marker parsing, ANSI stripping, column composition.
//! Pure functions — no I/O.

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

/// Strip all ANSI escape sequences for width measurement.
pub fn strip_ansi(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
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

/// Strip cursor-movement ANSI codes (A–H, J–l, n–z) but keep SGR color codes (ending in 'm').
pub fn strip_cursor_codes(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            let mut params = String::new();
            while let Some(&c) = chars.peek() {
                chars.next();
                if c.is_ascii_alphabetic() {
                    if c == 'm' {
                        result.push('\x1b');
                        result.push('[');
                        result.push_str(&params);
                        result.push('m');
                    }
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
pub fn visual_width(s: &str, marker: char) -> usize {
    strip_ansi(s).chars().filter(|&c| c != marker).count()
}

/// Trim trailing lines that have no visible content (empty or ANSI-only).
pub fn trim_trailing_empty(lines: &[String], marker: char) -> &[String] {
    let end = lines
        .iter()
        .rposition(|l| visual_width(l, marker) > 0)
        .map(|i| i + 1)
        .unwrap_or(0);
    &lines[..end]
}

/// Extract (text_char, preceding_ansi_codes) pairs from a string.
/// ANSI escape sequences are collected into `pending` and attached to the next real char.
pub fn chars_with_ansi_context(s: &str) -> Vec<(char, String)> {
    let mut result = Vec::new();
    let mut pending = String::new();
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            pending.push('\x1b');
            chars.next();
            pending.push('[');
            while let Some(&c) = chars.peek() {
                chars.next();
                pending.push(c);
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push((ch, std::mem::take(&mut pending)));
        }
    }
    result
}

/// Truncate a string to at most `max_width` visible characters, preserving ANSI codes.
pub fn truncate_to_visual_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut visible = 0;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            let mut seq = String::from("\x1b[");
            chars.next();
            while let Some(&c) = chars.peek() {
                chars.next();
                seq.push(c);
                if c.is_ascii_alphabetic() {
                    break;
                }
            }
            result.push_str(&seq);
        } else {
            if visible >= max_width {
                break;
            }
            result.push(ch);
            visible += 1;
        }
    }
    result
}

/// Pair N column slices with computed per-row padding.
/// Returns rows × cols: each `(content, padding_after_this_col)`.
/// The last column always has padding = 0.
/// Shorter columns are padded with empty strings to match the tallest column.
pub fn compose_layout(
    cols: &[Vec<String>],
    gap: usize,
    marker: char,
) -> Vec<Vec<(String, usize)>> {
    let n = cols.len();
    if n == 0 {
        return Vec::new();
    }
    let max_widths: Vec<usize> = cols
        .iter()
        .map(|col| col.iter().map(|l| visual_width(l, marker)).max().unwrap_or(0))
        .collect();
    let height = cols.iter().map(|c| c.len()).max().unwrap_or(0);
    let empty = String::new();

    (0..height)
        .map(|row| {
            (0..n)
                .map(|ci| {
                    let content = cols[ci].get(row).unwrap_or(&empty);
                    let w = visual_width(content, marker);
                    let padding = if ci + 1 < n { max_widths[ci] + gap - w } else { 0 };
                    (content.clone(), padding)
                })
                .collect()
        })
        .collect()
}
