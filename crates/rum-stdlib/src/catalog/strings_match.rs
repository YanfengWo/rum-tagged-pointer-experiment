use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use super::message_templates::*;

use crate::strings;

pub(crate) const STRING_MATCH_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Characters",
        "Splits a string into a list of single-code-point strings.",
        strings::eval_characters,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "CharacterRange",
        "Builds an inclusive range of single-character strings between two endpoints.",
        strings::eval_character_range,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ToString",
        "Renders an evaluated expression into the current canonical kernel text form.",
        strings::eval_to_string,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringRepeat",
        "Repeats a string a non-negative number of times.",
        strings::eval_string_repeat,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "LetterQ",
        "Tests whether all characters in a string belong to the current letter subset.",
        strings::eval_letter_q,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "LowerCaseQ",
        "Tests whether every character in a string is lowercase.",
        AttributeSet::from_attribute(Attribute::Protected),
        strings::eval_lower_case_q,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "UpperCaseQ",
        "Tests whether every character in a string is uppercase.",
        AttributeSet::from_attribute(Attribute::Protected),
        strings::eval_upper_case_q,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary(
        "DigitQ",
        "Tests whether all characters in a string belong to the current digit subset.",
        strings::eval_digit_q,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringContainsQ",
        "Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.",
        strings::eval_string_contains_q,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringStartsQ",
        "Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.",
        strings::eval_string_starts_q,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringEndsQ",
        "Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.",
        strings::eval_string_ends_q,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringPosition",
        "Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.",
        strings::eval_string_position,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringSplit",
        "Splits strings using whitespace or exact delimiters in the current first-pass string subset.",
        strings::eval_string_split,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringTrim",
        "Trims whitespace or exact boundary substrings in the current first-pass string subset.",
        strings::eval_string_trim,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringReplace",
        "Replaces exact string needles using immediate string rules in the current first-pass string subset.",
        strings::eval_string_replace,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringCases",
        "Extracts non-overlapping exact string matches or immediate exact-string rule outputs in the current first-pass string subset.",
        strings::eval_string_cases,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringApply",
        "Applies a list-oriented function to string code points and reassembles string output in the current first-pass subset.",
        strings::eval_string_apply,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringDelete",
        "Deletes exact or current string-pattern matches from strings in the current first-pass subset.",
        strings::eval_string_delete,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringTake",
        "Extracts a substring using integer counts or inclusive 1-based ranges.",
        strings::eval_string_take,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringDrop",
        "Drops a substring using integer counts or inclusive 1-based ranges.",
        strings::eval_string_drop,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringTakeDrop",
        "Returns both the taken and dropped substrings using the current StringTake/StringDrop spec subset.",
        strings::eval_string_take_drop,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringPart",
        "Extracts individual characters by 1-based positions.",
        strings::eval_string_part,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringCount",
        "Counts exact or current string-pattern matches, including the current IgnoreCase and Overlaps option subset.",
        strings::eval_string_count,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringMatchQ",
        "Tests whether a whole string matches an exact or current string-pattern spec.",
        strings::eval_string_match_q,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringFreeQ",
        "Tests whether a string is free of exact or current string-pattern matches.",
        strings::eval_string_free_q,
    )
    .with_messages(&[
        ARGTU,
        ARGX_OPERATOR,
    ]).protected(),
];
