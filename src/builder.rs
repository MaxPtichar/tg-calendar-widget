use chrono::{Datelike, Months, NaiveDate};

use crate::{
    calendar::calendar_button,
    locale::types::Locale,
    models::{CalendarButton, CalendarConfig, CalendarEvent, CalendarMarkup},
};
/// Builds the main calendar day-grid markup for the month containing
/// `current_date`.
///
/// This is the primary entry point for rendering the calendar: it filters
/// the month's days against `config`'s constraints and delegates to
/// [`calendar_button`] for layout.
///
/// # Parameters
///
/// - `current_date`: any date within the month to display. Only its
///   year and month are used; the day is ignored.
/// - `locales`: locale used for month/weekday names.
/// - `callback`: encodes a selected [`NaiveDate`] into the callback data
///   sent when a day button is pressed.
/// - `config`: date constraints applied to every day in the grid (see
///   [`CalendarConfig::is_date_allowed`]). Disallowed days render as
///   blank, ignored buttons rather than being removed from the grid.
///
/// If `config` excludes every day of `current_date`'s month entirely
/// (e.g. `min_date` falls after this month, or `max_date` falls before
/// it), returns a single-row markup reading "No dates available" instead
/// of panicking or returning a malformed grid.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use calendar::builder::build_calendar;
/// use calendar::models::CalendarConfig;
/// use calendar::locale::types::Locale;
///
/// let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
/// let config = CalendarConfig::default();
/// let locale = Locale::default();
///
/// let markup = build_calendar(date, &locale, |d| d.format("%d.%m.%Y"), &config);
/// assert!(!markup.rows.is_empty());
/// ```
///
/// Fully-excluded month:
///
/// ```
/// use chrono::NaiveDate;
/// use calendar::builder::build_calendar;
/// use calendar::models::CalendarConfig;
/// use calendar::locale::types::Locale;
///
/// let current_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
/// let min_date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
/// let config = CalendarConfig { min_date: Some(min_date), max_date: None };
/// let locale = Locale::default();
///
/// let markup = build_calendar(current_date, &locale, |d| d.to_string(), &config);
///
/// assert_eq!(markup.rows.len(), 1);
/// assert_eq!(markup.rows[0][0].text, "No dates available");
/// ```
pub fn build_calendar<T, F>(
    current_date: NaiveDate,
    locales: &Locale,
    callback: F,
    config: &CalendarConfig,
) -> CalendarMarkup
where
    F: Fn(NaiveDate) -> T,
    T: ToString,
{
    let dates = allowed_dates_for_month(current_date, config);

    calendar_button(dates, locales, callback, config)
}

fn allowed_dates_for_month(current_date: NaiveDate, config: &CalendarConfig) -> Vec<NaiveDate> {
    let month_first_date = current_date.with_day(1).unwrap();

    let month = Months::new(1);

    let month_end_date = month_first_date.checked_add_months(month).unwrap();

    month_first_date
        .iter_days()
        .take_while(|date| *date < month_end_date)
        .filter(|date| config.is_date_allowed(*date))
        .collect::<Vec<NaiveDate>>()
}


/// Lays out the 6×7 day grid for a given month into fixed array positions,
/// based on which weekday the month starts on (Monday-first).
///
/// Returns `None` for the leading blank cells before day 1 and any
/// trailing cells after the last day. Panics if `dates` is empty.
pub(crate) fn calendar_grid(dates: &[NaiveDate]) -> Option<[Option<NaiveDate>; 42]> {
    let first_date = dates.first()?;
    let day_offset: usize = first_date.weekday().number_from_monday() as usize - 1;

    let mut grid: [Option<NaiveDate>; 42] = [Option::None; 42];

    for i in 0..dates.len() {
        grid[i + day_offset] = Some(dates[i]);
    }

    Some(grid)
}


/// Lays out the previous/back/next navigation row beneath the year grid.
///
/// # Layout
///
/// ```text
/// +------+------+------+
/// |  <<  |  ..  |  >>  |
/// +------+------+------+
/// ```
///
/// `<<`/`>>` page the entire 9-year window backward/forward by 9 years.
/// If `config.min_date`/`config.max_date`'s year falls within or before
/// the **current** displayed year, that direction's button is rendered
/// blank ([`CalendarEvent::Ignore`]) instead of paging further.
/// `..` always returns to the main day grid ([`CalendarEvent::ShowCalendar`]).
fn years_switcher(
    current_date: NaiveDate,
    prev_year: NaiveDate,
    next_year: NaiveDate,
    config: &CalendarConfig,
) -> Vec<CalendarButton> {
    let back_to_month = CalendarButton::new(
        "..",
        CalendarEvent::ShowCalendar(current_date).to_callback(),
    );

    let current_year = current_date.year();
    let past_years_button = if let Some(min_date) = config.min_date {
        if min_date.year() < current_year {
            CalendarButton::new("<<", CalendarEvent::ShowYears(prev_year).to_callback())
        } else {
            CalendarButton::new(" ", CalendarEvent::Ignore.to_callback())
        }
    } else {
        CalendarButton::new("<<", CalendarEvent::ShowYears(prev_year).to_callback())
    };

    let future_years_button = if let Some(max_date) = config.max_date {
        if max_date.year() > current_year {
            CalendarButton::new(">>", CalendarEvent::ShowYears(next_year).to_callback())
        } else {
            CalendarButton::new(" ", CalendarEvent::Ignore.to_callback())
        }
    } else {
        CalendarButton::new(">>", CalendarEvent::ShowYears(next_year).to_callback())
    };

    vec![past_years_button, back_to_month, future_years_button]
}


/// Builds the year picker grid: a 3×3 grid of 9 consecutive years
/// centered on `date`'s year, plus a navigation row.
///
/// # Layout
///
/// ```text
/// +------+------+------+
/// | 2013 | 2014 | 2015 |
/// | 2016 | 2017 | 2018 |
/// | 2019 | 2020 | 2021 |
/// +------+------+------+
/// |  <<  |  ..  |  >>  |
/// +------+------+------+
/// ```
///
/// Years disallowed by `config` (see [`CalendarConfig::is_year_allowed`])
/// render as blank, ignored buttons. Selecting a year jumps to that
/// year's first allowed month (see [`CalendarConfig::get_month_for_year`])
/// and opens the month picker ([`CalendarEvent::ShowMonths`]).
pub(crate) fn year_grid(date: &NaiveDate, config: &CalendarConfig) -> CalendarMarkup {
    let current_year = date.year();

    let start_date = current_year - 4;
    let end_date = current_year + 4;

    let prev_years = current_year - 9;
    let next_years: i32 = current_year + 9;

    let prev_date_years = NaiveDate::from_ymd_opt(prev_years, 1, 1).unwrap();
    let next_date_years = NaiveDate::from_ymd_opt(next_years, 1, 1).unwrap();

    let mut grid: Vec<Vec<CalendarButton>> = Vec::new();

    let switcher = years_switcher(*date, prev_date_years, next_date_years, config);

    let year_butt = (start_date..=end_date)
        .into_iter()
        .filter_map(|year| {
            if config.is_year_allowed(year) {
                let month = config.get_month_for_year(year);

                let created_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
                let formatted_year = format!("  {year}  ");
                Some(CalendarButton::new(
                    formatted_year,
                    CalendarEvent::ShowMonths(created_date, crate::models::BackTo::BackToYears )
                        .to_callback(),
                ))
            } else {
                Some(CalendarButton::new(
                    " ",
                    CalendarEvent::Ignore.to_callback(),
                ))
            }
        })
        .collect::<Vec<CalendarButton>>()
        .chunks(3)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<CalendarButton>>>();

    grid.extend(year_butt);
    grid.push(switcher);

    CalendarMarkup::new(grid)
}


/// Builds the month picker grid for `date`'s year: a 3×4 grid of month
/// names, plus a single "back" button.
///
/// # Layout
///
/// ```text
/// +----------+-----------+----------+
/// | January  | February  |  March   |
/// |  April   |    May    |   June   |
/// |  July    |  August   | September|
/// | October  | November  | December |
/// +----------+-----------+----------+
/// |                <<                |
/// +-----------------------------------+
/// ```
///
/// Months disallowed by `config` (see [`CalendarConfig::is_date_allowed`],
/// checked against the 1st of that month) render as blank, ignored buttons.
///
/// `event` determines where the back button (`<<`) returns to: it should
/// be either [`CalendarEvent::ShowCalendar`] or [`CalendarEvent::ShowYears`],
/// matching how the user navigated here (see [`BackTo`]). Any other event
/// variant is a programming error and will panic.
pub(crate) fn month_grid(
    date: &NaiveDate,
    locales: &Locale,
    config: &CalendarConfig,
    event: CalendarEvent,
) -> CalendarMarkup {
    let year = date.year();

    let month_names = locales.months;

    let mut month_button = month_names
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let month = i as u32 + 1;
            let created_date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

            let is_allowed = config.is_date_allowed(created_date);

            let (text_month, callback) = if is_allowed {
                (
                    m.to_string(),
                    CalendarEvent::ShowCalendar(created_date).to_callback(),
                )
            } else {
                (" ".to_string(), CalendarEvent::Ignore.to_callback())
            };

            CalendarButton::new(text_month, callback)
        })
        .collect::<Vec<CalendarButton>>()
        .chunks(3)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<Vec<CalendarButton>>>();

    dbg!(&event);

    let bact_to = match event {
        CalendarEvent::ShowCalendar(date) => vec![CalendarButton::new(
            "<<",
            CalendarEvent::ShowCalendar(date).to_callback(),
        )],

        CalendarEvent::ShowYears(date) => vec![CalendarButton::new(
            "<<",
            CalendarEvent::ShowYears(date).to_callback(),
        )],

        _ => unreachable!(),
    };

    month_button.push(bact_to);

    CalendarMarkup::new(month_button)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn returns_all_days_when_no_constraints() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let config = CalendarConfig::default();

        let dates = allowed_dates_for_month(date, &config);

        assert_eq!(dates.len(), 30);
        assert_eq!(dates[0], NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
        assert_eq!(dates[29], NaiveDate::from_ymd_opt(2026, 6, 30).unwrap());
    }

    #[test]
    fn handles_leap_february() {
        let date = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let config = CalendarConfig::default();

        assert_eq!(allowed_dates_for_month(date, &config).len(), 29);
    }

    #[test]
    fn handles_non_leap_february() {
        let date = NaiveDate::from_ymd_opt(2023, 2, 1).unwrap();
        let config = CalendarConfig::default();

        assert_eq!(allowed_dates_for_month(date, &config).len(), 28);
    }

    #[test]
    fn filters_dates_below_min_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let min_date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: None,
        };

        let dates = allowed_dates_for_month(date, &config);

        assert!(dates.iter().all(|d| *d >= min_date));
        assert_eq!(dates.len(), 16);
    }

    #[test]
    fn filters_dates_above_max_date() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let config = CalendarConfig {
            min_date: None,
            max_date: Some(max_date),
        };

        let dates = allowed_dates_for_month(date, &config);

        assert!(dates.iter().all(|d| *d <= max_date));
        assert_eq!(dates.len(), 10);
    }

    #[test]
    fn min_and_max_date_together() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let min_date = NaiveDate::from_ymd_opt(2026, 6, 10).unwrap();
        let max_date = NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: Some(max_date),
        };

        let dates = allowed_dates_for_month(date, &config);

        assert_eq!(dates.len(), 11); // 10..=20
        assert_eq!(dates[0], min_date);
        assert_eq!(*dates.last().unwrap(), max_date);
    }

    #[test]
    fn min_date_outside_current_month_excludes_all() {
        let date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let min_date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let config = CalendarConfig {
            min_date: Some(min_date),
            max_date: None,
        };

        assert!(allowed_dates_for_month(date, &config).is_empty());
    }
}
#[cfg(test)]
mod empty_dates_tests {
    use super::*;
    use chrono::NaiveDate;
    use crate::models::CalendarConfig;
    use crate::locale::types::Locale; 

    #[test]
    fn allowed_dates_for_month_returns_empty_when_fully_excluded() {
        let current_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
        let min_date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
        let config = CalendarConfig { min_date: Some(min_date), max_date: None };

        let dates = allowed_dates_for_month(current_date, &config);

        assert!(dates.is_empty());
    }

    #[test]
fn calendar_grid_returns_none_on_empty_dates() {
    let empty: Vec<NaiveDate> = Vec::new();
    assert_eq!(calendar_grid(&empty), None);
}

   #[test]
fn build_calendar_shows_message_when_month_fully_excluded_by_config() {
    let current_date = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    let min_date = NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
    let config = CalendarConfig { min_date: Some(min_date), max_date: None };
    let locale = Locale::default();

    let markup = build_calendar(current_date, &locale, |d| d.to_string(), &config);

    assert_eq!(markup.rows.len(), 1);
    assert_eq!(markup.rows[0].len(), 1);
    assert_eq!(markup.rows[0][0].text, "No dates available");
}
}