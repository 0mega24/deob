use crate::cli::CharSet;
use rand::Rng;

const HACKER_CHARS: &[u8] = b"0123456789ABCDEF@#$%^&*!?";
const ALNUM_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ResolvedCharSet {
    Ascii,
    Alnum,
    Hacker,
}

pub fn resolve(charset: CharSet, input: &str) -> ResolvedCharSet {
    match charset {
        CharSet::Ascii => ResolvedCharSet::Ascii,
        CharSet::Alnum => ResolvedCharSet::Alnum,
        CharSet::Hacker => ResolvedCharSet::Hacker,
        CharSet::Auto => {
            if input.is_empty() {
                return ResolvedCharSet::Hacker;
            }
            let has_symbol = input
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace());
            if has_symbol {
                ResolvedCharSet::Ascii
            } else if input
                .chars()
                .all(|c| c.is_alphanumeric() || c.is_whitespace())
            {
                ResolvedCharSet::Alnum
            } else {
                ResolvedCharSet::Hacker
            }
        }
    }
}

pub fn random_char(set: ResolvedCharSet, rng: &mut impl Rng) -> char {
    match set {
        ResolvedCharSet::Hacker => {
            let idx = rng.gen_range(0..HACKER_CHARS.len());
            HACKER_CHARS[idx] as char
        }
        ResolvedCharSet::Alnum => {
            let idx = rng.gen_range(0..ALNUM_CHARS.len());
            ALNUM_CHARS[idx] as char
        }
        ResolvedCharSet::Ascii => rng.gen_range('!'..='~'),
    }
}
