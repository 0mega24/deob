use deob::animator::{animate_columns, AnimConfig, RevealOrder};
use deob::charset::ResolvedCharSet;
use deob::cli::{AnsiColor, VAlign};
use deob::layout::{
    collect_sgr_codes, compose_layout, parse_markers, propagate_sgr_across_lines, Segment,
};
use std::time::Duration;

fn zero_config() -> AnimConfig {
    AnimConfig {
        speed: Duration::from_millis(0),
        color: AnsiColor::Green,
        charset: ResolvedCharSet::Alnum,
        order: RevealOrder::Ordered,
        scrambles_min: 1,
        scrambles_max: 1,
        valign: VAlign::Top,
    }
}

// ── parse_markers ────────────────────────────────────────────────────────────

#[test]
fn no_markers_returns_single_static() {
    assert_eq!(
        parse_markers("hello world", '~'),
        vec![Segment::Static("hello world".into())]
    );
}

#[test]
fn single_marker_region_mid_line() {
    assert_eq!(
        parse_markers("OS: ~Ubuntu~", '~'),
        vec![
            Segment::Static("OS: ".into()),
            Segment::Scrambled("Ubuntu".into())
        ]
    );
}

#[test]
fn multiple_marker_regions() {
    assert_eq!(
        parse_markers("~A~ mid ~B~", '~'),
        vec![
            Segment::Scrambled("A".into()),
            Segment::Static(" mid ".into()),
            Segment::Scrambled("B".into()),
        ]
    );
}

#[test]
fn marker_region_at_start() {
    assert_eq!(
        parse_markers("~Ubuntu~ 24.04", '~'),
        vec![
            Segment::Scrambled("Ubuntu".into()),
            Segment::Static(" 24.04".into())
        ]
    );
}

#[test]
fn marker_region_at_end() {
    assert_eq!(
        parse_markers("OS: ~Ubuntu~", '~'),
        vec![
            Segment::Static("OS: ".into()),
            Segment::Scrambled("Ubuntu".into())
        ]
    );
}

#[test]
fn whitespace_inside_marker_preserved_as_scrambled_segment() {
    assert_eq!(
        parse_markers("~hello world~", '~'),
        vec![Segment::Scrambled("hello world".into())]
    );
}

#[test]
fn empty_line_returns_empty_vec() {
    assert_eq!(parse_markers("", '~'), vec![]);
}

#[test]
fn custom_marker_char() {
    assert_eq!(
        parse_markers("OS: |Ubuntu|", '|'),
        vec![
            Segment::Static("OS: ".into()),
            Segment::Scrambled("Ubuntu".into())
        ]
    );
}

// ── collect_sgr_codes ───────────────────────────────────────────────────────

#[test]
fn collect_sgr_codes_keeps_only_m_sequences_in_order() {
    assert_eq!(collect_sgr_codes("\x1b[1m\x1b[36mhi"), "\x1b[1m\x1b[36m");
}

// ── propagate_sgr_across_lines ─────────────────────────────────────────────

#[test]
fn propagate_repeats_sgr_on_continuation_lines() {
    let out = propagate_sgr_across_lines(
        vec![
            "\x1b[1m\x1b[36m    '".into(),
            "        'o".into(),
            "        'ooo".into(),
        ],
        '~',
    );
    assert!(out[1].starts_with("\x1b[1m\x1b[36m"), "line 2: {}", out[1]);
    assert!(out[2].starts_with("\x1b[1m\x1b[36m"), "line 3: {}", out[2]);
}

#[test]
fn propagate_skips_when_line_opens_with_escape() {
    let out = propagate_sgr_across_lines(vec!["\x1b[36mx".into(), "\x1b[0my".into()], '~');
    assert_eq!(out[1], "\x1b[0my");
}

#[test]
fn propagate_empty_line_preserves_state() {
    let out = propagate_sgr_across_lines(vec!["\x1b[36ma".into(), "".into(), "b".into()], '~');
    assert_eq!(out[2], "\x1b[36mb");
}

// ── compose_layout ───────────────────────────────────────────────────────────

#[test]
fn equal_heights_padding_applied() {
    let left = vec!["abc".to_string(), "x".to_string()];
    let right = vec!["1".to_string(), "2".to_string()];
    let layout = compose_layout(&[left, right], 2, '~');
    // max left visual width = 3, gap = 2 → 2 padding; "x" (1 wide) → 4 padding
    assert_eq!(layout[0][0].0, "abc");
    assert_eq!(layout[0][0].1, 2);
    assert_eq!(layout[1][0].0, "x");
    assert_eq!(layout[1][0].1, 4);
}

#[test]
fn left_longer_right_padded_with_empty() {
    let left = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let right = vec!["1".to_string()];
    let layout = compose_layout(&[left, right], 1, '~');
    assert_eq!(layout.len(), 3);
    assert_eq!(layout[1][1].0, "");
    assert_eq!(layout[2][1].0, "");
}

#[test]
fn right_longer_left_padded_with_empty() {
    let left = vec!["a".to_string()];
    let right = vec!["1".to_string(), "2".to_string()];
    let layout = compose_layout(&[left, right], 1, '~');
    assert_eq!(layout.len(), 2);
    assert_eq!(layout[1][0].0, "");
}

#[test]
fn ansi_codes_not_counted_in_width() {
    let left = vec!["\x1b[32mabc\x1b[0m".to_string()];
    let right = vec!["1".to_string()];
    let layout = compose_layout(&[left, right], 2, '~');
    assert_eq!(layout[0][0].1, 2);
}

#[test]
fn marker_chars_not_counted_in_width() {
    let left = vec!["~abc~".to_string()];
    let right = vec!["x".to_string()];
    let layout = compose_layout(&[left, right], 1, '~');
    assert_eq!(layout[0][0].1, 1);
}

#[test]
fn three_column_layout() {
    let a = vec!["ab".to_string()];
    let b = vec!["xyz".to_string()];
    let c = vec!["q".to_string()];
    let layout = compose_layout(&[a, b, c], 1, '~');
    assert_eq!(layout.len(), 1);
    // col 0: width 2, gap 1 → padding = 2+1-2 = 1
    assert_eq!(layout[0][0].1, 1);
    // col 1: width 3, gap 1 → padding = 3+1-3 = 1
    assert_eq!(layout[0][1].1, 1);
    // col 2 (last): padding always 0
    assert_eq!(layout[0][2].1, 0);
}

// ── animate_columns ──────────────────────────────────────────────────────────

#[test]
fn animate_cols_output_contains_real_text() {
    let left = vec!["logo".to_string()];
    let right = vec!["OS: ~Ubuntu~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_columns(&[left, right], 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("logo"), "left column text missing");
    assert!(output.contains("OS: "), "static right text missing");
    assert!(
        output.contains("Ubuntu"),
        "scrambled text missing in final state"
    );
}

#[test]
fn animate_cols_no_markers_prints_static() {
    let left = vec!["left".to_string()];
    let right = vec!["right".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_columns(&[left, right], 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("left"));
    assert!(output.contains("right"));
}

#[test]
fn animate_cols_both_columns_with_markers() {
    let left = vec!["~logo~".to_string()];
    let right = vec!["~info~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_columns(&[left, right], 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("logo"));
    assert!(output.contains("info"));
}

#[test]
fn animate_cols_empty_inputs_does_not_panic() {
    let mut buf: Vec<u8> = Vec::new();
    animate_columns(&[vec![], vec![]], 2, '~', &zero_config(), &mut buf);
}

#[test]
fn animate_cols_multiline() {
    let left = vec!["a".to_string(), "b".to_string()];
    let right = vec!["~x~".to_string(), "~y~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_columns(&[left, right], 1, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains('x'));
    assert!(output.contains('y'));
}

#[test]
fn animate_cols_three_columns() {
    let a = vec!["~alpha~".to_string()];
    let b = vec!["static".to_string()];
    let c = vec!["~gamma~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_columns(&[a, b, c], 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("alpha"));
    assert!(output.contains("static"));
    assert!(output.contains("gamma"));
}
