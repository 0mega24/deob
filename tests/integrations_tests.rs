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
