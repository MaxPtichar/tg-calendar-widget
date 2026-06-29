use chrono::NaiveDate;

use crate::{
    builder::{build_calendar, month_grid, year_grid},
    locale::types::Locale,
    models::{BackTo, CalendarAction, CalendarConfig, CalendarEvent},
};

/// Encoding/decoding of calendar callback data.
///
/// Callback strings follow the format `c:<command>:<date>`, where:
/// - `c:` — fixed prefix identifying calendar callbacks.
/// - `<command>` — one of `move`, `months_c`, `months_y`, `years`, `calendar`,
///   or `ignore`.
/// - `<date>` — the associated date, formatted as `dd.mm.yyyy`.
///
/// # Examples
///
/// ```text
/// c:move:23.03.2026
/// c:years:01.01.2026
/// c:ignore:
/// ```
impl CalendarEvent {
    /// Parses a callback string into a [`CalendarEvent`].
    ///
    /// Returns `None` if the string is not a recognized calendar callback
    /// (e.g. a custom callback defined by the caller) or if it's an
    /// `ignore` callback.
    pub fn from_callback(data: &str) -> Option<Self> {
        if data.contains("c:ignore:") {
            return None;
        }
        if !data.contains("c:") {
            return None;
        }

        let clean_callback = data.trim_start_matches("c:");
        let (command, date_str) = clean_callback.split_once(":")?;
        let date = NaiveDate::parse_from_str(date_str, "%d.%m.%Y").ok()?;

        match command {
            "move" => Some(Self::MoveMonth(date)),
            "months_c" => Some(Self::ShowMonths(date, BackTo::BackToCalendar)),
            "months_y" => Some(Self::ShowMonths(date, BackTo::BackToYears)),
            "years" => Some(Self::ShowYears(date)),
            "calendar" => Some(Self::ShowCalendar(date)),
            _ => None,
        }
    }

    /// Encodes this event back into its callback string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::NaiveDate;
    /// use calendar::models::CalendarEvent;
    ///
    /// let event = CalendarEvent::MoveMonth(NaiveDate::from_ymd_opt(2026, 3, 23).unwrap());
    /// assert_eq!(event.to_callback(), "c:move:23.03.2026");
    /// ```
    pub fn to_callback(&self) -> String {
        let fmt = "%d.%m.%Y";

        match &self {
            CalendarEvent::MoveMonth(date) => format!("c:move:{}", date.format(fmt)),
            CalendarEvent::ShowMonths(date, backto) => match backto {
                BackTo::BackToCalendar => format!("c:months_c:{}", date.format(fmt)),
                BackTo::BackToYears => format!("c:months_y:{}", date.format(fmt)),
            },
            CalendarEvent::ShowYears(date) => format!("c:years:{}", date.format(fmt)),
            CalendarEvent::ShowCalendar(date) => format!("c:calendar:{}", date.format(fmt)),
            CalendarEvent::Ignore => format!("c:ignore:"),
        }
    }
}

impl CalendarAction {
    /// Parses a raw callback string and produces the corresponding action.
    ///
    /// If the callback matches a known [`CalendarEvent`] (navigation: moving
    /// to a date, switching to month/year picker view, going back), returns
    /// [`CalendarAction::Redraw`] with the freshly rendered markup — always
    /// built using the provided `config`, so date constraints are preserved
    /// across navigation.
    ///
    /// If the callback does not match any known calendar event, it's treated
    /// as a custom (user-defined) callback and returned as
    /// [`CalendarAction::Custom`] with the raw string.
    ///
    /// Returns `None` if the callback is a recognized navigation event that
    /// should be ignored (e.g. tapping a disabled/out-of-range day).
    ///
    /// # Parameters
    ///
    /// - `callback`: the raw callback data string from Telegram.
    /// - `locale`: locale used to render month/weekday names.
    /// - `callback_data`: function that encodes a selected [`NaiveDate`] into
    ///   the callback data sent on a day button press.
    /// - `config`: date constraints applied consistently across all renders.
    ///
    /// # Examples
    ///
    /// ```
    /// use calendar::models::{CalendarAction, CalendarConfig};
    /// use calendar::locale::types::Locale;
    ///
    ///
    /// let config = CalendarConfig::default();
    /// let locale = Locale::default();
    ///
    /// let action = CalendarAction::handle_callback(
    ///     "some_unrecognized_callback",
    ///     &locale,
    ///     |date| date.to_string(),
    ///     &config,
    /// );
    ///
    /// assert!(matches!(action, Some(CalendarAction::Custom(_))));
    /// ```
    pub fn handle_callback<T, F>(
        callback: &str,
        locale: &Locale,
        callback_data: F,
        config: &CalendarConfig,
    ) -> Option<CalendarAction>
    where
        F: Fn(NaiveDate) -> T + Copy,
        T: ToString,
    {
        if let Some(event) = CalendarEvent::from_callback(callback) {
            match event {
                CalendarEvent::MoveMonth(date) => {
                    return Some(CalendarAction::Redraw(build_calendar(
                        date,
                        locale,
                        callback_data,
                        config,
                    )));
                }

                CalendarEvent::ShowMonths(date, back) => {
                    let back_event = match back {
                        BackTo::BackToCalendar => CalendarEvent::ShowCalendar(date),
                        BackTo::BackToYears => CalendarEvent::ShowYears(date),
                    };

                    return Some(CalendarAction::Redraw(month_grid(
                        &date, locale, config, back_event,
                    )));
                }
                CalendarEvent::ShowYears(date) => {
                    return Some(CalendarAction::Redraw(year_grid(&date, config)));
                }

                CalendarEvent::ShowCalendar(date) => {
                    return Some(CalendarAction::Redraw(build_calendar(
                        date,
                        locale,
                        callback_data,
                        config,
                    )));
                }
                CalendarEvent::Ignore => {
                    return None;
                }
            }
        } else {
            return Some(CalendarAction::Custom(callback.to_string()));
        }
    }
}

/// Checks whether a callback string belongs to the calendar's namespace.
///
/// Intended for use as a filter predicate in `dptree` routing, so that
/// all calendar-related callbacks — including `ignore` callbacks from
/// disabled buttons — are routed to the calendar handler, while other
/// (unrelated) callbacks fall through to other handlers.
pub fn is_calendar_callback(data: &str) -> bool {
    data.starts_with("c:")
}

#[cfg(test)]
mod is_calendar_callback_tests {
    use super::*;

    #[test]
    fn recognizes_calendar_prefixed_callbacks() {
        assert!(is_calendar_callback("c:move:23.03.2026"));
        assert!(is_calendar_callback("c:years:01.01.2026"));
        assert!(is_calendar_callback("c:calendar:01.01.2026"));
    }

    #[test]
    fn recognizes_ignore_as_calendar_callback() {
        // Important: ignore callbacks must still be routed into the
        // calendar handler (so they're silently no-op'd there),
        // rather than falling through to unrelated bot handlers.
        assert!(is_calendar_callback("c:ignore: "));
    }

    #[test]
    fn rejects_non_calendar_callbacks() {
        assert!(!is_calendar_callback("some_custom_callback"));
        assert!(!is_calendar_callback("calendar")); // no "c:" prefix
    }
}

#[cfg(test)]
mod callback_tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn parses_move_month() {
        let event = CalendarEvent::from_callback("c:move:23.03.2026");
        let expected_date = NaiveDate::from_ymd_opt(2026, 3, 23).unwrap();

        assert_eq!(event, Some(CalendarEvent::MoveMonth(expected_date)));
    }

    #[test]
    fn parses_show_months_back_to_calendar() {
        let event = CalendarEvent::from_callback("c:months_c:01.01.2026");
        let expected_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        assert_eq!(
            event,
            Some(CalendarEvent::ShowMonths(
                expected_date,
                BackTo::BackToCalendar
            ))
        );
    }

    #[test]
    fn parses_show_months_back_to_years() {
        let event = CalendarEvent::from_callback("c:months_y:01.01.2026");
        let expected_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        assert_eq!(
            event,
            Some(CalendarEvent::ShowMonths(
                expected_date,
                BackTo::BackToYears
            ))
        );
    }

    #[test]
    fn parses_show_years() {
        let event = CalendarEvent::from_callback("c:years:01.01.2026");
        let expected_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        assert_eq!(event, Some(CalendarEvent::ShowYears(expected_date)));
    }

    #[test]
    fn parses_show_calendar() {
        let event = CalendarEvent::from_callback("c:calendar:01.01.2026");
        let expected_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();

        assert_eq!(event, Some(CalendarEvent::ShowCalendar(expected_date)));
    }

    #[test]
    fn ignore_callback_returns_none() {
        assert_eq!(CalendarEvent::from_callback("c:ignore: "), None);
    }

    #[test]
    fn non_calendar_callback_returns_none() {
        assert_eq!(CalendarEvent::from_callback("some_custom_callback"), None);
    }

    #[test]
    fn unknown_command_returns_none() {
        assert_eq!(
            CalendarEvent::from_callback("c:bogus_command:01.01.2026"),
            None
        );
    }

    #[test]
    fn malformed_date_does_not_panic() {
        // Regression test: this used to panic via .unwrap() on parse_from_str.
        assert_eq!(CalendarEvent::from_callback("c:move:not_a_date"), None);
    }

    #[test]
    fn missing_date_does_not_panic() {
        // No ":" separator after the command at all.
        assert_eq!(CalendarEvent::from_callback("c:move"), None);
    }

    #[test]
    fn to_callback_move_month() {
        let date = NaiveDate::from_ymd_opt(2026, 3, 23).unwrap();
        let event = CalendarEvent::MoveMonth(date);

        assert_eq!(event.to_callback(), "c:move:23.03.2026");
    }

    #[test]
    fn to_callback_show_months_back_to_calendar() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let event = CalendarEvent::ShowMonths(date, BackTo::BackToCalendar);

        assert_eq!(event.to_callback(), "c:months_c:01.01.2026");
    }

    #[test]
    fn to_callback_show_months_back_to_years() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let event = CalendarEvent::ShowMonths(date, BackTo::BackToYears);

        assert_eq!(event.to_callback(), "c:months_y:01.01.2026");
    }

    #[test]
    fn to_callback_show_years() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let event = CalendarEvent::ShowYears(date);

        assert_eq!(event.to_callback(), "c:years:01.01.2026");
    }

    #[test]
    fn to_callback_show_calendar() {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let event = CalendarEvent::ShowCalendar(date);

        assert_eq!(event.to_callback(), "c:calendar:01.01.2026");
    }

    #[test]
    fn to_callback_ignore() {
        assert_eq!(CalendarEvent::Ignore.to_callback(), "c:ignore:");
    }

    #[test]
    fn roundtrip_move_month() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let event = CalendarEvent::MoveMonth(date);

        let encoded = event.to_callback();
        let decoded = CalendarEvent::from_callback(&encoded);

        assert_eq!(decoded, Some(event));
    }
}
