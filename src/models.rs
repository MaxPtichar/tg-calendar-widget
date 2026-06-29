use chrono::{Datelike, NaiveDate};

/// A single button with display text and an associated callback payload.
///
/// Used to build calendar UI rows (e.g. in a Telegram inline keyboard
/// via an adapter).
#[derive(Default, Clone)]
pub struct CalendarButton {
    pub text: String,
    pub callback_data: String,
}
impl CalendarButton {
    /// Creates a new button from any types convertible into `String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tg_calendar_widget::models::CalendarButton;
    ///
    /// let button = CalendarButton::new("15", "move_to:2026-06-15");
    /// assert_eq!(button.text, "15");
    /// ```
    pub fn new(text: impl Into<String>, callback_data: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            callback_data: callback_data.into(),
        }
    }
}

/// A grid of buttons representing the full rendered calendar UI
/// (e.g. one row per week, plus navigation rows).
pub struct CalendarMarkup {
    pub rows: Vec<Vec<CalendarButton>>,
}

impl CalendarMarkup {
    /// Wraps a grid of buttons into a `CalendarMarkup`.
    pub fn new(button: Vec<Vec<CalendarButton>>) -> Self {
        Self { rows: button }
    }
}

/// Parsed navigation/render events encoded in calendar callback data.
///
/// - [`CalendarEvent::MoveMonth`] - navigate to the previous/next month
///   and redraw the main day grid.
/// - [`CalendarEvent::ShowMonths`] - switch to the month-picker grid.
/// - [`CalendarEvent::ShowYears`] - switch to the year-picker grid;
///   also used to page forward/backward through year ranges (±9 years).
/// - [`CalendarEvent::ShowCalendar`] - switch back to the main day grid.
/// - [`CalendarEvent::Ignore`] - callback should produce no UI change
///   (e.g. a disabled boundary button at the edge of `min_date`/`max_date`).
#[derive(Debug, PartialEq)]
pub enum CalendarEvent {
    MoveMonth(NaiveDate),
    ShowMonths(NaiveDate, BackTo),
    ShowYears(NaiveDate),
    ShowCalendar(NaiveDate),
    Ignore,
}

/// Indicates where [`CalendarEvent::ShowMonths`] should navigate back to
/// when the user presses "back" from the month picker.
///
/// - [`BackTo::BackToCalendar`] - return to the main day grid.
/// - [`BackTo::BackToYears`] - return to the year picker.
#[derive(Debug, PartialEq)]
pub enum BackTo {
    BackToCalendar,
    BackToYears,
}

/// Represents the outcome of handling a calendar callback.
///
/// - [`CalendarAction::Redraw`] - the calendar UI needs to be re-rendered
///   (e.g. after navigating to a different month or year). Carries the
///   new [`CalendarMarkup`] to display.
/// - [`CalendarAction::Custom`] - a user-defined callback was triggered
///   (e.g. selecting a specific date). Carries the raw callback data string
///   for the caller to interpret.
///
/// # Examples
///
/// ```
/// use tg_calendar_widget::models::CalendarAction;
///
/// fn handle(action: CalendarAction) {
///     match action {
///         CalendarAction::Redraw(markup) => {
///             // re-send the calendar with updated markup
///         }
///         CalendarAction::Custom(data) => {
///             // parse `data` as the selected date
///         }
///     }
/// }
/// ```
pub enum CalendarAction {
    /// Calendar should be redrawn with the given markup (navigation occurred).
    Redraw(CalendarMarkup),
    /// A custom, non-navigation callback was triggered; carries the raw callback data.
    Custom(String),
}

/// Configuration for constraining the selectable date range in the calendar.
///
/// Both fields are optional - leaving a bound as `None` means that side
/// is unconstrained. Use [`CalendarConfig::default()`] for no constraints at all.
///
/// # Examples
///
/// Restrict selection to dates from 2026 onward:
///
/// ```
/// use chrono::NaiveDate;
/// use tg_calendar_widget::models::CalendarConfig;
///
/// let config = CalendarConfig {
///     min_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
///     max_date: None,
/// };
///
/// assert!(config.is_date_allowed(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
/// ```
#[derive(Default)]
pub struct CalendarConfig {
    pub min_date: Option<NaiveDate>,
    pub max_date: Option<NaiveDate>,
}

impl CalendarConfig {
    /// Checks if the given date falls within the configured `min_date`/`max_date` range.
    ///
    /// If a bound is `None`, that side is treated as unconstrained.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use tg_calendar_widget::models::CalendarConfig;
    ///
    /// let config = CalendarConfig {
    ///     min_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
    ///     max_date: None,
    /// };
    ///
    /// assert!(config.is_date_allowed(NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()));
    /// assert!(!config.is_date_allowed(NaiveDate::from_ymd_opt(2025, 12, 31).unwrap()));
    /// ```
    pub fn is_date_allowed(&self, date: NaiveDate) -> bool {
        match (self.min_date, self.max_date) {
            (Some(min_d), Some(max_d)) => date >= min_d && date <= max_d,
            (Some(min_d), Option::None) => date >= min_d,
            (Option::None, Some(max_d)) => date <= max_d,
            (Option::None, Option::None) => true,
        }
    }

    /// Checks if the given year falls within the configured `min_date`/`max_date` range.
    ///
    /// If a bound is `None`, that side is treated as unconstrained.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use tg_calendar_widget::models::CalendarConfig;
    ///
    /// let config = CalendarConfig {
    ///     min_date: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
    ///     max_date: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
    /// };
    ///
    /// assert!(config.is_year_allowed(2025));
    /// assert!(!config.is_year_allowed(2027));
    /// ```
    pub fn is_year_allowed(&self, year: i32) -> bool {
        match (self.min_date, self.max_date) {
            (Some(min_d), Some(max_d)) => year >= min_d.year() && year <= max_d.year(),
            (Some(min_d), Option::None) => year >= min_d.year(),
            (Option::None, Some(max_d)) => year <= max_d.year(),
            (Option::None, Option::None) => true,
        }
    }

    /// Returns the month to display when navigating to `year`.
    ///
    /// If `year` matches `min_date`'s year, returns that month.
    /// If `year` matches `max_date`'s year, returns that month.
    /// **Note:** if `min_date` and `max_date` share the same year,
    /// `min_date`'s month takes precedence.
    /// Otherwise defaults to `1` (January).
    pub fn get_month_for_year(&self, year: i32) -> u32 {
        match (self.min_date, self.max_date) {
            (Some(min_d), Some(max_d)) => {
                if year == min_d.year() {
                    min_d.month()
                } else if year == max_d.year() {
                    max_d.month()
                } else {
                    1
                }
            }
            (Some(min_d), None) => {
                if year == min_d.year() {
                    min_d.month()
                } else {
                    1
                }
            }
            (None, Some(max_d)) => {
                if year == max_d.year() {
                    max_d.month()
                } else {
                    1
                }
            }
            (None, None) => 1,
        }
    }
}

#[cfg(test)]
mod is_date_allowed_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn min_max_date_allowed() {
        let min_date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        let date_allowed = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let date_allowed_top_edge = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let date_allowed_bottom_edge = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date_not_allowed = NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();

        assert_eq!(config.is_date_allowed(date_allowed), true);
        assert_eq!(config.is_date_allowed(date_allowed_top_edge), true);
        assert_eq!(config.is_date_allowed(date_allowed_bottom_edge), true);
        assert_eq!(config.is_date_allowed(date_not_allowed), false);
    }

    #[test]
    fn max_date_allowed() {
        let max_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let config = CalendarConfig {
            min_date: None,
            max_date: Some(max_date),
        };

        let date_allowed = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let date_allowed_top_edge = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let date_allowed_bottom_edge = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let date_not_allowed = NaiveDate::from_ymd_opt(2026, 6, 21).unwrap();
        let past_year = NaiveDate::from_ymd_opt(1999, 6, 21).unwrap();

        assert_eq!(config.is_date_allowed(date_allowed), true);
        assert_eq!(config.is_date_allowed(date_allowed_top_edge), true);
        assert_eq!(config.is_date_allowed(date_allowed_bottom_edge), true);
        assert_eq!(config.is_date_allowed(past_year), true);
        assert_eq!(config.is_date_allowed(date_not_allowed), false);
    }

    #[test]
    fn min_date_allowed() {
        let min_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: None,
        };

        let date_allowed = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let date_allowed_top_edge = NaiveDate::from_ymd_opt(3023, 6, 20).unwrap();
        let date_allowed_bottom_edge = NaiveDate::from_ymd_opt(1890, 6, 10).unwrap();
        let date_not_allowed = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();

        assert_eq!(config.is_date_allowed(date_allowed), true);
        assert_eq!(config.is_date_allowed(date_allowed_top_edge), true);
        assert_eq!(config.is_date_allowed(date_allowed_bottom_edge), false);
        assert_eq!(config.is_date_allowed(date_not_allowed), false);
    }

    #[test]
    fn no_min_max() {
        let config = CalendarConfig {
            min_date: None,
            max_date: None,
        };

        let date_allowed = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let date_allowed_top_edge = NaiveDate::from_ymd_opt(3023, 6, 20).unwrap();
        let date_allowed_bottom_edge = NaiveDate::from_ymd_opt(1890, 6, 10).unwrap();
        let date_not_allowed = NaiveDate::from_ymd_opt(2026, 6, 19).unwrap();

        assert_eq!(config.is_date_allowed(date_allowed), true);
        assert_eq!(config.is_date_allowed(date_allowed_top_edge), true);
        assert_eq!(config.is_date_allowed(date_allowed_bottom_edge), true);
        assert_eq!(config.is_date_allowed(date_not_allowed), true);
    }
}
#[cfg(test)]
mod is_year_allowed_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn min_max_year_allowed() {
        let min_date = NaiveDate::from_ymd_opt(2024, 6, 10).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        assert_eq!(config.is_year_allowed(2025), true);
        assert_eq!(config.is_year_allowed(2024), true);
        assert_eq!(config.is_year_allowed(2026), true);
        assert_eq!(config.is_year_allowed(2023), false);
        assert_eq!(config.is_year_allowed(2027), false);
    }

    #[test]
    fn max_year_allowed() {
        let max_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let config = CalendarConfig {
            min_date: None,
            max_date: Some(max_date),
        };

        assert_eq!(config.is_year_allowed(2026), true);
        assert_eq!(config.is_year_allowed(1890), true);
        assert_eq!(config.is_year_allowed(2027), false);
    }

    #[test]
    fn min_year_allowed() {
        let min_date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: None,
        };

        assert_eq!(config.is_year_allowed(2026), true);
        assert_eq!(config.is_year_allowed(3023), true);
        assert_eq!(config.is_year_allowed(2025), false);
    }

    #[test]
    fn no_min_max_year() {
        let config = CalendarConfig {
            min_date: None,
            max_date: None,
        };

        assert_eq!(config.is_year_allowed(2026), true);
        assert_eq!(config.is_year_allowed(1890), true);
        assert_eq!(config.is_year_allowed(3023), true);
    }
}

#[cfg(test)]
mod get_month_for_year_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn min_max_year_matches_min() {
        let min_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        assert_eq!(config.get_month_for_year(2024), 3); // совпал с min
    }

    #[test]
    fn min_max_year_matches_max() {
        let min_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        assert_eq!(config.get_month_for_year(2026), 9); // совпал с max
    }

    #[test]
    fn min_max_year_in_between_defaults_to_january() {
        let min_date = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        assert_eq!(config.get_month_for_year(2025), 1); // ни min, ни max - январь
    }

    #[test]
    fn min_equals_max_year_resolves_to_min_month() {
        // Зафиксированное поведение: если min_d.year() == max_d.year(),
        // ветка `year == min_d.year()` стоит первой и побеждает,
        // независимо от того, что год тот же самый что и у max_d.
        let min_date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 9, 1).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        assert_eq!(config.get_month_for_year(2026), 3);
    }

    #[test]
    fn only_min_date_matches() {
        let min_date = NaiveDate::from_ymd_opt(2024, 5, 1).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: None,
        };

        assert_eq!(config.get_month_for_year(2024), 5);
        assert_eq!(config.get_month_for_year(2030), 1);
    }

    #[test]
    fn only_max_date_matches() {
        let max_date = NaiveDate::from_ymd_opt(2026, 11, 1).unwrap();
        let config = CalendarConfig {
            min_date: None,
            max_date: Some(max_date),
        };

        assert_eq!(config.get_month_for_year(2026), 11);
        assert_eq!(config.get_month_for_year(2020), 1);
    }

    #[test]
    fn no_min_max_defaults_to_january() {
        let config = CalendarConfig {
            min_date: None,
            max_date: None,
        };

        assert_eq!(config.get_month_for_year(2026), 1);
    }
}
