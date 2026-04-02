use deob::integrations::{parse_markers, Segment};

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
