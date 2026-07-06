use super::operators::unicode_operator_token_kind;
use super::symbols::is_symbol_start;
use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn next_token(
        &mut self,
        trivia: &mut Vec<TriviaPiece>,
    ) -> Result<Option<Token>, SyntaxError> {
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

        let can_start_raw_string_before_token = self.can_start_raw_string;

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
            '\'' => {
                if self.can_start_raw_string && self.input[self.offset..].starts_with("'''") {
                    self.lex_raw_multiline_string()?
                } else {
                    self.single(TokenKind::Prime)
                }
            }
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
            '~' => {
                if self.consume_str("~~") {
                    TokenKind::TildeTilde
                } else {
                    self.single(TokenKind::Tilde)
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

        self.can_start_raw_string =
            token_can_be_followed_by_raw_string_operand(&token, can_start_raw_string_before_token);

        Ok(Some(Token::new(
            token,
            Span::new(start, self.offset),
            leading_trivia,
        )))
    }
}

fn token_can_be_followed_by_raw_string_operand(
    token: &TokenKind,
    token_started_operand_position: bool,
) -> bool {
    match token {
        TokenKind::End => false,
        TokenKind::LBracket
        | TokenKind::DoubleLBracket
        | TokenKind::LBrace
        | TokenKind::LAssociation
        | TokenKind::LParen
        | TokenKind::Comma
        | TokenKind::Semicolon
        | TokenKind::Span
        | TokenKind::At
        | TokenKind::AtAt
        | TokenKind::AtAtAt
        | TokenKind::AtStar
        | TokenKind::Plus
        | TokenKind::PlusEqual
        | TokenKind::Minus
        | TokenKind::MinusEqual
        | TokenKind::Star
        | TokenKind::StarStar
        | TokenKind::StarEqual
        | TokenKind::Slash
        | TokenKind::SlashAt
        | TokenKind::SlashEqual
        | TokenKind::SlashSlash
        | TokenKind::SlashSlashAt
        | TokenKind::SlashSlashEqual
        | TokenKind::SlashStar
        | TokenKind::TildeTilde
        | TokenKind::Tilde
        | TokenKind::Caret
        | TokenKind::CaretEqual
        | TokenKind::CaretColonEqual
        | TokenKind::And
        | TokenKind::Or
        | TokenKind::Same
        | TokenKind::Unsame
        | TokenKind::Equal
        | TokenKind::Unequal
        | TokenKind::Less
        | TokenKind::Greater
        | TokenKind::LessEqual
        | TokenKind::GreaterEqual
        | TokenKind::StringJoin
        | TokenKind::Colon
        | TokenKind::MessageName
        | TokenKind::TagSet
        | TokenKind::Arrow
        | TokenKind::RuleDelayed
        | TokenKind::Set
        | TokenKind::SetDelayed
        | TokenKind::ReplaceAll
        | TokenKind::ReplaceRepeated
        | TokenKind::Condition
        | TokenKind::Dot
        | TokenKind::Bar
        | TokenKind::BarMinusGreater
        | TokenKind::Question
        | TokenKind::Underscore
        | TokenKind::DoubleUnderscore
        | TokenKind::TripleUnderscore => true,
        TokenKind::PlusPlus | TokenKind::MinusMinus | TokenKind::Bang => {
            token_started_operand_position
        }
        TokenKind::Integer(_)
        | TokenKind::String(_)
        | TokenKind::Symbol(_)
        | TokenKind::RBracket
        | TokenKind::DoubleRBracket
        | TokenKind::RBrace
        | TokenKind::RAssociation
        | TokenKind::RParen
        | TokenKind::Prime
        | TokenKind::Unset
        | TokenKind::Repeated
        | TokenKind::RepeatedNull
        | TokenKind::Ampersand
        | TokenKind::Hash => false,
    }
}
