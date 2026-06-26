use chrono::{Datelike, Months, NaiveDate};

use crate::{
    builder::calendar_grid,
    locale::types::Locale,
    models::{CalendarButton, CalendarConfig, CalendarEvent, CalendarMarkup},
};




/// Builds the main day-grid UI: year row, month-navigation row, weekday
/// header row, and the day grid itself (6 rows × 7 columns, Monday-first).
///
/// Cells outside the current month (leading/trailing blanks) and dates
/// disallowed by `config` are rendered as empty buttons with an
/// [`CalendarEvent::Ignore`] callback, so taps on them are no-ops.
///
/// If `config` excludes every day of the month (e.g. `min_date`/`max_date`
/// don't overlap with `dates`'s month at all), returns a single-row markup
/// reading "No dates available" instead of an empty or malformed grid.
///
/// # Layout
///
/// ```text
/// +---------------------------------------------------------------------------------+
/// |                                       2017                                      |
/// +---------------------------+-----------------------------+-----------------------+
/// |             <<            |            March            |           >>          |
/// +-----------+-----------+---+-------+-----------+---------+--+-----------+-----------+
/// |    Mon    |    Tue    |    Wed    |    Thu    |    Fri    |    Sat    |    Sun    |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// |           |           |     1     |     2     |     3     |     4     |     5     |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// |     6     |     7     |     8     |     9     |    10     |    11     |    12     |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// |    13     |    14     |    15     |    16     |    17     |    18     |    19     |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// |    20     |    21     |    22     |    23     |    24     |    25     |    26     |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// |    27     |    28     |    29     |    30     |    31     |           |           |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// |           |           |           |           |           |           |           |
/// +-----------+-----------+-----------+-----------+-----------+-----------+-----------+
/// ```
///
/// # Note on `config` constraints
///
/// Dates disallowed by `config.min_date`/`config.max_date` currently render
/// as a **blank** button (same as out-of-month blanks), not as a disabled
/// button showing the day number. If you need the day number visible but
/// disabled, the filtering logic needs to distinguish "out of month" from
/// "in month but disallowed" before reaching this rendering step.
pub fn calendar_button<T, F>(
    dates: Vec<NaiveDate>,
    locales: &Locale,
    date_callback: F,
    config: &CalendarConfig,
) -> CalendarMarkup
where
    F: Fn(NaiveDate) -> T,
    T: ToString,
{
     let Some(start_date) = dates.first() else {
        return CalendarMarkup::new(vec![vec![CalendarButton::new(
            "No dates available",
            CalendarEvent::Ignore.to_callback(),
        )]]);
    };

    let grid = calendar_grid(&dates).unwrap_or([None; 42]);

    let year_button = year_button(start_date);

    let prev_next_month = prev_next_month(start_date, locales, config);

    let mut calendar: Vec<Vec<CalendarButton>> = Vec::new();

    calendar.push(year_button);
    calendar.push(prev_next_month);

    calendar.push(week_days(locales));

    let month: Vec<Vec<CalendarButton>> = grid
        .iter()
        .filter_map(|opt_date| {
            if let Some(date) = opt_date {
                let button = CalendarButton::new(
                    date.day().to_string(),
                    date_callback(*date).to_string(), //сделать каллбэк
                );

                Some(button)
            } else {
                let button = CalendarButton::new(" ", CalendarEvent::Ignore.to_callback());

                Some(button)
            }
        })
        .collect::<Vec<CalendarButton>>()
        .chunks(7)
        .enumerate()
        .map(|(row, chunk)| (row, chunk))
        .take_while(|(row, _)| *row != 6)
        .map(|(_, chunk)| chunk.to_vec())
        .collect();

    calendar.extend(month);

    let res = CalendarMarkup::new(calendar);

    res
}



/// Builds the navigation row above the day grid: previous/next month
/// buttons flanking a central button that opens the month picker.
///
/// # Layout
///
/// ```text
/// +--------+----------+----------+
/// |   <<   | October  |    >>    |
/// +--------+----------+----------+
/// ```
///
/// If `config.min_date`/`config.max_date` would put the previous/next
/// month entirely out of range, that side's button is rendered blank
/// with an [`CalendarEvent::Ignore`] callback instead of `<<`/`>>`.
pub fn prev_next_month(
    date: &NaiveDate,
    locales: &Locale,
    config: &CalendarConfig,
) -> Vec<CalendarButton> {
    let prev_month = date.checked_sub_months(Months::new(1)).unwrap();
    let next_month = date.checked_add_months(Months::new(1)).unwrap();

    let month = date.month() - 1;

    let display_month = locales.months[month as usize];

    let back_button = if let Some(min_date_config) = config.min_date {
        if (prev_month.year(), prev_month.month())
            >= (min_date_config.year(), min_date_config.month())
        {
            CalendarButton::new("<<", CalendarEvent::MoveMonth(prev_month).to_callback())
        } else {
            CalendarButton::new(" ", CalendarEvent::Ignore.to_callback())
        }
    } else {
        CalendarButton::new("<<", CalendarEvent::MoveMonth(prev_month).to_callback())
    };

    let forward_button = if let Some(max_date_config) = config.max_date {
        if (next_month.year(), next_month.month())
            <= (max_date_config.year(), max_date_config.month())
        {
            CalendarButton::new(">>", CalendarEvent::MoveMonth(next_month).to_callback())
        } else {
            CalendarButton::new(" ", CalendarEvent::Ignore.to_callback())
        }
    } else {
        CalendarButton::new(">>", CalendarEvent::MoveMonth(next_month).to_callback())
    };

    vec![
        back_button,
        CalendarButton::new(
            format!("{display_month}"),
            CalendarEvent::ShowMonths(*date, crate::models::BackTo::BackToCalendar).to_callback(),
        ),
        forward_button,
    ]
}

/// Builds the single-button row showing the current year; tapping it
/// opens the year picker grid.
///
/// # Layout
///
/// ```text
/// +-------------------------------+
/// |             2016              |
/// +-------------------------------+
/// ```
pub fn year_button(date: &NaiveDate) -> Vec<CalendarButton> {
    vec![CalendarButton::new(
        date.year().to_string(),
        CalendarEvent::ShowYears(*date).to_callback(),
    )]
}


/// Construct day's name grid
fn week_days(locale: &Locale) -> Vec<CalendarButton> {
    let days_name: [&str; 7] = locale.week_days;

    days_name
        .iter()
        .map(|day| CalendarButton::new(*day, CalendarEvent::Ignore.to_callback()))
        .collect::<Vec<CalendarButton>>()
}
