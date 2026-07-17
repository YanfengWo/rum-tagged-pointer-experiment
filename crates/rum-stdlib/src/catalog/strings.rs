use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use super::message_templates::*;

use crate::strings;

const ATTR_PROTECTED: AttributeSet = AttributeSet::from_attribute(Attribute::Protected);

pub(crate) const STRING_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::with_attributes(
        "Alternatives",
        "Represents the current pattern alternative head.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "Blank",
        "Represents the current single-element pattern atom; in string patterns it matches one character.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "BlankNullSequence",
        "Represents the current zero-or-more sequence pattern atom; in string patterns it matches zero or more characters.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "BlankSequence",
        "Represents the current one-or-more sequence pattern atom; in string patterns it matches one or more characters.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "DigitCharacter",
        "Represents the current string-pattern atom for one ASCII decimal digit.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "EndOfLine",
        "Represents the current string-pattern anchor for the end of a string or a position before LF.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "EndOfString",
        "Represents the current string-pattern anchor for the end of a string.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "Except",
        "Represents the current pattern exclusion head.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "LetterCharacter",
        "Represents the current string-pattern atom for one alphabetic character.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "Pattern",
        "Represents the current named pattern-binding head.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "PunctuationCharacter",
        "Represents the current string-pattern atom for one ASCII punctuation character.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "RegularExpression",
        "Represents the current regular-expression string-pattern wrapper.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "StartOfLine",
        "Represents the current string-pattern anchor for the start of a string or a position after LF.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "StartOfString",
        "Represents the current string-pattern anchor for the start of a string.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "StringExpression",
        "Represents the current string-pattern sequence head produced by `~~` syntax.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "WhitespaceCharacter",
        "Represents the current string-pattern atom for one whitespace character.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "WordBoundary",
        "Represents the current string-pattern anchor at a transition between word and non-word text.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::with_attributes(
        "WordCharacter",
        "Represents the current string-pattern atom for one alphabetic, ASCII digit, or underscore character.",
        ATTR_PROTECTED,
    ),
    BuiltinSpec::ordinary(
        "StringJoin",
        "Concatenates strings or a list of strings into a single string.",
        strings::eval_string_join,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "StringLength",
        "Returns the Unicode code-point length of a string.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_string_length,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "StringReverse",
        "Reverses the Unicode code-point characters in a string.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_string_reverse,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToLowerCase",
        "Converts a string to lowercase using the current Unicode-aware subset.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_to_lower_case,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "ToUpperCase",
        "Converts a string or list of strings to uppercase using the current Unicode-aware subset.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected),
        strings::eval_to_upper_case,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Capitalize",
        "Uppercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_capitalize,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary_with_attributes(
        "Decapitalize",
        "Lowercases the first character of a string or each string in a list.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        strings::eval_decapitalize,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]),
    BuiltinSpec::ordinary(
        "StringForm",
        "Formats a template string eagerly, replacing `` and `n` placeholders with ToString-rendered arguments.",
        strings::eval_string_form,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "slot",
            template: "Reports a missing or invalid template slot.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringWrap",
        "Greedily word-wraps a string to a positive character width, separating output lines with LF. With \"PreserveNewlines\" -> True, wraps each LF-delimited line independently and preserves blank or whitespace-only lines verbatim; the default False collapses all whitespace as word separators.",
        strings::eval_string_wrap,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "width must be a positive integer",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringIndent",
        "Prefixes each non-blank line of a string with a string prefix or an exact number of spaces. With \"BlankLines\" -> True, blank and whitespace-only lines are prefixed too; the default False leaves them unchanged.",
        strings::eval_string_indent,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringDedent",
        "Removes the longest common leading whitespace from the non-blank lines of a string.",
        strings::eval_string_dedent,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringExpandTabs",
        "Expands tabs in a string to the next fixed-width column stop, defaulting to width 8.",
        strings::eval_string_expand_tabs,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argform",
            template: "tab width must be a positive integer",
        },
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AlignColumns",
        "Formats a list of string rows as left-aligned columns separated by \"Separator\" -> \"  \" by default; any string separator can be supplied.",
        strings::eval_align_columns,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "DecimalString",
        "Renders an exact number as a plain decimal string with exactly n fractional digits.",
        strings::eval_decimal_string,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "HtmlEscape",
        "Escapes the five HTML text characters &, <, >, double quote, and single quote.",
        strings::eval_html_escape,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "HtmlUnescape",
        "Unescapes the five named HTML entities and decimal or hexadecimal numeric character references, leaving unknown and malformed entities verbatim by default; with \"Strict\" -> True, malformed entities fail closed with $Failed.",
        strings::eval_html_unescape,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "entity",
            template: "Reports an invalid HTML entity.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ShellSplit",
        "Splits a POSIX-style shell word string with whitespace, quotes, and backslash escapes, returning $Failed on malformed quoting.",
        strings::eval_shell_split,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "syntax",
            template: "Reports invalid input syntax.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ShellQuote",
        "Single-quotes a string for POSIX-style shell input, escaping embedded single quotes.",
        strings::eval_shell_quote,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FileExtension",
        "Returns the final path component extension without the dot, or an empty string when no extension is present.",
        strings::eval_file_extension,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FileBaseName",
        "Returns the final path component without its extension.",
        strings::eval_file_base_name,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "FileNameSplit",
        "Splits a slash-separated path string into components, preserving an absolute leading slash as its own component.",
        strings::eval_file_name_split,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "UrlEncode",
        "Percent-encodes a string using UTF-8 bytes and the RFC 3986 unreserved character set; with \"FormEncoding\" -> True, spaces are encoded as + for form query data.",
        strings::eval_url_encode,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "UrlDecode",
        "Decodes percent escapes in a UTF-8 URL string, returning $Failed for invalid escapes or invalid UTF-8; with \"FormEncoding\" -> True, + decodes to a space.",
        strings::eval_url_decode,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "escape",
            template: "Reports an invalid escape sequence.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "UrlParse",
        "Parses scheme://host[:port][/path][?query][#fragment] into an association of string fields; \"DefaultScheme\" can supply a scheme for schemeless inputs, and \"DecodeQuery\" decodes query text using form encoding.",
        strings::eval_url_parse,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "escape",
            template: "Reports an invalid escape sequence.",
        },
        BuiltinMessageSpec {
            tag: "syntax",
            template: "Reports invalid input syntax.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "ParseArgs",
        "Parses script arguments into positional arguments, --key=value options, and --flag names; with \"ShortOptions\" -> True, single-dash clusters become single-letter flags, without attached-value syntax.",
        strings::eval_parse_args,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Row",
        "Renders a list of expressions as a single text string in the current harness subset.",
        strings::eval_row,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringRiffle",
        "Riffles a flat string list with a string separator or {left, separator, right} wrapper, and riffles nested string rows with row and item separators.",
        strings::eval_string_riffle,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringPadLeft",
        "Pads strings on the left to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width.",
        strings::eval_string_pad_left,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringPadRight",
        "Pads strings on the right to a target width using a string pad or a non-empty list of non-empty string pads; list pads cycle from the first emitted pad character and the final pad piece is truncated to the target character width. Values at or above width are unchanged.",
        strings::eval_string_pad_right,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringLines",
        "Splits a string on LF or CRLF newlines, preserving interior empty lines.",
        strings::eval_string_lines,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringTemplate",
        "Builds a simple TemplateObject wrapper from a template string in the current first-pass subset.",
        strings::eval_string_template,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "TemplateApply",
        "Applies a simple named or positional string template in the current first-pass subset; unfilled slots leave the call inert by default, while \"Strict\" -> True reports them and returns $Failed.",
        strings::eval_template_apply,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "slot",
            template: "Reports a missing or invalid template slot.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "TextWords",
        "Splits text into alphanumeric word runs in the current first-pass subset.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_text_words,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "ToCharacterCode",
        "Returns Unicode scalar values for strings, or UTF-8 byte values when given the \"UTF8\" or \"UTF-8\" encoding argument.",
        AttributeSet::from_attribute(Attribute::Listable),
        strings::eval_to_character_code,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "ToExpression",
        "Parses and evaluates input strings using the current InputForm/StandardForm-compatible subset.",
        strings::eval_to_expression,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "StringToByteArray",
        "Converts strings to ByteArray objects using UTF8/UTF-8 encoding.",
        strings::eval_string_to_byte_array,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    #[cfg(feature = "hash")]
    BuiltinSpec::ordinary(
        "Hash",
        "Computes SHA256, SHA1, SHA512, MD5, or CRC32 hashes over the current string/expression subset.",
        strings::eval_hash,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Compress",
        "Encodes expressions into the current project-local compressed string envelope.",
        strings::eval_compress,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Uncompress",
        "Recovers values from the current project-local compressed string envelope; malformed envelopes stay inert by default, while \"Strict\" -> True reports them and returns $Failed.",
        strings::eval_uncompress,
    )
    .with_messages(&[
        ARGTU,
        BuiltinMessageSpec {
            tag: "env",
            template: "Uncompress found a malformed or corrupt compressed string envelope.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Nearest",
        "Finds nearest strings or exact numeric values in the current first-pass subset.",
        strings::eval_nearest,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Pluralize",
        "Applies a simple English pluralization subset.",
        strings::eval_pluralize,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SyntaxQ",
        "Tests whether a string parses successfully in the current syntax subset.",
        strings::eval_syntax_q,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RumTokens",
        "Tokenizes rum source text into total, trivia-preserving lexer spans.",
        strings::eval_rum_tokens,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "tok",
            template: "Reports a tokenizer failure.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TokenizeWith",
        "Scans a string with string-pattern rules and returns total 1-based token spans.",
        strings::eval_tokenize_with,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "tok",
            template: "Reports a tokenizer failure.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "TextString",
        "Converts expressions to the current canonical text subset, leaving raw strings unquoted.",
        strings::eval_text_string,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "CharacterCounts",
        "Counts code-point occurrences in a string and returns an association in the current first-pass subset.",
        strings::eval_character_counts,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "RemoveDiacritics",
        "Removes common Latin diacritics from strings in the current first-pass subset.",
        strings::eval_remove_diacritics,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Alphabet",
        "Returns the current English lowercase alphabet subset.",
        strings::eval_alphabet,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "AlphabeticSort",
        "Sorts strings using the current English alphabet subset with simple diacritic folding.",
        strings::eval_alphabetic_sort,
    )
    .with_messages(&[
        ARGTU,
    ]).protected(),
    BuiltinSpec::ordinary(
        "EditDistance",
        "Computes unit-cost Levenshtein distance for strings or lists in the current first-pass subset.",
        strings::eval_edit_distance,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "HammingDistance",
        "Computes mismatch counts for equal-length strings or lists in the current first-pass subset; unequal lengths emit a message and leave the call inert.",
        strings::eval_hamming_distance,
    )
    .with_messages(&[
        ARGX_EXACT,
        BuiltinMessageSpec {
            tag: "len",
            template: "HammingDistance expects inputs with the same length in `1`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "LongestCommonSubsequence",
        "Finds the longest common contiguous substring or sublist in the current first-pass subset.",
        strings::eval_longest_common_subsequence,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "SequenceAlignment",
        "Builds a simple global alignment summary for strings or lists in the current first-pass subset.",
        strings::eval_sequence_alignment,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Diff",
        "Builds a simple DiffObject for strings or lists in the current first-pass subset.",
        strings::eval_diff,
    )
    .with_messages(&[
        ARGX_EXACT,
    ]).protected(),
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
