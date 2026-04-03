use std::io::Write;
use std::time::Duration;

use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use rand::seq::SliceRandom;
use rand::Rng;

use crate::charset::{random_char, ResolvedCharSet};
use crate::cli::{AnsiColor, VAlign};
use crate::layout::{
    chars_with_ansi_context, compose_layout, parse_markers, strip_ansi, strip_cursor_codes,
    trim_trailing_empty, truncate_to_visual_width, visual_width, Segment,
};

pub use crate::cli::RevealOrder;

pub struct AnimConfig {
    pub speed: Duration,
    pub color: AnsiColor,
    pub charset: ResolvedCharSet,
    pub order: RevealOrder,
    pub scrambles_min: u32,
    pub scrambles_max: u32,
    pub valign: VAlign,
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
pub fn build_schedule(
    candidate_indices: Vec<usize>,
    order: RevealOrder,
    rng: &mut impl Rng,
) -> Vec<usize> {
    let mut indices = candidate_indices;
    if order == RevealOrder::Random {
        indices.shuffle(rng);
    }
    indices
}

// ── Single-string animation ──────────────────────────────────────────────────

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
    let schedule = build_schedule(non_ws_indices, config.order.clone(), &mut rng);
    let mut locked = vec![false; len];

    stdout.execute(cursor::Hide).ok();

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

// ── Multi-column animation ───────────────────────────────────────────────────

struct ScrambleChar {
    real: char,
    lock_frame: usize,
    /// ANSI codes immediately preceding this char; emitted on lock-in to restore original color.
    color_before: String,
}

enum ReadySegment {
    Static(String),
    Scrambled(Vec<ScrambleChar>),
}

fn build_ready_line(
    segs: Vec<Segment>,
    total_frames: usize,
    order: &RevealOrder,
    rng: &mut impl Rng,
) -> Vec<ReadySegment> {
    segs.into_iter()
        .map(|seg| match seg {
            Segment::Static(s) => ReadySegment::Static(s),
            Segment::Scrambled(s) => {
                let text_chars = chars_with_ansi_context(&s);
                let n = text_chars.len();
                let mut frames: Vec<usize> =
                    (0..n).map(|_| rng.gen_range(1..=total_frames)).collect();
                if *order == RevealOrder::Ordered {
                    frames.sort_unstable();
                }
                ReadySegment::Scrambled(
                    text_chars
                        .into_iter()
                        .zip(frames)
                        .map(|((c, color_before), f)| ScrambleChar {
                            real: c,
                            lock_frame: f,
                            color_before,
                        })
                        .collect(),
                )
            }
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
                        if !sc.color_before.is_empty() {
                            stdout.execute(Print(&sc.color_before)).ok();
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
    cols: &[&[ReadySegment]],
    paddings: &[usize],
    frame: usize,
    charset: ResolvedCharSet,
    anim_color: Color,
    rng: &mut impl Rng,
) {
    let mut in_anim = false;
    for (segs, &padding) in cols.iter().zip(paddings.iter()) {
        render_segs(stdout, segs, frame, charset, anim_color, &mut in_anim, rng);
        if in_anim {
            stdout.execute(ResetColor).ok();
            in_anim = false;
        }
        stdout.execute(Print(" ".repeat(padding))).ok();
    }
    stdout.execute(Print('\n')).ok();
}

pub fn animate_columns(
    cols: &[Vec<String>],
    gap: usize,
    marker: char,
    config: &AnimConfig,
    stdout: &mut impl Write,
) {
    if cols.is_empty() {
        return;
    }

    // 1. Trim trailing empty lines per column.
    let trimmed: Vec<Vec<String>> = cols
        .iter()
        .map(|col| trim_trailing_empty(col, marker).to_vec())
        .collect();

    // 2. Vertical centering.
    let cols: Vec<Vec<String>> = match config.valign {
        VAlign::Center => {
            let max_h = trimmed.iter().map(|c| c.len()).max().unwrap_or(0);
            trimmed
                .iter()
                .map(|col| {
                    let p = (max_h - col.len()) / 2;
                    std::iter::repeat(String::new())
                        .take(p)
                        .chain(col.iter().cloned())
                        .collect()
                })
                .collect()
        }
        VAlign::Top => trimmed,
    };

    // 3. Terminal width truncation — each column gets whatever space remains after previous ones.
    let cols: Vec<Vec<String>> = if let Ok((term_w, _)) = crossterm::terminal::size() {
        let max_ws: Vec<usize> = cols
            .iter()
            .map(|col| col.iter().map(|l| visual_width(l, marker)).max().unwrap_or(0))
            .collect();
        let mut used = 0usize;
        cols.iter()
            .enumerate()
            .map(|(ci, col)| {
                let avail = (term_w as usize).saturating_sub(used);
                let col_out = col
                    .iter()
                    .map(|l| {
                        if visual_width(l, marker) > avail {
                            truncate_to_visual_width(l, avail)
                        } else {
                            l.clone()
                        }
                    })
                    .collect();
                used += max_ws[ci].min(avail);
                if ci + 1 < cols.len() {
                    used = used.saturating_add(gap);
                }
                col_out
            })
            .collect()
    } else {
        cols
    };

    // 4. Compose layout: rows × cols of (content, padding_after).
    let layout = compose_layout(&cols, gap, marker);
    let n_lines = layout.len();
    if n_lines == 0 {
        return;
    }

    let mut rng = rand::thread_rng();

    // Parse segments and collect paddings.
    let parsed: Vec<Vec<Vec<Segment>>> = layout
        .iter()
        .map(|row| {
            row.iter()
                .map(|(content, _)| parse_markers(&strip_cursor_codes(content), marker))
                .collect()
        })
        .collect();
    let paddings: Vec<Vec<usize>> =
        layout.iter().map(|row| row.iter().map(|(_, p)| *p).collect()).collect();

    // Max visible chars in any scrambled segment (drives total_frames).
    let max_chars = parsed
        .iter()
        .flatten()
        .flatten()
        .filter_map(|seg| {
            if let Segment::Scrambled(s) = seg {
                Some(strip_ansi(s).chars().count())
            } else {
                None
            }
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

    let ready: Vec<Vec<Vec<ReadySegment>>> = parsed
        .into_iter()
        .map(|row| {
            row.into_iter()
                .map(|segs| {
                    build_ready_line(segs, total_frames.max(1), &config.order, &mut rng)
                })
                .collect()
        })
        .collect();

    // Reserve vertical space so the final newline never causes a scroll that
    // breaks MoveUp tracking.
    for _ in 0..n_lines {
        stdout.execute(Print('\n')).ok();
    }
    stdout.execute(cursor::MoveUp(n_lines as u16)).ok();

    if max_chars == 0 {
        for (i, row) in ready.iter().enumerate() {
            stdout.execute(cursor::MoveToColumn(0)).ok();
            let col_segs: Vec<&[ReadySegment]> = row.iter().map(|c| c.as_slice()).collect();
            render_row(stdout, &col_segs, &paddings[i], 1, config.charset, color, &mut rng);
        }
        stdout.execute(cursor::Show).ok();
        stdout.flush().ok();
        return;
    }

    // Initial frame — all scrambled chars show noise (lock_frame all >= 1).
    for (i, row) in ready.iter().enumerate() {
        stdout.execute(cursor::MoveToColumn(0)).ok();
        let col_segs: Vec<&[ReadySegment]> = row.iter().map(|c| c.as_slice()).collect();
        render_row(stdout, &col_segs, &paddings[i], 0, config.charset, color, &mut rng);
    }
    stdout.flush().ok();
    std::thread::sleep(config.speed);

    // Frames 1..=total_frames.
    for frame in 1..=total_frames {
        stdout.execute(cursor::MoveUp(n_lines as u16)).ok();
        for (i, row) in ready.iter().enumerate() {
            stdout.execute(cursor::MoveToColumn(0)).ok();
            let col_segs: Vec<&[ReadySegment]> = row.iter().map(|c| c.as_slice()).collect();
            render_row(stdout, &col_segs, &paddings[i], frame, config.charset, color, &mut rng);
        }
        stdout.flush().ok();
        if frame < total_frames {
            std::thread::sleep(config.speed);
        }
    }

    stdout.execute(cursor::Show).ok();
    stdout.flush().ok();
}
