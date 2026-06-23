use crate::error::SyntaxError;
use crate::span::Span;
use crate::trivia::{LexedSource, SyntaxToken, SyntaxTokenKind, TriviaPiece};

pub(crate) type Token = SyntaxToken;
pub(crate) type TokenKind = SyntaxTokenKind;

const NAMED_CHARACTERS: &[(&str, char)] = &[
    ("Alpha", '\u{3b1}'),
    ("Beta", '\u{3b2}'),
    ("Gamma", '\u{3b3}'),
    ("Delta", '\u{3b4}'),
    ("Epsilon", '\u{3f5}'),
    ("Zeta", '\u{3b6}'),
    ("Eta", '\u{3b7}'),
    ("Theta", '\u{3b8}'),
    ("Iota", '\u{3b9}'),
    ("Kappa", '\u{3ba}'),
    ("Lambda", '\u{3bb}'),
    ("Mu", '\u{3bc}'),
    ("Nu", '\u{3bd}'),
    ("Xi", '\u{3be}'),
    ("Omicron", '\u{3bf}'),
    ("Pi", '\u{3c0}'),
    ("Rho", '\u{3c1}'),
    ("Sigma", '\u{3c3}'),
    ("Tau", '\u{3c4}'),
    ("Upsilon", '\u{3c5}'),
    ("Phi", '\u{3d5}'),
    ("Chi", '\u{3c7}'),
    ("Psi", '\u{3c8}'),
    ("Omega", '\u{3c9}'),
    ("CurlyEpsilon", '\u{3b5}'),
    ("CurlyTheta", '\u{3d1}'),
    ("CurlyKappa", '\u{3f0}'),
    ("CurlyPi", '\u{3d6}'),
    ("CurlyRho", '\u{3f1}'),
    ("CurlyPhi", '\u{3c6}'),
    ("FinalSigma", '\u{3c2}'),
    ("CapitalAlpha", '\u{391}'),
    ("CapitalBeta", '\u{392}'),
    ("CapitalGamma", '\u{393}'),
    ("CapitalDelta", '\u{394}'),
    ("CapitalEpsilon", '\u{395}'),
    ("CapitalZeta", '\u{396}'),
    ("CapitalEta", '\u{397}'),
    ("CapitalTheta", '\u{398}'),
    ("CapitalIota", '\u{399}'),
    ("CapitalKappa", '\u{39a}'),
    ("CapitalLambda", '\u{39b}'),
    ("CapitalMu", '\u{39c}'),
    ("CapitalNu", '\u{39d}'),
    ("CapitalXi", '\u{39e}'),
    ("CapitalOmicron", '\u{39f}'),
    ("CapitalPi", '\u{3a0}'),
    ("CapitalRho", '\u{3a1}'),
    ("CapitalSigma", '\u{3a3}'),
    ("CapitalTau", '\u{3a4}'),
    ("CapitalUpsilon", '\u{3a5}'),
    ("CapitalPhi", '\u{3a6}'),
    ("CapitalChi", '\u{3a7}'),
    ("CapitalPsi", '\u{3a8}'),
    ("CapitalOmega", '\u{3a9}'),
    ("And", '\u{2227}'),
    ("Or", '\u{2228}'),
    ("Not", '\u{ac}'),
    ("Xor", '\u{22bb}'),
    ("Nand", '\u{22bc}'),
    ("Nor", '\u{22bd}'),
    ("ForAll", '\u{2200}'),
    ("Exists", '\u{2203}'),
    ("NotExists", '\u{2204}'),
    ("Implies", '\u{f523}'),
    ("Equivalent", '\u{29e6}'),
    ("RightTee", '\u{22a2}'),
    ("LeftTee", '\u{22a3}'),
    ("UpTee", '\u{22a5}'),
    ("DownTee", '\u{22a4}'),
    ("DoubleRightTee", '\u{22a8}'),
    ("Wedge", '\u{22c0}'),
    ("Vee", '\u{22c1}'),
    ("Equal", '\u{f431}'),
    ("NotEqual", '\u{2260}'),
    ("LessEqual", '\u{2264}'),
    ("GreaterEqual", '\u{2265}'),
    ("LongEqual", '\u{f7d9}'),
    ("Tilde", '\u{223c}'),
    ("TildeTilde", '\u{2248}'),
    ("TildeEqual", '\u{2243}'),
    ("TildeFullEqual", '\u{2245}'),
    ("Congruent", '\u{2261}'),
    ("Proportional", '\u{221d}'),
    ("Element", '\u{2208}'),
    ("NotElement", '\u{2209}'),
    ("ReverseElement", '\u{220b}'),
    ("Subset", '\u{2282}'),
    ("Superset", '\u{2283}'),
    ("SubsetEqual", '\u{2286}'),
    ("SupersetEqual", '\u{2287}'),
    ("Union", '\u{22c3}'),
    ("Intersection", '\u{22c2}'),
    ("EmptySet", '\u{2205}'),
    ("Infinity", '\u{221e}'),
    ("PartialD", '\u{2202}'),
    ("Del", '\u{2207}'),
    ("Integral", '\u{222b}'),
    ("ContourIntegral", '\u{222e}'),
    ("Sum", '\u{2211}'),
    ("Product", '\u{220f}'),
    ("CirclePlus", '\u{2295}'),
    ("CircleTimes", '\u{2297}'),
    ("CircleDot", '\u{2299}'),
    ("Cross", '\u{f4a0}'),
    ("Times", '\u{d7}'),
    ("Divide", '\u{f7}'),
    ("PlusMinus", '\u{b1}'),
    ("MinusPlus", '\u{2213}'),
    ("CenterDot", '\u{b7}'),
    ("Star", '\u{22c6}'),
    ("Bullet", '\u{2022}'),
    ("Diamond", '\u{22c4}'),
    ("SmallCircle", '\u{2218}'),
    ("Degree", '\u{b0}'),
    ("Angle", '\u{2220}'),
    ("Perpendicular", '\u{27c2}'),
    ("RightArrow", '\u{2192}'),
    ("LeftArrow", '\u{2190}'),
    ("LeftRightArrow", '\u{2194}'),
    ("UpArrow", '\u{2191}'),
    ("DownArrow", '\u{2193}'),
    ("DoubleRightArrow", '\u{21d2}'),
    ("DoubleLeftArrow", '\u{21d0}'),
    ("DoubleLeftRightArrow", '\u{21d4}'),
    ("Rule", '\u{f522}'),
    ("RuleDelayed", '\u{f51f}'),
    ("Function", '\u{f4a1}'),
];

pub fn named_character(name: &str) -> Option<char> {
    NAMED_CHARACTERS
        .iter()
        .find_map(|(candidate, ch)| (*candidate == name).then_some(*ch))
}

pub fn lex_with_trivia(input: &str) -> Result<LexedSource, SyntaxError> {
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();
    let mut trivia = Vec::new();

    while let Some(token) = lexer.next_token(&mut trivia)? {
        let is_end = matches!(token.kind, TokenKind::End);
        tokens.push(token);

        if is_end {
            break;
        }
    }

    Ok(LexedSource::new(tokens, trivia))
}

pub(crate) fn lex(input: &str) -> Result<Vec<Token>, SyntaxError> {
    Ok(lex_with_trivia(input)?.into_tokens())
}

struct Lexer<'a> {
    input: &'a str,
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, offset: 0 }
    }

    fn next_token(&mut self, trivia: &mut Vec<TriviaPiece>) -> Result<Option<Token>, SyntaxError> {
        let leading_trivia = self.collect_trivia(trivia)?;

        if self.is_eof() {
            return Ok(Some(Token::new(
                TokenKind::End,
                Span::point(self.offset),
                leading_trivia,
            )));
        }

        let start = self.offset;
        let ch = self.peek_char().expect("checked eof above");

        let token = match ch {
            '[' => {
                if self.consume_str("[[") {
                    TokenKind::DoubleLBracket
                } else {
                    self.single(TokenKind::LBracket)
                }
            }
            ']' => self.single(TokenKind::RBracket),
            '{' => self.single(TokenKind::LBrace),
            '}' => self.single(TokenKind::RBrace),
            '<' => {
                if self.consume_str("<|") {
                    TokenKind::LAssociation
                } else if self.consume_str("<=") {
                    TokenKind::LessEqual
                } else if self.consume_str("<>") {
                    TokenKind::StringJoin
                } else {
                    self.single(TokenKind::Less)
                }
            }
            '>' => {
                if self.consume_str(">=") {
                    TokenKind::GreaterEqual
                } else {
                    self.single(TokenKind::Greater)
                }
            }
            '|' => {
                if self.consume_str("|->") {
                    TokenKind::BarMinusGreater
                } else if self.consume_str("|>") {
                    TokenKind::RAssociation
                } else if self.consume_str("||") {
                    TokenKind::Or
                } else {
                    self.single(TokenKind::Bar)
                }
            }
            '(' => self.single(TokenKind::LParen),
            ')' => self.single(TokenKind::RParen),
            ',' => self.single(TokenKind::Comma),
            ';' => {
                if self.consume_str(";;") {
                    TokenKind::Span
                } else {
                    self.single(TokenKind::Semicolon)
                }
            }
            '@' => {
                if self.consume_str("@@@") {
                    TokenKind::AtAtAt
                } else if self.consume_str("@@") {
                    TokenKind::AtAt
                } else if self.consume_str("@*") {
                    TokenKind::AtStar
                } else {
                    self.single(TokenKind::At)
                }
            }
            '+' => {
                if self.consume_str("++") {
                    TokenKind::PlusPlus
                } else if self.consume_str("+=") {
                    TokenKind::PlusEqual
                } else {
                    self.single(TokenKind::Plus)
                }
            }
            '*' => {
                if self.consume_str("**") {
                    TokenKind::StarStar
                } else if self.consume_str("*=") {
                    TokenKind::StarEqual
                } else {
                    self.single(TokenKind::Star)
                }
            }
            '^' => {
                if self.consume_str("^:=") {
                    TokenKind::CaretColonEqual
                } else if self.consume_str("^=") {
                    TokenKind::CaretEqual
                } else {
                    self.single(TokenKind::Caret)
                }
            }
            '&' => {
                if self.consume_str("&&") {
                    TokenKind::And
                } else {
                    self.single(TokenKind::Ampersand)
                }
            }
            '!' => {
                if self.consume_str("!=") {
                    TokenKind::Unequal
                } else {
                    self.single(TokenKind::Bang)
                }
            }
            '\'' => self.single(TokenKind::Prime),
            '.' => self.lex_dot_prefixed_token(),
            '?' => self.single(TokenKind::Question),
            '#' => self.single(TokenKind::Hash),
            '-' => {
                if self.consume_str("->") {
                    TokenKind::Arrow
                } else if self.consume_str("--") {
                    TokenKind::MinusMinus
                } else if self.consume_str("-=") {
                    TokenKind::MinusEqual
                } else {
                    self.single(TokenKind::Minus)
                }
            }
            ':' => {
                if self.consume_str("::") {
                    TokenKind::MessageName
                } else if self.consume_str(":=") {
                    TokenKind::SetDelayed
                } else if self.consume_str(":>") {
                    TokenKind::RuleDelayed
                } else {
                    self.single(TokenKind::Colon)
                }
            }
            '=' => {
                if self.consume_str("=!=") {
                    TokenKind::Unsame
                } else if self.consume_str("===") {
                    TokenKind::Same
                } else if self.consume_str("==") {
                    TokenKind::Equal
                } else if self.consume_str("=.") {
                    TokenKind::Unset
                } else {
                    self.single(TokenKind::Set)
                }
            }
            '/' => {
                if self.consume_str("//.") {
                    TokenKind::ReplaceRepeated
                } else if self.consume_str("//@") {
                    TokenKind::SlashSlashAt
                } else if self.consume_str("/@") {
                    TokenKind::SlashAt
                } else if self.consume_str("//=") {
                    TokenKind::SlashSlashEqual
                } else if self.consume_str("//") {
                    TokenKind::SlashSlash
                } else if self.consume_str("/=") {
                    TokenKind::SlashEqual
                } else if self.consume_str("/.") {
                    TokenKind::ReplaceAll
                } else if self.consume_str("/:") {
                    TokenKind::TagSet
                } else if self.consume_str("/;") {
                    TokenKind::Condition
                } else if self.consume_str("/*") {
                    TokenKind::SlashStar
                } else {
                    self.single(TokenKind::Slash)
                }
            }
            '_' => {
                if self.consume_str("___") {
                    TokenKind::TripleUnderscore
                } else if self.consume_str("__") {
                    TokenKind::DoubleUnderscore
                } else {
                    self.single(TokenKind::Underscore)
                }
            }
            '"' => self.lex_string()?,
            '\\' => self.lex_named_character_token(start)?,
            ch if ch.is_ascii_digit() => self.lex_number(),
            ch if unicode_operator_token_kind(ch).is_some() => {
                self.bump_char();
                unicode_operator_token_kind(ch).expect("checked unicode operator above")
            }
            ch if is_symbol_start(ch) => self.lex_symbol()?,
            other => {
                return Err(SyntaxError::UnexpectedCharacter {
                    found: other,
                    span: Span::new(start, start + other.len_utf8()),
                });
            }
        };

        Ok(Some(Token::new(
            token,
            Span::new(start, self.offset),
            leading_trivia,
        )))
    }

    fn lex_number(&mut self) -> TokenKind {
        let start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }

        let has_decimal_point = self.peek_char() == Some('.') && self.peek_next_char() != Some('.');
        if has_decimal_point {
            self.bump_char();
            while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
                self.bump_char();
            }
            self.consume_decimal_exponent();
        }

        TokenKind::Integer(self.input[start..self.offset].to_string())
    }

    fn lex_dot_prefixed_token(&mut self) -> TokenKind {
        if self.peek_next_char().is_some_and(|ch| ch.is_ascii_digit()) {
            let start = self.offset;
            self.bump_char();
            while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
                self.bump_char();
            }
            self.consume_decimal_exponent();
            return TokenKind::Integer(self.input[start..self.offset].to_string());
        }

        if self.consume_str("...") {
            TokenKind::RepeatedNull
        } else if self.consume_str("..") {
            TokenKind::Repeated
        } else {
            self.single(TokenKind::Dot)
        }
    }

    fn consume_decimal_exponent(&mut self) {
        let Some(exponent_marker) = self.peek_char() else {
            return;
        };
        if !matches!(exponent_marker, 'e' | 'E') {
            return;
        }

        let mut scan = self.offset + exponent_marker.len_utf8();
        if let Some(sign) = self.input[scan..].chars().next() {
            if matches!(sign, '+' | '-') {
                scan += sign.len_utf8();
            }
        }

        let digits_start = scan;
        while let Some(ch) = self.input[scan..].chars().next() {
            if !ch.is_ascii_digit() {
                break;
            }
            scan += ch.len_utf8();
        }

        if scan > digits_start {
            self.offset = scan;
        }
    }

    fn lex_symbol(&mut self) -> Result<TokenKind, SyntaxError> {
        let mut value = String::new();
        value.push(self.bump_char().expect("symbol start"));
        self.lex_symbol_continuation(value)
    }

    fn lex_symbol_continuation(&mut self, mut value: String) -> Result<TokenKind, SyntaxError> {
        while let Some(ch) = self.peek_char() {
            if is_symbol_continue(ch) {
                value.push(ch);
                self.bump_char();
                continue;
            }

            if self.input[self.offset..].starts_with("\\[") {
                let Some((decoded, end)) = self.named_character_escape_at(self.offset)? else {
                    unreachable!("starts_with checked above");
                };

                if unicode_operator_token_kind(decoded).is_none() {
                    value.push(decoded);
                    self.offset = end;
                    continue;
                }
            }

            break;
        }

        Ok(TokenKind::Symbol(value))
    }

    fn lex_string(&mut self) -> Result<TokenKind, SyntaxError> {
        let start = self.offset;
        self.bump_char();
        let mut value = String::new();

        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    self.bump_char();
                    return Ok(TokenKind::String(value));
                }
                '\\' => {
                    self.bump_char();

                    let escaped = self.peek_char().ok_or(SyntaxError::UnterminatedString {
                        span: Span::new(start, self.offset),
                    })?;

                    match escaped {
                        '"' | '\\' => {
                            value.push(escaped);
                            self.bump_char();
                        }
                        '[' => {
                            let escape_start = self.offset - 1;
                            let Some((decoded, end)) =
                                self.named_character_escape_at(escape_start)?
                            else {
                                unreachable!("string escape branch checked '['");
                            };
                            value.push(decoded);
                            self.offset = end;
                        }
                        'n' => {
                            value.push('\n');
                            self.bump_char();
                        }
                        't' => {
                            value.push('\t');
                            self.bump_char();
                        }
                        'r' => {
                            value.push('\r');
                            self.bump_char();
                        }
                        'u' => {
                            if let Some(unicode) = self.try_lex_unicode_escape() {
                                value.push(unicode);
                            } else {
                                value.push('u');
                                self.bump_char();
                            }
                        }
                        other => {
                            value.push(other);
                            self.bump_char();
                        }
                    }
                }
                other => {
                    value.push(other);
                    self.bump_char();
                }
            }
        }

        Err(SyntaxError::UnterminatedString {
            span: Span::new(start, self.offset),
        })
    }

    fn lex_named_character_token(&mut self, start: usize) -> Result<TokenKind, SyntaxError> {
        let Some((decoded, end)) = self.named_character_escape_at(start)? else {
            return Err(SyntaxError::UnexpectedCharacter {
                found: '\\',
                span: Span::new(start, start + 1),
            });
        };

        self.offset = end;

        if let Some(kind) = unicode_operator_token_kind(decoded) {
            return Ok(kind);
        }

        let mut value = String::new();
        value.push(decoded);
        self.lex_symbol_continuation(value)
    }

    fn named_character_escape_at(
        &self,
        start: usize,
    ) -> Result<Option<(char, usize)>, SyntaxError> {
        if !self.input[start..].starts_with("\\[") {
            return Ok(None);
        }

        let name_start = start + 2;
        let mut scan = name_start;

        while scan < self.input.len() {
            let ch = self.input[scan..].chars().next().expect("scan in bounds");
            if ch == ']' {
                let end = scan + 1;
                let name = &self.input[name_start..scan];
                let Some(decoded) = named_character(name) else {
                    return Err(SyntaxError::UnexpectedCharacter {
                        found: '\\',
                        span: Span::new(start, end),
                    });
                };
                return Ok(Some((decoded, end)));
            }
            scan += ch.len_utf8();
        }

        Err(SyntaxError::UnexpectedCharacter {
            found: '\\',
            span: Span::new(start, self.input.len()),
        })
    }

    fn try_lex_unicode_escape(&mut self) -> Option<char> {
        if !self.input[self.offset..].starts_with("u{") {
            return None;
        }

        let original_offset = self.offset;
        self.offset += 2;
        let digits_start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_hexdigit()) {
            self.bump_char();
        }

        if self.offset == digits_start || self.peek_char() != Some('}') {
            self.offset = original_offset;
            return None;
        }

        let digits = &self.input[digits_start..self.offset];
        self.bump_char();

        let value = u32::from_str_radix(digits, 16).ok()?;
        char::from_u32(value)
    }

    fn single(&mut self, kind: TokenKind) -> TokenKind {
        self.bump_char();
        kind
    }

    fn consume_str(&mut self, expected: &str) -> bool {
        if self.input[self.offset..].starts_with(expected) {
            self.offset += expected.len();
            true
        } else {
            false
        }
    }

    fn collect_trivia(
        &mut self,
        trivia: &mut Vec<TriviaPiece>,
    ) -> Result<std::ops::Range<usize>, SyntaxError> {
        let start = trivia.len();

        loop {
            if self.offset == 0 && self.input.starts_with("#!") {
                trivia.push(self.collect_shebang_comment());
                continue;
            }

            let whitespace_start = self.offset;

            while matches!(self.peek_char(), Some(ch) if ch.is_whitespace()) {
                self.bump_char();
            }

            if self.offset > whitespace_start {
                let span = Span::new(whitespace_start, self.offset);
                trivia.push(TriviaPiece::whitespace(
                    span,
                    self.input[whitespace_start..self.offset].to_string(),
                ));
            }

            if self.input[self.offset..].starts_with("(*") {
                trivia.push(self.collect_comment()?);
                continue;
            }

            return Ok(start..trivia.len());
        }
    }

    fn collect_shebang_comment(&mut self) -> TriviaPiece {
        let start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch != '\n' && ch != '\r') {
            self.bump_char();
        }

        let span = Span::new(start, self.offset);
        TriviaPiece::comment(span, self.input[start..self.offset].to_string())
    }

    fn collect_comment(&mut self) -> Result<TriviaPiece, SyntaxError> {
        let start = self.offset;
        self.offset += 2;
        let mut depth = 1usize;

        while !self.is_eof() {
            if self.input[self.offset..].starts_with("(*") {
                self.offset += 2;
                depth += 1;
                continue;
            }

            if self.input[self.offset..].starts_with("*)") {
                self.offset += 2;
                depth -= 1;

                if depth == 0 {
                    let span = Span::new(start, self.offset);
                    return Ok(TriviaPiece::comment(
                        span,
                        self.input[start..self.offset].to_string(),
                    ));
                }

                continue;
            }

            self.bump_char();
        }

        Err(SyntaxError::UnterminatedComment {
            span: Span::new(start, self.offset),
        })
    }

    fn peek_char(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn peek_next_char(&self) -> Option<char> {
        let mut chars = self.input[self.offset..].chars();
        chars.next()?;
        chars.next()
    }

    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.input.len()
    }
}

fn is_symbol_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '$' || is_named_character_symbol_char(ch)
}

fn is_symbol_continue(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '$' | '`') || is_named_character_symbol_char(ch)
}

fn is_named_character_symbol_char(ch: char) -> bool {
    unicode_operator_token_kind(ch).is_none()
        && NAMED_CHARACTERS.iter().any(|(_, named_ch)| *named_ch == ch)
}

fn unicode_operator_token_kind(ch: char) -> Option<TokenKind> {
    match ch {
        '\u{2227}' => Some(TokenKind::And),
        '\u{2228}' => Some(TokenKind::Or),
        '\u{ac}' => Some(TokenKind::Bang),
        '\u{f431}' => Some(TokenKind::Equal),
        '\u{2260}' => Some(TokenKind::Unequal),
        '\u{2264}' => Some(TokenKind::LessEqual),
        '\u{2265}' => Some(TokenKind::GreaterEqual),
        '\u{d7}' => Some(TokenKind::Star),
        '\u{f7}' => Some(TokenKind::Slash),
        '\u{f522}' => Some(TokenKind::Arrow),
        '\u{f51f}' => Some(TokenKind::RuleDelayed),
        '\u{f4a1}' => Some(TokenKind::Ampersand),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TriviaKind;

    #[test]
    fn lexes_multi_char_operators() {
        let tokens = lex(
            "x::tag := y //. z /. q -> w /; p f /: g =. a[[1]] b;;c <|d -> e|> h@i j@@k l@@@m n@*o p/*q r/@s aa^=bb cc^:=dd ee**ff gg//@hh ii|->jj a&&b c||d !e f===g h=!=i j==k l!=m n<o p>q r<=s t>=u v:_ \"a\"<>\"b\"",
        )
        .unwrap();

        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::MessageName))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::SetDelayed))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::ReplaceRepeated))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::ReplaceAll))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Arrow))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Condition))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::TagSet))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Unset))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::DoubleLBracket))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Span))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::LAssociation))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::RAssociation))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::At))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::AtAt))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::AtAtAt))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::AtStar))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::SlashStar))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::SlashAt))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::CaretEqual))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::CaretColonEqual))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::StarStar))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::SlashSlashAt))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::BarMinusGreater))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::And))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Or))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Bang))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Same))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Unsame))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Equal))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Unequal))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Less))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Greater))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::LessEqual))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::GreaterEqual))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::Colon))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::StringJoin))
        );
    }

    #[test]
    fn lexes_context_qualified_symbol() {
        let tokens = lex("System`List").unwrap();
        assert!(matches!(
            &tokens[0].kind,
            TokenKind::Symbol(value) if value == "System`List"
        ));
    }

    #[test]
    fn lexes_unicode_letter_symbols_used_by_wolfram_source() {
        let source = "Module[{\u{03C3} = <||>, \u{03B1}Ty = tyVar[\"A\"]}, \u{03C3}[\u{03B1}Ty]]";
        let tokens = lex(source).unwrap();
        let sigma = '\u{03C3}'.to_string();
        let alpha_ty = format!("{}Ty", '\u{03B1}');

        assert!(
            tokens
                .iter()
                .any(|token| matches!(&token.kind, TokenKind::Symbol(value) if value == &sigma))
        );
        assert!(
            tokens
                .iter()
                .any(|token| matches!(&token.kind, TokenKind::Symbol(value) if value == &alpha_ty))
        );
    }

    #[test]
    fn decodes_named_character_escapes_in_strings() {
        let tokens = lex(r#""\[Exists]\[ForAll]\[And]\[Or]\[Not]\[DoubleRightArrow]""#).unwrap();

        assert!(matches!(
            &tokens[0].kind,
            TokenKind::String(value) if value == "∃∀∧∨¬⇒" && value.chars().count() == 6
        ));
    }

    #[test]
    fn decodes_named_character_escapes_in_code_symbols() {
        let tokens = lex(r"\[Alpha] x\[Beta] \[CapitalOmega]1 \[Exists]").unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "α"));
        assert!(matches!(&tokens[1].kind, TokenKind::Symbol(value) if value == "xβ"));
        assert!(matches!(&tokens[2].kind, TokenKind::Symbol(value) if value == "Ω1"));
        assert!(matches!(&tokens[3].kind, TokenKind::Symbol(value) if value == "∃"));
    }

    #[test]
    fn decodes_named_character_operator_with_source_span() {
        let tokens = lex(r"\[Alpha] \[And] \[Beta]").unwrap();

        assert_eq!(tokens[0].span, Span::new(0, 8));
        assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "α"));
        assert_eq!(tokens[1].span, Span::new(9, 15));
        assert!(matches!(tokens[1].kind, TokenKind::And));
        assert_eq!(tokens[2].span, Span::new(16, 23));
        assert!(matches!(&tokens[2].kind, TokenKind::Symbol(value) if value == "β"));
    }

    #[test]
    fn reports_unknown_named_character_escapes() {
        let err = lex(r#""\[NotARealName]""#).unwrap_err();
        assert_eq!(
            err,
            SyntaxError::UnexpectedCharacter {
                found: '\\',
                span: Span::new(1, 16),
            }
        );

        let err = lex(r"\[NotARealName]").unwrap_err();
        assert_eq!(
            err,
            SyntaxError::UnexpectedCharacter {
                found: '\\',
                span: Span::new(0, 15),
            }
        );
    }

    #[test]
    fn skips_comments_and_nested_comments_as_trivia() {
        let tokens = lex("f[(* outer (* inner *) *)x, y]").unwrap();

        assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "f"));
        assert!(matches!(&tokens[2].kind, TokenKind::Symbol(value) if value == "x"));
        assert!(matches!(&tokens[4].kind, TokenKind::Symbol(value) if value == "y"));
    }

    #[test]
    fn treats_initial_shebang_as_script_comment_trivia() {
        let source = "#!/usr/bin/env wolframscript\nPrint[1]";
        let lexed = lex_with_trivia(source).unwrap();
        let trivia = lexed.trivia();

        assert_eq!(trivia[0].kind, TriviaKind::Comment);
        assert_eq!(trivia[0].text, "#!/usr/bin/env wolframscript");

        let tokens = lex(source).unwrap();
        assert!(matches!(&tokens[0].kind, TokenKind::Symbol(value) if value == "Print"));
    }

    #[test]
    fn reports_unterminated_nested_comment() {
        let err = lex("f[(* outer (* inner *)").unwrap_err();

        assert_eq!(
            err,
            SyntaxError::UnterminatedComment {
                span: Span::new(2, 22),
            }
        );
    }

    #[test]
    fn reports_unexpected_character_byte_span() {
        let err = lex("\\").unwrap_err();
        assert_eq!(
            err,
            SyntaxError::UnexpectedCharacter {
                found: '\\',
                span: Span::new(0, 1),
            }
        );

        let err = lex("\u{2603}").unwrap_err();
        assert_eq!(
            err,
            SyntaxError::UnexpectedCharacter {
                found: '\u{2603}',
                span: Span::new(0, 3),
            }
        );
    }

    #[test]
    fn lexes_replace_repeated_before_replace_all_prefix() {
        let tokens = lex("expr //. rule").unwrap();

        assert!(matches!(tokens[1].kind, TokenKind::ReplaceRepeated));
    }

    #[test]
    fn lexes_specific_slash_operators_before_postfix_apply() {
        let tokens = lex("a //. b a //= b a /. b a // b").unwrap();

        assert!(matches!(tokens[1].kind, TokenKind::ReplaceRepeated));
        assert!(matches!(tokens[4].kind, TokenKind::SlashSlashEqual));
        assert!(matches!(tokens[7].kind, TokenKind::ReplaceAll));
        assert!(matches!(tokens[10].kind, TokenKind::SlashSlash));
    }

    #[test]
    fn lexes_postfix_operator_tokens_without_splitting_decimal_literals() {
        let tokens = lex("a .. b ... f' 1.5 .5 1. 1.. 1... 1.5e3").unwrap();

        assert!(matches!(tokens[1].kind, TokenKind::Repeated));
        assert!(matches!(tokens[3].kind, TokenKind::RepeatedNull));
        assert!(matches!(tokens[5].kind, TokenKind::Prime));
        assert!(matches!(&tokens[6].kind, TokenKind::Integer(value) if value == "1.5"));
        assert!(matches!(&tokens[7].kind, TokenKind::Integer(value) if value == ".5"));
        assert!(matches!(&tokens[8].kind, TokenKind::Integer(value) if value == "1."));
        assert!(matches!(&tokens[9].kind, TokenKind::Integer(value) if value == "1"));
        assert!(matches!(tokens[10].kind, TokenKind::Repeated));
        assert!(matches!(&tokens[11].kind, TokenKind::Integer(value) if value == "1"));
        assert!(matches!(tokens[12].kind, TokenKind::RepeatedNull));
        assert!(matches!(&tokens[13].kind, TokenKind::Integer(value) if value == "1.5e3"));
    }

    #[test]
    fn lexes_new_infix_operators_before_shared_prefixes() {
        let tokens = lex(
            "a ^:= b a ^= b a ^ b a ** b a *= b a * b f //@ x f /@ x a // f x |-> y a || b a | b",
        )
        .unwrap();

        assert!(matches!(tokens[1].kind, TokenKind::CaretColonEqual));
        assert!(matches!(tokens[4].kind, TokenKind::CaretEqual));
        assert!(matches!(tokens[7].kind, TokenKind::Caret));
        assert!(matches!(tokens[10].kind, TokenKind::StarStar));
        assert!(matches!(tokens[13].kind, TokenKind::StarEqual));
        assert!(matches!(tokens[16].kind, TokenKind::Star));
        assert!(matches!(tokens[19].kind, TokenKind::SlashSlashAt));
        assert!(matches!(tokens[22].kind, TokenKind::SlashAt));
        assert!(matches!(tokens[25].kind, TokenKind::SlashSlash));
        assert!(matches!(tokens[28].kind, TokenKind::BarMinusGreater));
        assert!(matches!(tokens[31].kind, TokenKind::Or));
        assert!(matches!(tokens[34].kind, TokenKind::Bar));
    }

    #[test]
    fn tokens_carry_byte_spans() {
        let tokens = lex("f[x]").unwrap();

        assert_eq!(tokens[0].span, Span::new(0, 1));
        assert_eq!(tokens[1].span, Span::new(1, 2));
        assert_eq!(tokens[2].span, Span::new(2, 3));
        assert_eq!(tokens[3].span, Span::new(3, 4));
        assert_eq!(tokens[4].span, Span::point(4));
    }

    #[test]
    fn comments_do_not_shift_token_spans_for_following_tokens() {
        let tokens = lex("x(*comment*)+y").unwrap();

        assert_eq!(tokens[0].span, Span::new(0, 1));
        assert_eq!(tokens[1].span, Span::new(12, 13));
        assert_eq!(tokens[2].span, Span::new(13, 14));
    }

    #[test]
    fn preserves_comments_and_whitespace_as_leading_trivia() {
        let lexed = lex_with_trivia("f[ (* a *) x, y]").unwrap();
        let trivia = lexed.leading_trivia(2).unwrap();

        assert_eq!(trivia.len(), 3);
        assert_eq!(trivia[0].kind, TriviaKind::Whitespace);
        assert_eq!(trivia[0].span, Span::new(2, 3));
        assert_eq!(trivia[0].text, " ");
        assert_eq!(trivia[1].kind, TriviaKind::Comment);
        assert_eq!(trivia[1].span, Span::new(3, 10));
        assert_eq!(trivia[1].text, "(* a *)");
        assert_eq!(trivia[2].kind, TriviaKind::Whitespace);
        assert_eq!(trivia[2].span, Span::new(10, 11));
        assert_eq!(trivia[2].text, " ");
    }

    #[test]
    fn preserves_trailing_trivia_on_end_token() {
        let lexed = lex_with_trivia("x (* tail *)").unwrap();
        let trivia = lexed.leading_trivia(1).unwrap();

        assert_eq!(trivia.len(), 2);
        assert_eq!(trivia[0].kind, TriviaKind::Whitespace);
        assert_eq!(trivia[1].kind, TriviaKind::Comment);
        assert_eq!(trivia[1].text, "(* tail *)");
    }

    #[test]
    fn lexes_string_literals_with_basic_and_unicode_escapes() {
        let tokens = lex("\"a\\n\\r\\t\\u{1F642}\"").unwrap();

        assert!(matches!(
            &tokens[0].kind,
            TokenKind::String(value) if value == "a\n\r\t🙂"
        ));
    }
}
