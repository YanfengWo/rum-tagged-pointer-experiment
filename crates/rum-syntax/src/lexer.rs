use crate::error::SyntaxError;
use crate::span::Span;
use crate::trivia::{LexedSource, SyntaxToken, SyntaxTokenKind, TriviaPiece};

pub(crate) type Token = SyntaxToken;
pub(crate) type TokenKind = SyntaxTokenKind;

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
                if self.consume_str("|>") {
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
                if self.consume_str("*=") {
                    TokenKind::StarEqual
                } else {
                    self.single(TokenKind::Star)
                }
            }
            '^' => self.single(TokenKind::Caret),
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
                } else if self.consume_str("/@") {
                    TokenKind::SlashAt
                } else if self.consume_str("//=") {
                    TokenKind::SlashSlashEqual
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
            ch if ch.is_ascii_digit() => self.lex_integer(),
            ch if is_symbol_start(ch) => self.lex_symbol(),
            other => {
                return Err(SyntaxError::UnexpectedCharacter {
                    found: other,
                    span: Span::new(start, self.offset),
                });
            }
        };

        Ok(Some(Token::new(
            token,
            Span::new(start, self.offset),
            leading_trivia,
        )))
    }

    fn lex_integer(&mut self) -> TokenKind {
        let start = self.offset;

        while matches!(self.peek_char(), Some(ch) if ch.is_ascii_digit()) {
            self.bump_char();
        }

        TokenKind::Integer(self.input[start..self.offset].to_string())
    }

    fn lex_symbol(&mut self) -> TokenKind {
        let start = self.offset;
        self.bump_char();

        while matches!(self.peek_char(), Some(ch) if is_symbol_continue(ch)) {
            self.bump_char();
        }

        TokenKind::Symbol(self.input[start..self.offset].to_string())
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
    ch.is_alphabetic() || ch == '$'
}

fn is_symbol_continue(ch: char) -> bool {
    ch.is_alphanumeric() || matches!(ch, '$' | '`')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TriviaKind;

    #[test]
    fn lexes_multi_char_operators() {
        let tokens = lex(
            "x::tag := y //. z /. q -> w /; p f /: g =. a[[1]] b;;c <|d -> e|> h@i j@@k l@@@m n@*o p/*q r/@s a&&b c||d !e f===g h=!=i j==k l!=m n<o p>q r<=s t>=u v:_ \"a\"<>\"b\"",
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
    fn lexes_replace_repeated_before_replace_all_prefix() {
        let tokens = lex("expr //. rule").unwrap();

        assert!(matches!(tokens[1].kind, TokenKind::ReplaceRepeated));
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
