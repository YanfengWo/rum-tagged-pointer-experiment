use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::datetime;

pub(crate) const DATETIME_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_attributes(
        "DateList",
        "Converts an exact Unix timestamp to a UTC date list, or reads the current UTC date list in the zero-argument form.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_list,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DateString",
        "Formats an exact timestamp or date list as an ISO 8601 UTC string.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_string,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "FromDateString",
        "Parses the documented ISO 8601 UTC/fixed-offset subset into an exact Unix timestamp.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_from_date_string,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DatePlus",
        "Adds an exact fixed-duration offset to an exact timestamp or date list.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_plus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DateDifference",
        "Returns the exact fixed-duration difference between two timestamps or date lists.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_date_difference,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DayOfWeek",
        "Returns the ISO weekday symbol for an exact timestamp or date list.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_day_of_week,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LeapYearQ",
        "Tests whether an exact integer year is a leap year in the proleptic Gregorian calendar.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_leap_year_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "DaysInMonth",
        "Returns the number of days in a proleptic Gregorian month.",
        AttributeSet::empty().with(Attribute::Protected),
        datetime::eval_days_in_month,
    ),
];
