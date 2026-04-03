use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "deob",
    version,
    about = "De-obfuscate text with a hacker-style animation"
)]
pub struct Args {
    /// Text to animate (reads from stdin if omitted)
    pub text: Option<String>,

    /// Delay between frames in milliseconds
    #[arg(short = 's', long, default_value_t = 50)]
    pub speed: u64,

    /// ANSI foreground color for noise characters
    #[arg(short = 'C', long, default_value_t = AnsiColor::Green)]
    pub color: AnsiColor,

    /// Noise character set
    #[arg(short = 'c', long, default_value_t = CharSet::Auto)]
    pub charset: CharSet,

    /// Character reveal order
    #[arg(short = 'o', long, default_value_t = RevealOrder::Ordered)]
    pub order: RevealOrder,

    /// Minimum scramble frames per character before it locks
    #[arg(short = 'm', long, default_value_t = 3)]
    pub scrambles_min: u32,

    /// Maximum scramble frames per character before it locks
    #[arg(short = 'x', long, default_value_t = 10)]
    pub scrambles_max: u32,

    /// Column files for side-by-side mode (repeat for each column, minimum 2)
    #[arg(long = "col", value_name = "FILE")]
    pub cols: Vec<std::path::PathBuf>,

    /// Extra spaces between columns
    #[arg(long, default_value_t = 2)]
    pub gap: usize,

    /// Character delimiting scramble regions in column mode
    #[arg(long, default_value_t = '~')]
    pub marker: char,

    /// Vertical alignment of shorter columns relative to the tallest
    #[arg(long, value_enum, default_value_t = VAlign::Top)]
    pub valign: VAlign,
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

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum VAlign {
    /// Align all columns to the top (default)
    Top,
    /// Center shorter columns vertically against the tallest
    Center,
}

impl std::fmt::Display for VAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value().unwrap().get_name().fmt(f)
    }
}
