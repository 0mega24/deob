//! Integration extension point.
//!
//! Future integrations (fastfetch, starship, etc.) live here.
//! The `animator::animate` function accepts any `std::io::Write` impl,
//! so integrations can provide their own output targets.
