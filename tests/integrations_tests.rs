use deob::integrations::{compose_layout, parse_markers, Segment};

#[test]
fn no_markers_returns_single_static() {
    assert_eq!(parse_markers("hello world", '~'), vec![Segment::Static("hello world".into())]);
}

#[test]
fn single_marker_region_mid_line() {
    assert_eq!(
        parse_markers("OS: ~Ubuntu~", '~'),
        vec![Segment::Static("OS: ".into()), Segment::Scrambled("Ubuntu".into())]
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
        vec![Segment::Scrambled("Ubuntu".into()), Segment::Static(" 24.04".into())]
    );
}

#[test]
fn marker_region_at_end() {
    assert_eq!(
        parse_markers("OS: ~Ubuntu~", '~'),
        vec![Segment::Static("OS: ".into()), Segment::Scrambled("Ubuntu".into())]
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
        vec![Segment::Static("OS: ".into()), Segment::Scrambled("Ubuntu".into())]
    );
}

#[test]
fn equal_heights_padding_applied() {
    let left = vec!["abc".to_string(), "x".to_string()];
    let right = vec!["1".to_string(), "2".to_string()];
    let layout = compose_layout(&left, &right, 2, '~');
    // max left visual width = 3, gap = 2 → total left slot = 5
    // "abc" (3 wide) needs 5 - 3 = 2 padding spaces
    // "x"   (1 wide) needs 5 - 1 = 4 padding spaces
    assert_eq!(layout[0].0, "abc");
    assert_eq!(layout[0].1, 2);
    assert_eq!(layout[1].0, "x");
    assert_eq!(layout[1].1, 4);
}

#[test]
fn left_longer_right_padded_with_empty() {
    let left = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let right = vec!["1".to_string()];
    let layout = compose_layout(&left, &right, 1, '~');
    assert_eq!(layout.len(), 3);
    assert_eq!(layout[1].2, "");
    assert_eq!(layout[2].2, "");
}

#[test]
fn right_longer_left_padded_with_empty() {
    let left = vec!["a".to_string()];
    let right = vec!["1".to_string(), "2".to_string()];
    let layout = compose_layout(&left, &right, 1, '~');
    assert_eq!(layout.len(), 2);
    assert_eq!(layout[1].0, "");
}

#[test]
fn ansi_codes_not_counted_in_width() {
    // "\x1b[32mabc\x1b[0m" visually = 3 chars
    let left = vec!["\x1b[32mabc\x1b[0m".to_string()];
    let right = vec!["1".to_string()];
    let layout = compose_layout(&left, &right, 2, '~');
    // visual width 3, gap 2 → 2 padding spaces
    assert_eq!(layout[0].1, 2);
}

#[test]
fn marker_chars_not_counted_in_width() {
    // "~abc~" visual width = 3 (markers stripped)
    let left = vec!["~abc~".to_string()];
    let right = vec!["x".to_string()];
    let layout = compose_layout(&left, &right, 1, '~');
    assert_eq!(layout[0].1, 1);
}

use deob::integrations::animate_side_by_side;
use deob::animator::{AnimConfig, RevealOrder};
use deob::charset::ResolvedCharSet;
use deob::cli::AnsiColor;
use std::time::Duration;

fn zero_config() -> AnimConfig {
    AnimConfig {
        speed: Duration::from_millis(0),
        color: AnsiColor::Green,
        charset: ResolvedCharSet::Alnum,
        order: RevealOrder::Ordered,
        scrambles_min: 1,
        scrambles_max: 1,
    }
}

#[test]
fn animate_sbs_output_contains_real_text() {
    let left = vec!["logo".to_string()];
    let right = vec!["OS: ~Ubuntu~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_side_by_side(&left, &right, 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("logo"), "left column text missing");
    assert!(output.contains("OS: "), "static right text missing");
    assert!(output.contains("Ubuntu"), "scrambled right text missing in final state");
}

#[test]
fn animate_sbs_no_markers_prints_static() {
    let left = vec!["left".to_string()];
    let right = vec!["right".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_side_by_side(&left, &right, 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("left"));
    assert!(output.contains("right"));
}

#[test]
fn animate_sbs_both_columns_with_markers() {
    let left = vec!["~logo~".to_string()];
    let right = vec!["~info~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_side_by_side(&left, &right, 2, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains("logo"));
    assert!(output.contains("info"));
}

#[test]
fn animate_sbs_empty_inputs_does_not_panic() {
    let mut buf: Vec<u8> = Vec::new();
    animate_side_by_side(&[], &[], 2, '~', &zero_config(), &mut buf);
}

#[test]
fn animate_sbs_multiline() {
    let left = vec!["a".to_string(), "b".to_string()];
    let right = vec!["~x~".to_string(), "~y~".to_string()];
    let mut buf: Vec<u8> = Vec::new();
    animate_side_by_side(&left, &right, 1, '~', &zero_config(), &mut buf);
    let output = String::from_utf8_lossy(&buf);
    assert!(output.contains('x'));
    assert!(output.contains('y'));
}
