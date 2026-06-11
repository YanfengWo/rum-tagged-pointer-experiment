use super::*;

pub(super) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "StringJoin",
        "Concatenates strings or a list of strings into a single string.",
        BuiltinHandler::StringJoin,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StringLength",
        "Returns the Unicode code-point length of a string.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::StringLength,
    ),
    BuiltinSpec::ordinary(
        "ToUpperCase",
        "Converts a string or list of strings to uppercase using the current Unicode-aware subset.",
        BuiltinHandler::ToUpperCase,
    ),
    BuiltinSpec::ordinary(
        "StringForm",
        "Formats a template string using the current simple `` placeholder subset.",
        BuiltinHandler::StringForm,
    ),
    BuiltinSpec::ordinary(
        "Row",
        "Renders a list of expressions as a single text string in the current harness subset.",
        BuiltinHandler::Row,
    ),
    BuiltinSpec::ordinary(
        "StringRiffle",
        "Riffles strings together using the current flat and nested-list separator subset.",
        BuiltinHandler::StringRiffle,
    ),
    BuiltinSpec::ordinary(
        "StringPadLeft",
        "Pads strings on the left to a target width using the current simple padding subset.",
        BuiltinHandler::StringPadLeft,
    ),
    BuiltinSpec::ordinary(
        "StringTemplate",
        "Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.",
        BuiltinHandler::StringTemplate,
    ),
    BuiltinSpec::ordinary(
        "TemplateApply",
        "Applies a simple named or positional string template in the current first-pass subset.",
        BuiltinHandler::TemplateApply,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "TextWords",
        "Splits text into alphanumeric word runs in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::TextWords,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToCharacterCode",
        "Returns Unicode scalar values for strings in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::ToCharacterCode,
    ),
    BuiltinSpec::ordinary(
        "ToExpression",
        "Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.",
        BuiltinHandler::ToExpression,
    ),
    BuiltinSpec::ordinary(
        "StringToByteArray",
        "Converts strings to ByteArray objects using the current UTF-8 subset.",
        BuiltinHandler::StringToByteArray,
    ),
    BuiltinSpec::ordinary(
        "Hash",
        "Computes SHA256 hashes over the current string/expression subset.",
        BuiltinHandler::Hash,
    ),
    BuiltinSpec::ordinary(
        "Compress",
        "Encodes expressions into the current project-local compressed string envelope.",
        BuiltinHandler::Compress,
    ),
    BuiltinSpec::ordinary(
        "Uncompress",
        "Recovers values from the current project-local compressed string envelope.",
        BuiltinHandler::Uncompress,
    ),
    BuiltinSpec::ordinary(
        "Encrypt",
        "Wraps values in the current first-pass encrypted object subset using string keys.",
        BuiltinHandler::Encrypt,
    ),
    BuiltinSpec::ordinary(
        "Decrypt",
        "Decrypts the current first-pass encrypted object subset using string or ByteArray keys.",
        BuiltinHandler::Decrypt,
    ),
    BuiltinSpec::ordinary(
        "Nearest",
        "Finds nearest strings or exact numeric values in the current first-pass subset.",
        BuiltinHandler::Nearest,
    ),
    BuiltinSpec::ordinary(
        "Pluralize",
        "Applies a simple English pluralization subset.",
        BuiltinHandler::Pluralize,
    ),
    BuiltinSpec::ordinary(
        "SyntaxQ",
        "Tests whether a string parses successfully in the current syntax subset.",
        BuiltinHandler::SyntaxQ,
    ),
    BuiltinSpec::ordinary(
        "TextString",
        "Converts expressions to the current canonical text subset, leaving raw strings unquoted.",
        BuiltinHandler::TextString,
    ),
    BuiltinSpec::ordinary(
        "FileTemplateApply",
        "Applies a simple file-backed string template and writes the rendered result to a temporary file in the current first-pass subset.",
        BuiltinHandler::FileTemplateApply,
    ),
    BuiltinSpec::ordinary(
        "CharacterCounts",
        "Counts code-point occurrences in a string and returns an association in the current first-pass subset.",
        BuiltinHandler::CharacterCounts,
    ),
    BuiltinSpec::ordinary(
        "RemoveDiacritics",
        "Removes common Latin diacritics from strings in the current first-pass subset.",
        BuiltinHandler::RemoveDiacritics,
    ),
    BuiltinSpec::ordinary(
        "Alphabet",
        "Returns the current English lowercase alphabet subset.",
        BuiltinHandler::Alphabet,
    ),
    BuiltinSpec::ordinary(
        "AlphabeticSort",
        "Sorts strings using the current English alphabet subset with simple diacritic folding.",
        BuiltinHandler::AlphabeticSort,
    ),
    BuiltinSpec::ordinary(
        "EditDistance",
        "Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.",
        BuiltinHandler::EditDistance,
    ),
    BuiltinSpec::ordinary(
        "HammingDistance",
        "Computes mismatch counts for equal-length strings or lists in the current first-pass subset.",
        BuiltinHandler::HammingDistance,
    ),
    BuiltinSpec::ordinary(
        "LongestCommonSubsequence",
        "Finds the longest common contiguous substring or sublist in the current first-pass subset.",
        BuiltinHandler::LongestCommonSubsequence,
    ),
    BuiltinSpec::ordinary(
        "SequenceAlignment",
        "Builds a simple global alignment summary for strings or lists in the current first-pass subset.",
        BuiltinHandler::SequenceAlignment,
    ),
    BuiltinSpec::ordinary(
        "Diff",
        "Builds a simple DiffObject for strings or lists in the current first-pass subset.",
        BuiltinHandler::Diff,
    ),
    BuiltinSpec::ordinary(
        "Characters",
        "Splits a string into a list of single-code-point strings.",
        BuiltinHandler::Characters,
    ),
    BuiltinSpec::ordinary(
        "CharacterRange",
        "Builds an inclusive range of single-character strings between two endpoints.",
        BuiltinHandler::CharacterRange,
    ),
    BuiltinSpec::ordinary(
        "ToString",
        "Renders an evaluated expression into the current canonical kernel text form.",
        BuiltinHandler::ToString,
    ),
    BuiltinSpec::ordinary(
        "StringRepeat",
        "Repeats a string a non-negative number of times.",
        BuiltinHandler::StringRepeat,
    ),
    BuiltinSpec::ordinary(
        "LetterQ",
        "Tests whether all characters in a string belong to the current letter subset.",
        BuiltinHandler::LetterQ,
    ),
    BuiltinSpec::ordinary(
        "DigitQ",
        "Tests whether all characters in a string belong to the current digit subset.",
        BuiltinHandler::DigitQ,
    ),
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
    BuiltinSpec::ordinary(
        "StringContainsQ",
        "Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.",
        BuiltinHandler::StringContainsQ,
    ),
    BuiltinSpec::ordinary(
        "StringStartsQ",
        "Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.",
        BuiltinHandler::StringStartsQ,
    ),
    BuiltinSpec::ordinary(
        "StringEndsQ",
        "Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.",
        BuiltinHandler::StringEndsQ,
    ),
    BuiltinSpec::ordinary(
        "StringPosition",
        "Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.",
        BuiltinHandler::StringPosition,
    ),
    BuiltinSpec::ordinary(
        "StringSplit",
        "Splits strings using whitespace or exact delimiters in the current first-pass string subset.",
        BuiltinHandler::StringSplit,
    ),
    BuiltinSpec::ordinary(
        "StringTrim",
        "Trims whitespace or exact boundary substrings in the current first-pass string subset.",
        BuiltinHandler::StringTrim,
    ),
    BuiltinSpec::ordinary(
        "StringReplace",
        "Replaces exact string needles using immediate string rules in the current first-pass string subset.",
        BuiltinHandler::StringReplace,
    ),
    BuiltinSpec::ordinary(
        "StringCases",
        "Extracts non-overlapping exact string matches or immediate exact-string rule outputs in the current first-pass string subset.",
        BuiltinHandler::StringCases,
    ),
    BuiltinSpec::ordinary(
        "StringApply",
        "Applies a list-oriented function to string code points and reassembles string output in the current first-pass subset.",
        BuiltinHandler::StringApply,
    ),
    BuiltinSpec::ordinary(
        "StringDelete",
        "Deletes exact or current string-pattern matches from strings in the current first-pass subset.",
        BuiltinHandler::StringDelete,
    ),
    BuiltinSpec::ordinary(
        "StringTake",
        "Extracts a substring using integer counts or inclusive 1-based ranges.",
        BuiltinHandler::StringTake,
    ),
    BuiltinSpec::ordinary(
        "StringDrop",
        "Drops a substring using integer counts or inclusive 1-based ranges.",
        BuiltinHandler::StringDrop,
    ),
    BuiltinSpec::ordinary(
        "StringTakeDrop",
        "Returns both the taken and dropped substrings using the current StringTake/StringDrop spec subset.",
        BuiltinHandler::StringTakeDrop,
    ),
    BuiltinSpec::ordinary(
        "StringPart",
        "Extracts individual characters by 1-based positions.",
        BuiltinHandler::StringPart,
    ),
    BuiltinSpec::ordinary(
        "StringCount",
        "Counts exact or current string-pattern matches, including the current IgnoreCase and Overlaps option subset.",
        BuiltinHandler::StringCount,
    ),
    BuiltinSpec::ordinary(
        "StringMatchQ",
        "Tests whether a whole string matches an exact or current string-pattern spec.",
        BuiltinHandler::StringMatchQ,
    ),
    BuiltinSpec::ordinary(
        "StringFreeQ",
        "Tests whether a string is free of exact or current string-pattern matches.",
        BuiltinHandler::StringFreeQ,
    ),
];
