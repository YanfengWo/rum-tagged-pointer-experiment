use super::*;

pub(super) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "Whitespace",
        "Represents the current string-pattern atom for one or more whitespace characters.",
    ),
    BuiltinSpec::new(
        "NumberString",
        "Represents the current string-pattern atom for decimal-style numeric text.",
    ),
    BuiltinSpec::new(
        "IgnoreCase",
        "Represents the current option symbol for case-insensitive string matching.",
    ),
    BuiltinSpec::new(
        "Overlaps",
        "Represents the current option symbol for overlapping string-match counting.",
    ),
    BuiltinSpec::new(
        "StartOfString",
        "Represents the current string-pattern anchor for the start of a string.",
    ),
    BuiltinSpec::new(
        "EndOfString",
        "Represents the current string-pattern anchor for the end of a string.",
    ),
    BuiltinSpec::new(
        "Repeated",
        "Represents the current string-pattern repetition head for one or more matches.",
    ),
    BuiltinSpec::new(
        "RepeatedNull",
        "Represents the current string-pattern repetition head for zero or more matches.",
    ),
    BuiltinSpec::new(
        "Shortest",
        "Represents the current string-pattern wrapper that prefers the shortest match.",
    ),
    BuiltinSpec::new(
        "Longest",
        "Represents the current string-pattern wrapper that prefers the longest match.",
    ),
    BuiltinSpec::new(
        "TemplateObject",
        "Represents the current simple template object wrapper returned by StringTemplate.",
    ),
    BuiltinSpec::new(
        "DiffObject",
        "Represents the current diff result object returned by Diff.",
    ),
    BuiltinSpec::new(
        "ByteArray",
        "Represents raw byte payloads using base64 text in the current first-pass subset.",
    ),
    BuiltinSpec::new(
        "EncryptedObject",
        "Represents the current first-pass encrypted object envelope returned by Encrypt.",
    ),
    BuiltinSpec::new(
        "InputForm",
        "Represents the current formatting symbol accepted by the current ToExpression subset.",
    ),
    BuiltinSpec::new(
        "OutputForm",
        "Represents the current formatting symbol accepted by the current ToString subset.",
    ),
    BuiltinSpec::new(
        "StandardForm",
        "Represents the current formatting symbol accepted by the current ToExpression subset.",
    ),
    BuiltinSpec::new(
        "TraditionalForm",
        "Represents the current formatting symbol accepted by the current ToString subset.",
    ),
    BuiltinSpec::new(
        "Automatic",
        "Represents the current default-specifier symbol shared by several first-pass string and tooling subsets.",
    ),
];
