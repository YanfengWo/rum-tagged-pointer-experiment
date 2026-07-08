use rum_core::Expr;
use rum_runtime::Session;

pub(crate) const UPPERCASE_LETTERS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
pub(crate) const LOWERCASE_LETTERS: &str = "abcdefghijklmnopqrstuvwxyz";
pub(crate) const DIGIT_CHARACTERS: &str = "0123456789";
pub(crate) const PUNCTUATION_CHARACTERS: &str = r##"!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;
pub(crate) const ALPHANUMERIC: &str =
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
pub(crate) const WHITESPACE_CHARACTERS: &str = " \t\n\r";

pub(crate) const CHARACTER_CLASS_CONSTANTS: &[(&str, &str)] = &[
    ("UppercaseLetters", UPPERCASE_LETTERS),
    ("LowercaseLetters", LOWERCASE_LETTERS),
    ("DigitCharacters", DIGIT_CHARACTERS),
    ("PunctuationCharacters", PUNCTUATION_CHARACTERS),
    ("Alphanumeric", ALPHANUMERIC),
    ("WhitespaceCharacters", WHITESPACE_CHARACTERS),
];

pub(crate) fn install(session: &mut Session) {
    for (name, value) in CHARACTER_CLASS_CONSTANTS {
        session.install_system_own_value(name, Expr::string(*value));
    }
}

pub(crate) fn is_punctuation_character(ch: char) -> bool {
    PUNCTUATION_CHARACTERS.contains(ch)
}
