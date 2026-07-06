use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};

use crate::strings;

pub(crate) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::new(
        "Alternatives",
        "Represents the current pattern alternative head.",
    ),
    BuiltinSpec::new(
        "Blank",
        "Represents the current single-element pattern atom; in string patterns it matches one character.",
    ),
    BuiltinSpec::new(
        "BlankNullSequence",
        "Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.",
    ),
    BuiltinSpec::new(
        "BlankSequence",
        "Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.",
    ),
    BuiltinSpec::new(
        "DigitCharacter",
        "Represents the current string-pattern atom for one ASCII decimal digit.",
    ),
    BuiltinSpec::new(
        "EndOfLine",
        "Represents the current string-pattern anchor for the end of a string or a position before LF.",
    ),
    BuiltinSpec::new(
        "EndOfString",
        "Represents the current string-pattern anchor for the end of a string.",
    ),
    BuiltinSpec::new("Except", "Represents the current pattern exclusion head."),
    BuiltinSpec::new(
        "LetterCharacter",
        "Represents the current string-pattern atom for one alphabetic character.",
    ),
    BuiltinSpec::new(
        "Pattern",
        "Represents the current named pattern-binding head.",
    ),
    BuiltinSpec::new(
        "PunctuationCharacter",
        "Represents the current string-pattern atom for one ASCII punctuation character.",
    ),
    BuiltinSpec::new(
        "RegularExpression",
        "Represents the current regular-expression string-pattern wrapper.",
    ),
    BuiltinSpec::new(
        "StartOfLine",
        "Represents the current string-pattern anchor for the start of a string or a position after LF.",
    ),
    BuiltinSpec::new(
        "StartOfString",
        "Represents the current string-pattern anchor for the start of a string.",
    ),
    BuiltinSpec::new(
        "StringExpression",
        "Represents the current string-pattern sequence head produced by `~~` syntax.",
    ),
    BuiltinSpec::new(
        "WhitespaceCharacter",
        "Represents the current string-pattern atom for one whitespace character.",
    ),
    BuiltinSpec::new(
        "WordBoundary",
        "Represents the current string-pattern anchor at a transition between word and non-word text.",
    ),
    BuiltinSpec::new(
        "WordCharacter",
        "Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.",
    ),
    BuiltinSpec::ordinary(
        "StringJoin",
        "Concatenates strings or a list of strings into a single string.",
        strings::eval_string_join,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StringLength",
        "Returns the Unicode code-point length of a string.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_string_length,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "StringReverse",
        "Reverses the Unicode code-point characters in a string.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_string_reverse,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToLowerCase",
        "Converts a string to lowercase using the current Unicode-aware subset.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_to_lower_case,
    ),
    BuiltinSpec::ordinary(
        "ToUpperCase",
        "Converts a string or list of strings to uppercase using the current Unicode-aware subset.",
        strings::eval_to_upper_case,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Capitalize",
        "Uppercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_capitalize,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Decapitalize",
        "Lowercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_decapitalize,
    ),
    BuiltinSpec::ordinary(
        "StringForm",
        "Formats a template string eagerly, replacing `` and `n` placeholders with ToString-rendered arguments.",
        strings::eval_string_form,
    ),
    BuiltinSpec::ordinary(
        "StringWrap",
        "Greedily word-wraps a string to a positive character width, separating output lines with LF.",
        strings::eval_string_wrap,
    ),
    BuiltinSpec::ordinary(
        "StringIndent",
        "Prefixes each non-blank line of a string with a string prefix or an exact number of spaces.",
        strings::eval_string_indent,
    ),
    BuiltinSpec::ordinary(
        "StringDedent",
        "Removes the longest common leading whitespace from the non-blank lines of a string.",
        strings::eval_string_dedent,
    ),
    BuiltinSpec::ordinary(
        "StringExpandTabs",
        "Expands tabs in a string to the next fixed-width column stop, defaulting to width 8.",
        strings::eval_string_expand_tabs,
    ),
    BuiltinSpec::ordinary(
        "AlignColumns",
        "Formats a list of string rows as left-aligned columns separated by two spaces.",
        strings::eval_align_columns,
    ),
    BuiltinSpec::ordinary(
        "HtmlEscape",
        "Escapes the five HTML text characters &, <, >, double quote, and single quote.",
        strings::eval_html_escape,
    ),
    BuiltinSpec::ordinary(
        "HtmlUnescape",
        "Unescapes the five named HTML entities and decimal or hexadecimal numeric character references, leaving unknown entities verbatim.",
        strings::eval_html_unescape,
    ),
    BuiltinSpec::ordinary(
        "ShellSplit",
        "Splits a POSIX-style shell word string with whitespace, quotes, and backslash escapes, returning $Failed on malformed quoting.",
        strings::eval_shell_split,
    ),
    BuiltinSpec::ordinary(
        "ShellQuote",
        "Single-quotes a string for POSIX-style shell input, escaping embedded single quotes.",
        strings::eval_shell_quote,
    ),
    BuiltinSpec::ordinary(
        "FileExtension",
        "Returns the final path component extension without the dot, or an empty string when no extension is present.",
        strings::eval_file_extension,
    ),
    BuiltinSpec::ordinary(
        "FileBaseName",
        "Returns the final path component without its extension.",
        strings::eval_file_base_name,
    ),
    BuiltinSpec::ordinary(
        "FileNameSplit",
        "Splits a slash-separated path string into components, preserving an absolute leading slash as its own component.",
        strings::eval_file_name_split,
    ),
    BuiltinSpec::ordinary(
        "UrlEncode",
        "Percent-encodes a string using UTF-8 bytes and the RFC 3986 unreserved character set.",
        strings::eval_url_encode,
    ),
    BuiltinSpec::ordinary(
        "UrlDecode",
        "Decodes percent escapes in a UTF-8 URL string, returning $Failed for invalid escapes or invalid UTF-8.",
        strings::eval_url_decode,
    ),
    BuiltinSpec::ordinary(
        "UrlParse",
        "Parses scheme://host[:port][/path][?query][#fragment] into an association of string fields.",
        strings::eval_url_parse,
    ),
    BuiltinSpec::ordinary(
        "ParseArgs",
        "Parses long-form script arguments into positional arguments, --key=value options, and --flag names.",
        strings::eval_parse_args,
    ),
    BuiltinSpec::ordinary(
        "Row",
        "Renders a list of expressions as a single text string in the current harness subset.",
        strings::eval_row,
    ),
    BuiltinSpec::ordinary(
        "StringRiffle",
        "Riffles strings together using the current flat and nested-list separator subset.",
        strings::eval_string_riffle,
    ),
    BuiltinSpec::ordinary(
        "StringPadLeft",
        "Pads strings on the left to a target width using the current simple padding subset.",
        strings::eval_string_pad_left,
    ),
    BuiltinSpec::ordinary(
        "StringPadRight",
        "Pads strings on the right to a target width using the current simple padding subset; values at or above width are unchanged.",
        strings::eval_string_pad_right,
    ),
    BuiltinSpec::ordinary(
        "StringLines",
        "Splits a string on LF or CRLF newlines, preserving interior empty lines.",
        strings::eval_string_lines,
    ),
    BuiltinSpec::ordinary(
        "StringTemplate",
        "Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.",
        strings::eval_string_template,
    ),
    BuiltinSpec::ordinary(
        "TemplateApply",
        "Applies a simple named or positional string template in the current first-pass subset.",
        strings::eval_template_apply,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "TextWords",
        "Splits text into alphanumeric word runs in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_text_words,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "ToCharacterCode",
        "Returns Unicode scalar values for strings in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_to_character_code,
    ),
    BuiltinSpec::ordinary(
        "ToExpression",
        "Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.",
        strings::eval_to_expression,
    ),
    BuiltinSpec::ordinary(
        "StringToByteArray",
        "Converts strings to ByteArray objects using the current UTF-8 subset.",
        strings::eval_string_to_byte_array,
    ),
    BuiltinSpec::ordinary(
        "Hash",
        "Computes SHA256 hashes over the current string/expression subset.",
        strings::eval_hash,
    ),
    BuiltinSpec::ordinary(
        "Compress",
        "Encodes expressions into the current project-local compressed string envelope.",
        strings::eval_compress,
    ),
    BuiltinSpec::ordinary(
        "Uncompress",
        "Recovers values from the current project-local compressed string envelope.",
        strings::eval_uncompress,
    ),
    BuiltinSpec::ordinary(
        "Encrypt",
        "Wraps values in the current first-pass encrypted object subset using string keys.",
        strings::eval_encrypt,
    ),
    BuiltinSpec::ordinary(
        "Decrypt",
        "Decrypts the current first-pass encrypted object subset using string or ByteArray keys.",
        strings::eval_decrypt,
    ),
    BuiltinSpec::ordinary(
        "Nearest",
        "Finds nearest strings or exact numeric values in the current first-pass subset.",
        strings::eval_nearest,
    ),
    BuiltinSpec::ordinary(
        "Pluralize",
        "Applies a simple English pluralization subset.",
        strings::eval_pluralize,
    ),
    BuiltinSpec::ordinary(
        "SyntaxQ",
        "Tests whether a string parses successfully in the current syntax subset.",
        strings::eval_syntax_q,
    ),
    BuiltinSpec::ordinary(
        "TextString",
        "Converts expressions to the current canonical text subset, leaving raw strings unquoted.",
        strings::eval_text_string,
    ),
    BuiltinSpec::ordinary(
        "CharacterCounts",
        "Counts code-point occurrences in a string and returns an association in the current first-pass subset.",
        strings::eval_character_counts,
    ),
    BuiltinSpec::ordinary(
        "RemoveDiacritics",
        "Removes common Latin diacritics from strings in the current first-pass subset.",
        strings::eval_remove_diacritics,
    ),
    BuiltinSpec::ordinary(
        "Alphabet",
        "Returns the current English lowercase alphabet subset.",
        strings::eval_alphabet,
    ),
    BuiltinSpec::ordinary(
        "AlphabeticSort",
        "Sorts strings using the current English alphabet subset with simple diacritic folding.",
        strings::eval_alphabetic_sort,
    ),
    BuiltinSpec::ordinary(
        "EditDistance",
        "Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.",
        strings::eval_edit_distance,
    ),
    BuiltinSpec::ordinary(
        "HammingDistance",
        "Computes mismatch counts for equal-length strings or lists in the current first-pass subset.",
        strings::eval_hamming_distance,
    ),
    BuiltinSpec::ordinary(
        "LongestCommonSubsequence",
        "Finds the longest common contiguous substring or sublist in the current first-pass subset.",
        strings::eval_longest_common_subsequence,
    ),
    BuiltinSpec::ordinary(
        "SequenceAlignment",
        "Builds a simple global alignment summary for strings or lists in the current first-pass subset.",
        strings::eval_sequence_alignment,
    ),
    BuiltinSpec::ordinary(
        "Diff",
        "Builds a simple DiffObject for strings or lists in the current first-pass subset.",
        strings::eval_diff,
    ),
    BuiltinSpec::ordinary(
        "Characters",
        "Splits a string into a list of single-code-point strings.",
        strings::eval_characters,
    ),
    BuiltinSpec::ordinary(
        "CharacterRange",
        "Builds an inclusive range of single-character strings between two endpoints.",
        strings::eval_character_range,
    ),
    BuiltinSpec::ordinary(
        "ToString",
        "Renders an evaluated expression into the current canonical kernel text form.",
        strings::eval_to_string,
    ),
    BuiltinSpec::ordinary(
        "StringRepeat",
        "Repeats a string a non-negative number of times.",
        strings::eval_string_repeat,
    ),
    BuiltinSpec::ordinary(
        "LetterQ",
        "Tests whether all characters in a string belong to the current letter subset.",
        strings::eval_letter_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "LowerCaseQ",
        "Tests whether every character in a string is lowercase.",
        AttributeSet::from_attribute(Attribute::Protected),
        strings::eval_lower_case_q,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "UpperCaseQ",
        "Tests whether every character in a string is uppercase.",
        AttributeSet::from_attribute(Attribute::Protected),
        strings::eval_upper_case_q,
    ),
    BuiltinSpec::ordinary(
        "DigitQ",
        "Tests whether all characters in a string belong to the current digit subset.",
        strings::eval_digit_q,
    ),
    BuiltinSpec::ordinary(
        "StringContainsQ",
        "Tests whether a string or list of strings contains an exact substring in the current first-pass string subset.",
        strings::eval_string_contains_q,
    ),
    BuiltinSpec::ordinary(
        "StringStartsQ",
        "Tests whether a string or list of strings starts with an exact substring in the current first-pass string subset.",
        strings::eval_string_starts_q,
    ),
    BuiltinSpec::ordinary(
        "StringEndsQ",
        "Tests whether a string or list of strings ends with an exact substring in the current first-pass string subset.",
        strings::eval_string_ends_q,
    ),
    BuiltinSpec::ordinary(
        "StringPosition",
        "Returns 1-based inclusive positions of exact substring matches, including overlaps, in the current first-pass string subset.",
        strings::eval_string_position,
    ),
    BuiltinSpec::ordinary(
        "StringSplit",
        "Splits strings using whitespace or exact delimiters in the current first-pass string subset.",
        strings::eval_string_split,
    ),
    BuiltinSpec::ordinary(
        "StringTrim",
        "Trims whitespace or exact boundary substrings in the current first-pass string subset.",
        strings::eval_string_trim,
    ),
    BuiltinSpec::ordinary(
        "StringReplace",
        "Replaces exact string needles using immediate string rules in the current first-pass string subset.",
        strings::eval_string_replace,
    ),
    BuiltinSpec::ordinary(
        "StringCases",
        "Extracts non-overlapping exact string matches or immediate exact-string rule outputs in the current first-pass string subset.",
        strings::eval_string_cases,
    ),
    BuiltinSpec::ordinary(
        "StringApply",
        "Applies a list-oriented function to string code points and reassembles string output in the current first-pass subset.",
        strings::eval_string_apply,
    ),
    BuiltinSpec::ordinary(
        "StringDelete",
        "Deletes exact or current string-pattern matches from strings in the current first-pass subset.",
        strings::eval_string_delete,
    ),
    BuiltinSpec::ordinary(
        "StringTake",
        "Extracts a substring using integer counts or inclusive 1-based ranges.",
        strings::eval_string_take,
    ),
    BuiltinSpec::ordinary(
        "StringDrop",
        "Drops a substring using integer counts or inclusive 1-based ranges.",
        strings::eval_string_drop,
    ),
    BuiltinSpec::ordinary(
        "StringTakeDrop",
        "Returns both the taken and dropped substrings using the current StringTake/StringDrop spec subset.",
        strings::eval_string_take_drop,
    ),
    BuiltinSpec::ordinary(
        "StringPart",
        "Extracts individual characters by 1-based positions.",
        strings::eval_string_part,
    ),
    BuiltinSpec::ordinary(
        "StringCount",
        "Counts exact or current string-pattern matches, including the current IgnoreCase and Overlaps option subset.",
        strings::eval_string_count,
    ),
    BuiltinSpec::ordinary(
        "StringMatchQ",
        "Tests whether a whole string matches an exact or current string-pattern spec.",
        strings::eval_string_match_q,
    ),
    BuiltinSpec::ordinary(
        "StringFreeQ",
        "Tests whether a string is free of exact or current string-pattern matches.",
        strings::eval_string_free_q,
    ),
];
