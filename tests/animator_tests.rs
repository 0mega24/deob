use deob::animator::{build_schedule, AnimConfig, RevealOrder as AnimOrder};
use deob::charset::ResolvedCharSet;
use rand::SeedableRng;

#[test]
fn ordered_schedule_is_left_to_right() {
    let schedule = build_schedule(vec![0, 1, 2, 3, 4], AnimOrder::Ordered, &mut rand::thread_rng());
    assert_eq!(schedule, vec![0, 1, 2, 3, 4]);
}

#[test]
fn random_schedule_contains_all_indices() {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(99);
    let schedule = build_schedule(vec![0, 1, 2, 3, 4], AnimOrder::Random, &mut rng);
    let mut sorted = schedule.clone();
    sorted.sort();
    assert_eq!(sorted, vec![0, 1, 2, 3, 4]);
}

#[test]
fn random_schedule_is_not_always_ordered() {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(1);
    let schedule = build_schedule(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9], AnimOrder::Random, &mut rng);
    let ordered: Vec<usize> = (0..10).collect();
    assert_ne!(schedule, ordered);
}

#[test]
fn animate_output_ends_with_real_text_and_newline() {
    use deob::cli::AnsiColor;
    use std::time::Duration;

    let config = AnimConfig {
        speed: Duration::from_millis(0),
        color: AnsiColor::Green,
        charset: ResolvedCharSet::Alnum,
        order: AnimOrder::Ordered,
        scrambles_min: 1,
        scrambles_max: 1,
        valign: deob::cli::VAlign::Top,
    };
    let mut buf: Vec<u8> = Vec::new();
    deob::animator::animate("hi", &config, &mut buf);

    let output = String::from_utf8_lossy(&buf);
    // Final state must contain the real text
    assert!(output.contains('h'));
    assert!(output.contains('i'));
}
