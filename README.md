# tg-calendar-widget

Calendar grid widget for Telegram bots — day, month, and year pickers, with optional teloxide support.

## Features

* Create grid of days/month/years
* Customizable callback data format for day selection
* Date constraints
* Localization RU/EN

## Installation
 
```toml
[dependencies]
tg-calendar-widget = { version = "0.1.0", features = ["teloxide"] }
chrono = "0.4.38"
```
 
Drop the `teloxide` feature if you only need the core grid-building logic without the `teloxide` conversions.
 
## Quick start (core logic only)
 
```rust
use chrono::NaiveDate;
use tg_calendar_widget::builder::build_calendar;
use tg_calendar_widget::models::CalendarConfig;
use tg_calendar_widget::locale::types::Locale;
 
let date = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();
let config = CalendarConfig::default(); // no constraints
let locale = Locale::default(); // English
 
let markup = build_calendar(date, &locale, |d| d.format("%d.%m.%Y"), &config);
```
 
`markup` is a `CalendarMarkup` — a framework-agnostic grid of `CalendarButton`s. Enable the `teloxide` feature to convert it directly into an `InlineKeyboardMarkup`:
 
```rust, ignore
use teloxide::types::InlineKeyboardMarkup;
 
let keyboard: InlineKeyboardMarkup = markup.into();
```
 
## Constraining the date range
 
Only allow dates up to today (e.g. for a "log when this happened" picker):
 
```rust
use chrono::Utc;
use tg_calendar_widget::models::CalendarConfig;
 
let config = CalendarConfig {
    min_date: None,
    max_date: Some(Utc::now().date_naive()),
};
```
 
Dates outside the configured range render as blank, non-interactive buttons rather than being removed from the grid — so the grid layout stays stable across navigation.
 
## Locales
 
```rust
use tg_calendar_widget::locale::translations;
 
let locale = translations::RU; // or translations::EN
```
 
Add your own by constructing a `Locale`:
 
```rust
use chrono::Weekday;
use tg_calendar_widget::locale::types::Locale;
 
const FR: Locale = Locale {
    week_days: ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"],
    months: [
        "Janvier", "Février", "Mars", "Avril", "Mai", "Juin",
        "Juillet", "Août", "Septembre", "Octobre", "Novembre", "Décembre",
    ],
    first_weekday: Weekday::Mon,
};
```
 
## Full `teloxide` integration
 
This is the pattern used in production: a `dptree` filter routes calendar callbacks to a dedicated handler, which renders the grid and edits the message in place.
 
```rust, ignore
use teloxide::dispatching::UpdateFilterExt;
use teloxide::prelude::*;
use teloxide::types::{CallbackQuery, InlineKeyboardMarkup};
use chrono::Utc;
use tg_calendar_widget::models::{CalendarAction, CalendarConfig};
use tg_calendar_widget::handler::is_calendar_callback;
use tg_calendar_widget::locale::translations;
 
/// dptree filter — only routes callbacks belonging to the calendar's
/// namespace (including disabled/ignore button taps) into `calendar_handle`.
pub fn calendar_filter(q: CallbackQuery) -> bool {
    q.data
        .as_deref()
        .is_some_and(|d| is_calendar_callback(d) || d == "calendar")
}
 
pub async fn calendar_handle(bot: Bot, q: CallbackQuery) -> ResponseResult<()> {
    bot.answer_callback_query(q.id).await?;
 
    let data = q.data.as_deref().unwrap_or("");
    let current_date = Utc::now().date_naive();
    let message = q.message.as_ref().unwrap();
    let chat_id = message.chat().id;
    let msg_id = message.id();
 
    // Example: only allow picking dates up to today.
    let config = CalendarConfig { min_date: None, max_date: Some(current_date) };
    let locale = translations::RU;
 
    match data {
        "calendar" => {
            let calendar: InlineKeyboardMarkup = tg_calendar_widget::builder::build_calendar(
                current_date,
                &locale,
                |d| d.format("%d.%m.%Y"),
                &config,
            )
            .into();
 
            bot.edit_message_text(chat_id, msg_id, "Календарь")
                .reply_markup(calendar)
                .await?;
        }
        _ => match CalendarAction::handle_callback(
            data,
            &locale,
            |d| d.format("%d.%m.%Y"),
            &config,
        ) {
            Some(CalendarAction::Redraw(markup)) => {
                bot.edit_message_text(chat_id, msg_id, "Календарь")
                    .reply_markup(markup.into())
                    .await?;
            }
            // A non-calendar callback (or a no-op `ignore` tap) — nothing to do.
            Some(CalendarAction::Custom(_)) | None => {}
        },
    }
 
    Ok(())
}
 
pub fn calendar_schema() -> teloxide::dispatching::UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    Update::filter_callback_query()
        .filter(calendar_filter)
        .endpoint(calendar_handle)
}
```
 
Reading a selected date back out of a `Custom` callback (e.g. when the day button's callback data is the date itself, not a calendar-internal command):
 
```rust, , ignore
use chrono::NaiveDate;
 
if let Some(CalendarAction::Custom(data)) = action {
    if let Ok(date) = NaiveDate::parse_from_str(&data, "%d.%m.%Y") {
        // use the selected date
    }
}
```

## Callback data format

Calendar-internal navigation events are encoded as `c:<command>:<date>`, e.g. `c:move:23.03.2026`. This namespace is reserved — pick a different prefix for your own custom callback data routed through the same handler.
 
## Known limitations

- Dates disallowed by `CalendarConfig` render as **blank** buttons, indistinguishable from out-of-month blanks — there's currently no "visible but disabled" state showing the day number.
- `Locale::first_weekday` is reserved for future use; the grid is currently always Monday-first regardless of its value.
## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.