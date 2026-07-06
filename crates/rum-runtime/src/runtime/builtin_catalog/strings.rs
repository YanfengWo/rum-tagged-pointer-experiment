use super::*;

pub(super) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "Blank",
        "Represents the current single-element pattern atom; in string patterns it matches one character.",
    ),
    BuiltinSpec::new(
        "BlankSequence",
        "Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.",
    ),
    BuiltinSpec::new(
        "BlankNullSequence",
        "Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.",
    ),
    BuiltinSpec::new(
        "Pattern",
        "Represents the current named pattern-binding head.",
    ),
    BuiltinSpec::new(
        "Alternatives",
        "Represents the current pattern alternative head.",
    ),
    BuiltinSpec::new("Except", "Represents the current pattern exclusion head."),
    BuiltinSpec::new(
        "StringExpression",
        "Represents the current string-pattern sequence head produced by `~~` syntax.",
    ),
    BuiltinSpec::new(
        "RegularExpression",
        "Represents the current regular-expression string-pattern wrapper.",
    ),
    BuiltinSpec::new(
        "Whitespace",
        "Represents the current string-pattern atom for one or more whitespace characters.",
    ),
    BuiltinSpec::new(
        "WhitespaceCharacter",
        "Represents the current string-pattern atom for one whitespace character.",
    ),
    BuiltinSpec::new(
        "LetterCharacter",
        "Represents the current string-pattern atom for one alphabetic character.",
    ),
    BuiltinSpec::new(
        "DigitCharacter",
        "Represents the current string-pattern atom for one ASCII decimal digit.",
    ),
    BuiltinSpec::new(
        "WordCharacter",
        "Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.",
    ),
    BuiltinSpec::new(
        "NumberString",
        "Represents the current string-pattern atom for decimal-style numeric text.",
    ),
    BuiltinSpec::new(
        "PunctuationCharacter",
        "Represents the current string-pattern atom for one ASCII punctuation character.",
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
        "StartOfLine",
        "Represents the current string-pattern anchor for the start of a string or a position after LF.",
    ),
    BuiltinSpec::new(
        "EndOfLine",
        "Represents the current string-pattern anchor for the end of a string or a position before LF.",
    ),
    BuiltinSpec::new(
        "WordBoundary",
        "Represents the current string-pattern anchor at a transition between word and non-word text.",
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
