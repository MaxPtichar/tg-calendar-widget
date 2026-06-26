use chrono::Weekday;

use crate::locale::types::Locale;

pub const EN: Locale = Locale {
    week_days: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],

    months: [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],

    first_weekday: Weekday::Mon,
};

pub const RU: Locale = Locale {
    week_days: ["Пн", "Вт", "Ср", "Чт", "Пт", "Сб", "Вс"],

    months: [
        "Январь",
        "Февраль",
        "Март",
        "Апрель",
        "Май",
        "Июнь",
        "Июль",
        "Август",
        "Сентябрь",
        "Октябрь",
        "Ноябрь",
        "Декабрь",
    ],

    first_weekday: Weekday::Mon,
};

impl Default for Locale {
    fn default() -> Self {
        EN
    }
}
