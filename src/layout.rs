//! Layout helpers: marker parsing, ANSI stripping, column composition.
//! Pure functions — no I/O.

/// Terminal foreground after applying SGR `m` sequences in order (subset of ECMA-48).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum Fg {
    #[default]
    Default,
    Ansi(u8),
    Color256(u8),
    Rgb(u8, u8, u8),
}

/// Bold + foreground snapshot for SGR carry between lines.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct GraphicsState {
    bold: bool,
    fg: Fg,
}

impl GraphicsState {
    fn is_default(&self) -> bool {
        !self.bold && self.fg == Fg::default()
    }

    /// Minimal prefix that reproduces this state (for lines that omit repeated SGR).
    fn to_prefix(&self) -> String {
        if self.is_default() {
            return String::new();
        }
        let mut s = String::new();
        if self.bold {
            s.push_str("\x1b[1m");
        }
        match &self.fg {
            Fg::Default => {}
            Fg::Ansi(n) => {
                s.push_str(&format!("\x1b[{}m", n));
            }
            Fg::Color256(n) => {
                s.push_str(&format!("\x1b[38;5;{}m", n));
            }
            Fg::Rgb(r, g, b) => {
                s.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
            }
        }
        s
    }
}

fn parse_sgr_param_nums(params: &str) -> Vec<usize> {
    if params.is_empty() {
        return vec![0];
    }
    params
        .split(';')
        .map(|p| {
            if p.is_empty() {
                0
            } else {
                p.parse().unwrap_or(0)
            }
        })
        .collect()
}

fn apply_sgr_params(nums: &[usize], state: &mut GraphicsState) {
    let mut i = 0usize;
    while i < nums.len() {
        match nums[i] {
            0 => *state = GraphicsState::default(),
            1 => state.bold = true,
            22 => state.bold = false,
            30..=37 | 90..=97 => state.fg = Fg::Ansi(nums[i] as u8),
            38 if i + 2 < nums.len() && nums[i + 1] == 5 => {
                state.fg = Fg::Color256(nums[i + 2] as u8);
                i += 2;
            }
            38 if i + 4 < nums.len() && nums[i + 1] == 2 => {
                state.fg = Fg::Rgb(nums[i + 2] as u8, nums[i + 3] as u8, nums[i + 4] as u8);
                i += 4;
            }
            39 => state.fg = Fg::Default,
            _ => {}
        }
        i += 1;
    }
}

/// Apply all CSI SGR (`…m`) sequences in order (same scan as other helpers).
fn parse_sgr_line(line: &str) -> GraphicsState {
    let mut state = GraphicsState::default();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            let mut params = String::new();
            while let Some(&c) = chars.peek() {
                chars.next();
                if c.is_ascii_alphabetic() {
                    if c == 'm' {
                        let nums = parse_sgr_param_nums(&params);
                        apply_sgr_params(&nums, &mut state);
                    }
                    break;
                }
                params.push(c);
            }
        }
    }
    state
}

fn line_should_inherit_sgr(line: &str, state: &GraphicsState) -> bool {
    if state.is_default() {
        return false;
    }
    !line.trim_start().starts_with("\x1b[")
}

/// Repeat SGR foreground/bold on continuation lines when the source omits it (e.g. fastfetch logos).
/// Visually empty lines do not change the carried state (so blank padding rows do not reset color).
pub fn propagate_sgr_across_lines(lines: Vec<String>, marker: char) -> Vec<String> {
    let mut state = GraphicsState::default();
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        if visual_width(&line, marker) == 0 {
            out.push(line);
            continue;
        }
        let line_out = if line_should_inherit_sgr(&line, &state) {
            state.to_prefix() + &line
        } else {
            line
        };
        state = parse_sgr_line(&line_out);
        out.push(line_out);
    }
    out
}

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
