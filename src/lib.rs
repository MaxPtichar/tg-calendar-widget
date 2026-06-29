
#![doc = include_str!("../README.md")]
//! A framework-agnostic inline calendar widget builder.
//!
//! Provides date-picker UI logic (month/year navigation, day grids,
//! constrainable date ranges) decoupled from any specific bot framework.
//! Enable the `teloxide` feature for ready-made conversions into
//! `teloxide`'s `InlineKeyboardMarkup`.
//!
//! # Quick start
//!
//! ```
//! use chrono::NaiveDate;
//! use tg_calendar_widget::builder::build_calendar;
//! use tg_calendar_widget::models::CalendarConfig;
//! use tg_calendar_widget::locale::types::Locale;
//!
//! let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
//! let config = CalendarConfig::default();
//! let locale = Locale::default();
//!
//! let markup = build_calendar(date, &locale, |d| d.format("%d.%m.%Y"), &config);
//! assert!(!markup.rows.is_empty());
//! ```

pub mod builder;
pub mod calendar;
pub mod handler;
pub mod locale;
pub mod models;

#[cfg(feature = "teloxide")]
pub mod adapters;
