use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "deob", version, about = "De-obfuscate text with a hacker-style animation")]
pub struct Args {
    /// Text to animate (reads from stdin if omitted)
    pub text: Option<String>,

    /// Delay between frames in milliseconds
    #[arg(short = 's', long, default_value_t = 50)]
    pub speed: u64,

    /// ANSI foreground color
    #[arg(short = 'C', long, default_value_t = AnsiColor::Green)]
    pub color: AnsiColor,

    /// Noise character set
    #[arg(short = 'c', long, default_value_t = CharSet::Auto)]
    pub charset: CharSet,

    /// Character reveal order
    #[arg(short = 'o', long, default_value_t = RevealOrder::Ordered)]
    pub order: RevealOrder,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl std::fmt::Display for AnsiColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value().unwrap().get_name().fmt(f)
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum CharSet {
    /// All printable ASCII (! through ~)
    Ascii,
    /// Alphanumeric only (A-Z, a-z, 0-9)
    Alnum,
    /// Hex digits + @#$%^&*!?
    Hacker,
    /// Auto-detect from input content
    Auto,
}

impl std::fmt::Display for CharSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value().unwrap().get_name().fmt(f)
    }
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum RevealOrder {
    /// Lock characters left-to-right
    Ordered,
    /// Lock characters in a random sequence
    Random,
}

impl std::fmt::Display for RevealOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value().unwrap().get_name().fmt(f)
    }
}
