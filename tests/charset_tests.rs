use deob::charset::{random_char, resolve, ResolvedCharSet};
use deob::cli::CharSet;
use rand::rngs::SmallRng;
use rand::SeedableRng;

#[test]
fn auto_all_alnum_resolves_to_alnum() {
    assert_eq!(resolve(CharSet::Auto, "Hello123"), ResolvedCharSet::Alnum);
}

#[test]
fn auto_with_symbol_resolves_to_ascii() {
    assert_eq!(resolve(CharSet::Auto, "Hello!"), ResolvedCharSet::Ascii);
}

#[test]
fn auto_empty_resolves_to_hacker() {
    assert_eq!(resolve(CharSet::Auto, ""), ResolvedCharSet::Hacker);
}

#[test]
fn explicit_hacker_stays_hacker() {
    assert_eq!(
        resolve(CharSet::Hacker, "anything"),
        ResolvedCharSet::Hacker
    );
}

#[test]
fn random_char_alnum_is_alphanumeric() {
    let mut rng = SmallRng::seed_from_u64(42);
    for _ in 0..100 {
        let c = random_char(ResolvedCharSet::Alnum, &mut rng);
        assert!(c.is_alphanumeric(), "expected alphanumeric, got {:?}", c);
    }
}

#[test]
fn random_char_hacker_in_set() {
    const HACKER: &str = "0123456789ABCDEF@#$%^&*!?";
    let mut rng = SmallRng::seed_from_u64(0);
    for _ in 0..100 {
        let c = random_char(ResolvedCharSet::Hacker, &mut rng);
        assert!(HACKER.contains(c), "expected hacker char, got {:?}", c);
    }
}

#[test]
fn random_char_ascii_is_printable() {
    let mut rng = SmallRng::seed_from_u64(7);
    for _ in 0..100 {
        let c = random_char(ResolvedCharSet::Ascii, &mut rng);
        assert!(
            ('!'..='~').contains(&c),
            "expected printable ASCII, got {:?}",
            c
        );
    }
}
