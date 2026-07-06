use rum_runtime::kernel_api::BuiltinSpec;

use crate::datetime;

pub(crate) const DATETIME_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "DateList",
        "Converts an exact Unix timestamp to a UTC date list, or reads the current UTC date list in the zero-argument form.",
        datetime::eval_date_list,
    ),
    BuiltinSpec::ordinary(
        "DateString",
        "Formats an exact timestamp or date list as an ISO 8601 UTC string.",
        datetime::eval_date_string,
    ),
    BuiltinSpec::ordinary(
        "FromDateString",
        "Parses the documented ISO 8601 UTC/fixed-offset subset into an exact Unix timestamp.",
        datetime::eval_from_date_string,
    ),
    BuiltinSpec::ordinary(
        "DatePlus",
        "Adds an exact fixed-duration offset to an exact timestamp or date list.",
        datetime::eval_date_plus,
    ),
    BuiltinSpec::ordinary(
        "DateDifference",
        "Returns the exact fixed-duration difference between two timestamps or date lists.",
        datetime::eval_date_difference,
    ),
    BuiltinSpec::ordinary(
        "DayOfWeek",
        "Returns the ISO weekday symbol for an exact timestamp or date list.",
        datetime::eval_day_of_week,
    ),
    BuiltinSpec::ordinary(
        "LeapYearQ",
        "Tests whether an exact integer year is a leap year in the proleptic Gregorian calendar.",
        datetime::eval_leap_year_q,
    ),
    BuiltinSpec::ordinary(
        "DaysInMonth",
        "Returns the number of days in a proleptic Gregorian month.",
        datetime::eval_days_in_month,
    ),
];
