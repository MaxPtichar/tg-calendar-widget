use chrono::Weekday;

/// Locale-specific strings and settings for rendering the calendar.
#[derive(Debug, Clone, Copy)]
pub struct Locale {
    /// Weekday abbreviations, Monday-first (`week_days[0]` is always Monday).
    pub week_days: [&'static str; 7],
    /// Full month names, January first (`months[0]` is always January).
    pub months: [&'static str; 12],
    /// Reserved for future use — currently has no effect on rendering.
    /// The day grid is always Monday-first regardless of this value.
    pub first_weekday: Weekday,
}