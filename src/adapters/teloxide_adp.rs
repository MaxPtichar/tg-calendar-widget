//! Adapter converting calendar primitives into `teloxide` UI types.
//!
//! The core calendar logic ([`CalendarButton`], [`CalendarMarkup`]) is
//! framework-agnostic, so it can be rendered by any bot library. This
//! module provides `From` conversions into `teloxide`'s
//! [`InlineKeyboardButton`]/[`InlineKeyboardMarkup`] for projects using
//! `teloxide`.

use crate::models::{CalendarButton, CalendarMarkup};
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup};

/// Converts a [`CalendarButton`] into a `teloxide` callback button.
impl From<CalendarButton> for InlineKeyboardButton {
    fn from(value: CalendarButton) -> Self {
        InlineKeyboardButton::callback(value.text, value.callback_data)
    }
}

/// Converts a full [`CalendarMarkup`] grid into a `teloxide`
/// [`InlineKeyboardMarkup`], preserving row/column structure.
///
/// # Examples
///
/// ```
/// use chrono::NaiveDate;
/// use teloxide::types::InlineKeyboardMarkup;
/// use calendar::builder::build_calendar;
/// use calendar::models::CalendarConfig;
/// use calendar::locale::types::Locale;
///
/// let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
/// let config = CalendarConfig::default();
/// let locale = Locale::default();
///
/// let markup = build_calendar(date, &locale, |d| d.format("%d.%m.%Y"), &config);
/// let keyboard: InlineKeyboardMarkup = markup.into();
/// ```
impl From<CalendarMarkup> for InlineKeyboardMarkup {
    fn from(value: CalendarMarkup) -> Self {
        let rows = value
            .rows
            .into_iter()
            .map(|row| {
                row.into_iter()
                    .map(|button| InlineKeyboardButton::from(button))
                    .collect::<Vec<InlineKeyboardButton>>()
            })
            .collect::<Vec<Vec<InlineKeyboardButton>>>();

        InlineKeyboardMarkup::new(rows)
    }
}
