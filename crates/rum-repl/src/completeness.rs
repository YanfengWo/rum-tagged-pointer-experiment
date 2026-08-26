use rum_syntax::{SyntaxTokenKind, lex_with_trivia, raw_string_operand_position_after};

const MAX_CERTIFIED_DELIMITER_DEPTH: usize = 64;
const MAX_CERTIFIED_OPERATOR_TOKENS: usize = 64;
const MAX_CERTIFIED_IMPLICIT_LINKS: usize = 1_024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CompletenessHint {
    DefinitelyIncomplete,
    NeedsFullCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LexicalState {
    Normal,
    Quoted { escaped: bool },
    EscapedMultiline { escaped: bool },
    RawMultiline,
    Comment { depth: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DelimiterKind {
    Root,
    Call,
    Part,
    List,
    Association,
    Group,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExprState {
    Empty,
    NeedsOperand,
    Complete,
    AfterComma,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Frame {
    kind: DelimiterKind,
    state: ExprState,
}

impl Frame {
    const fn new(kind: DelimiterKind) -> Self {
        Self {
            kind,
            state: ExprState::Empty,
        }
    }
}

/// A conservative, incremental filter in front of the host completeness check.
///
/// Normal source spans are tokenized by `rum-syntax`; this type only carries
/// the state that the lexer itself must carry across appended lines (strings
/// and nested comments). The small expression-prefix recognizer prevents an
/// unmatched delimiter from hiding a hard parser error. Any construct it does
/// not certify falls back to the full host check.
pub(super) struct IncrementalCompleteness {
    lexical: LexicalState,
    frames: Vec<Frame>,
    trusted_prefix: bool,
    raw_string_operand_position: bool,
    trivia_before_next_token: bool,
    pending_part_close: bool,
    operator_tokens: usize,
    implicit_links: usize,
    #[cfg(test)]
    examined_bytes: usize,
}

impl Default for IncrementalCompleteness {
    fn default() -> Self {
        Self {
            lexical: LexicalState::Normal,
            frames: vec![Frame::new(DelimiterKind::Root)],
            trusted_prefix: true,
            raw_string_operand_position: true,
            trivia_before_next_token: false,
            pending_part_close: false,
            operator_tokens: 0,
            implicit_links: 0,
            #[cfg(test)]
            examined_bytes: 0,
        }
    }
}

impl IncrementalCompleteness {
    pub(super) fn append(&mut self, text: &str) {
        self.record_examined(text.len());
        if !self.trusted_prefix {
            return;
        }

        let mut offset = 0;
        while offset < text.len() && self.trusted_prefix {
            offset = match self.lexical {
                LexicalState::Normal => self.scan_normal(text, offset),
                LexicalState::Quoted { escaped } => {
                    self.scan_escaped_string(text, offset, escaped, false)
                }
                LexicalState::EscapedMultiline { escaped } => {
                    self.scan_escaped_string(text, offset, escaped, true)
                }
                LexicalState::RawMultiline => self.scan_raw_string(text, offset),
                LexicalState::Comment { depth } => self.scan_comment(text, offset, depth),
            };
        }

        // A part closer is two `]` tokens on the same source line. The real
        // parser hard-errors on a lone first `]`, so it is never a safe fast
        // incomplete result at an append boundary.
        if self.pending_part_close {
            self.distrust();
        }
    }

    pub(super) fn hint(&self) -> CompletenessHint {
        if self.trusted_prefix && (self.lexical != LexicalState::Normal || self.frames.len() > 1) {
            CompletenessHint::DefinitelyIncomplete
        } else {
            CompletenessHint::NeedsFullCheck
        }
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    #[cfg(test)]
    pub(super) fn examined_bytes(&self) -> usize {
        self.examined_bytes
    }

    fn scan_normal(&mut self, text: &str, start: usize) -> usize {
        let bytes = text.as_bytes();
        let mut offset = start;

        while offset < text.len() {
            if bytes[offset] == b'\\' && bytes.get(offset + 1) == Some(&b'[') {
                if let Some(end) = text[offset + 2..].find(']') {
                    offset += end + 3;
                    continue;
                }

                self.lex_normal_span(&text[start..]);
                return text.len();
            }

            // `#!` is offset-sensitive shebang trivia. Segmenting normal text
            // would change its apparent offset, so conservatively fall back.
            if bytes[offset] == b'#' && bytes.get(offset + 1) == Some(&b'!') {
                self.lex_normal_span(&text[start..offset]);
                self.distrust();
                return text.len();
            }

            if bytes[offset] == b'(' && bytes.get(offset + 1) == Some(&b'*') {
                self.lex_normal_span(&text[start..offset]);
                if !self.trusted_prefix {
                    return text.len();
                }
                self.lexical = LexicalState::Comment { depth: 1 };
                self.trivia_before_next_token = true;
                return offset + 2;
            }

            if bytes[offset] == b'"' {
                self.lex_normal_span(&text[start..offset]);
                if !self.trusted_prefix {
                    return text.len();
                }
                if text[offset..].starts_with("\"\"\"") {
                    self.lexical = LexicalState::EscapedMultiline { escaped: false };
                    return offset + 3;
                }
                self.lexical = LexicalState::Quoted { escaped: false };
                return offset + 1;
            }

            if bytes[offset] == b'\'' && text[offset..].starts_with("'''") {
                self.lex_normal_span(&text[start..offset]);
                if !self.trusted_prefix {
                    return text.len();
                }
                if self.raw_string_operand_position {
                    self.lexical = LexicalState::RawMultiline;
                } else {
                    let first_has_trivia = self.trivia_before_next_token;
                    self.process_token(&SyntaxTokenKind::Prime, first_has_trivia);
                    self.process_token(&SyntaxTokenKind::Prime, false);
                    self.process_token(&SyntaxTokenKind::Prime, false);
                }
                return offset + 3;
            }

            offset += text[offset..]
                .chars()
                .next()
                .expect("offset is in bounds")
                .len_utf8();
        }

        self.lex_normal_span(&text[start..]);
        text.len()
    }

    fn scan_escaped_string(
        &mut self,
        text: &str,
        mut offset: usize,
        mut escaped: bool,
        multiline: bool,
    ) -> usize {
        while offset < text.len() {
            if escaped {
                if text.as_bytes()[offset] == b'[' {
                    // Named-character validation belongs to the real lexer.
                    self.distrust();
                    return text.len();
                }
                offset += text[offset..]
                    .chars()
                    .next()
                    .expect("offset is in bounds")
                    .len_utf8();
                escaped = false;
                continue;
            }

            if multiline && text[offset..].starts_with("\"\"\"") {
                self.finish_string();
                return offset + 3;
            }
            if !multiline && text.as_bytes()[offset] == b'"' {
                self.finish_string();
                return offset + 1;
            }
            if text.as_bytes()[offset] == b'\\' {
                escaped = true;
                offset += 1;
                continue;
            }

            offset += text[offset..]
                .chars()
                .next()
                .expect("offset is in bounds")
                .len_utf8();
        }

        self.lexical = if multiline {
            LexicalState::EscapedMultiline { escaped }
        } else {
            LexicalState::Quoted { escaped }
        };
        text.len()
    }

    fn scan_raw_string(&mut self, text: &str, offset: usize) -> usize {
        if let Some(end) = text[offset..].find("'''") {
            self.finish_string();
            offset + end + 3
        } else {
            text.len()
        }
    }

    fn scan_comment(&mut self, text: &str, mut offset: usize, mut depth: usize) -> usize {
        while offset < text.len() {
            if text[offset..].starts_with("(*") {
                depth += 1;
                offset += 2;
            } else if text[offset..].starts_with("*)") {
                depth -= 1;
                offset += 2;
                if depth == 0 {
                    self.lexical = LexicalState::Normal;
                    return offset;
                }
            } else {
                offset += text[offset..]
                    .chars()
                    .next()
                    .expect("offset is in bounds")
                    .len_utf8();
            }
        }

        self.lexical = LexicalState::Comment { depth };
        text.len()
    }

    fn lex_normal_span(&mut self, source: &str) {
        if source.is_empty() || !self.trusted_prefix {
            return;
        }
        self.record_examined(source.len());

        let Ok(lexed) = lex_with_trivia(source) else {
            self.distrust();
            return;
        };

        for token in lexed.tokens() {
            if matches!(token.kind, SyntaxTokenKind::End) {
                self.trivia_before_next_token |= !token.leading_trivia.is_empty();
                break;
            }
            let has_leading_trivia =
                self.trivia_before_next_token || !token.leading_trivia.is_empty();
            self.process_token(&token.kind, has_leading_trivia);
            if !self.trusted_prefix {
                break;
            }
        }
    }

    fn finish_string(&mut self) {
        let has_leading_trivia = self.trivia_before_next_token;
        self.lexical = LexicalState::Normal;
        self.process_token(&SyntaxTokenKind::String(String::new()), has_leading_trivia);
    }

    fn process_token(&mut self, kind: &SyntaxTokenKind, has_leading_trivia: bool) {
        let started_operand_position = self.raw_string_operand_position;
        self.raw_string_operand_position =
            raw_string_operand_position_after(kind, started_operand_position);
        self.trivia_before_next_token = false;

        if self.pending_part_close {
            if matches!(kind, SyntaxTokenKind::RBracket) {
                self.pending_part_close = false;
                self.close(DelimiterKind::Part);
            } else {
                self.distrust();
            }
            return;
        }

        match kind {
            SyntaxTokenKind::Integer(_)
            | SyntaxTokenKind::String(_)
            | SyntaxTokenKind::Symbol(_)
            | SyntaxTokenKind::Hash => self.accept_primary(),
            SyntaxTokenKind::Underscore
            | SyntaxTokenKind::DoubleUnderscore
            | SyntaxTokenKind::TripleUnderscore => {
                if self.count_operator() {
                    self.accept_primary();
                }
            }
            SyntaxTokenKind::LBracket => self.open_postfix(DelimiterKind::Call),
            SyntaxTokenKind::DoubleLBracket => self.open_postfix(DelimiterKind::Part),
            SyntaxTokenKind::LBrace => self.open_primary(DelimiterKind::List),
            SyntaxTokenKind::LAssociation => self.open_primary(DelimiterKind::Association),
            SyntaxTokenKind::LParen => self.open_primary(DelimiterKind::Group),
            SyntaxTokenKind::RBracket => {
                if self.current_kind() == DelimiterKind::Part {
                    self.pending_part_close = true;
                } else {
                    self.close(DelimiterKind::Call);
                }
            }
            SyntaxTokenKind::DoubleRBracket => self.close(DelimiterKind::Part),
            SyntaxTokenKind::RBrace => self.close(DelimiterKind::List),
            SyntaxTokenKind::RAssociation => self.close(DelimiterKind::Association),
            SyntaxTokenKind::RParen => self.close(DelimiterKind::Group),
            SyntaxTokenKind::Comma => self.comma(),
            SyntaxTokenKind::Plus | SyntaxTokenKind::Minus => self.prefix_or_binary(),
            SyntaxTokenKind::PlusPlus | SyntaxTokenKind::MinusMinus | SyntaxTokenKind::Bang => {
                self.prefix_or_postfix(kind, has_leading_trivia)
            }
            SyntaxTokenKind::Prime => self.postfix(!has_leading_trivia),
            SyntaxTokenKind::Repeated | SyntaxTokenKind::RepeatedNull => self.postfix(true),
            SyntaxTokenKind::Unset | SyntaxTokenKind::Ampersand => self.distrust(),
            SyntaxTokenKind::Semicolon
            | SyntaxTokenKind::Span
            | SyntaxTokenKind::Tilde
            | SyntaxTokenKind::Dot => self.distrust(),
            SyntaxTokenKind::At
            | SyntaxTokenKind::AtAt
            | SyntaxTokenKind::AtAtAt
            | SyntaxTokenKind::AtStar
            | SyntaxTokenKind::PlusEqual
            | SyntaxTokenKind::MinusEqual
            | SyntaxTokenKind::Star
            | SyntaxTokenKind::StarStar
            | SyntaxTokenKind::StarEqual
            | SyntaxTokenKind::Slash
            | SyntaxTokenKind::SlashAt
            | SyntaxTokenKind::SlashEqual
            | SyntaxTokenKind::SlashSlash
            | SyntaxTokenKind::SlashSlashAt
            | SyntaxTokenKind::SlashSlashEqual
            | SyntaxTokenKind::SlashStar
            | SyntaxTokenKind::TildeTilde
            | SyntaxTokenKind::Caret
            | SyntaxTokenKind::CaretEqual
            | SyntaxTokenKind::CaretColonEqual
            | SyntaxTokenKind::And
            | SyntaxTokenKind::Or
            | SyntaxTokenKind::Same
            | SyntaxTokenKind::Unsame
            | SyntaxTokenKind::Equal
            | SyntaxTokenKind::Unequal
            | SyntaxTokenKind::Less
            | SyntaxTokenKind::Greater
            | SyntaxTokenKind::LessEqual
            | SyntaxTokenKind::GreaterEqual
            | SyntaxTokenKind::StringJoin
            | SyntaxTokenKind::Colon
            | SyntaxTokenKind::MessageName
            | SyntaxTokenKind::TagSet
            | SyntaxTokenKind::Arrow
            | SyntaxTokenKind::RuleDelayed
            | SyntaxTokenKind::Set
            | SyntaxTokenKind::SetDelayed
            | SyntaxTokenKind::ReplaceAll
            | SyntaxTokenKind::ReplaceRepeated
            | SyntaxTokenKind::Condition
            | SyntaxTokenKind::Bar
            | SyntaxTokenKind::BarMinusGreater
            | SyntaxTokenKind::Question => self.binary(),
            SyntaxTokenKind::End => {}
        }
    }

    fn accept_primary(&mut self) {
        if self.current_state() == ExprState::Complete {
            self.implicit_links += 1;
            if self.implicit_links > MAX_CERTIFIED_IMPLICIT_LINKS {
                self.distrust();
                return;
            }
        }
        self.set_current_state(ExprState::Complete);
    }

    fn open_primary(&mut self, kind: DelimiterKind) {
        self.accept_primary();
        if self.trusted_prefix {
            self.push_frame(kind);
        }
    }

    fn open_postfix(&mut self, kind: DelimiterKind) {
        if self.current_state() != ExprState::Complete {
            self.distrust();
            return;
        }
        self.push_frame(kind);
    }

    fn push_frame(&mut self, kind: DelimiterKind) {
        self.frames.push(Frame::new(kind));
        if self.frames.len() - 1 > MAX_CERTIFIED_DELIMITER_DEPTH {
            self.distrust();
        }
    }

    fn close(&mut self, expected: DelimiterKind) {
        let Some(frame) = self.frames.last().copied() else {
            self.distrust();
            return;
        };
        let allow_empty = !matches!(expected, DelimiterKind::Group);
        if frame.kind != expected
            || !(frame.state == ExprState::Complete
                || (allow_empty && frame.state == ExprState::Empty))
        {
            self.distrust();
            return;
        }
        self.frames.pop();
    }

    fn comma(&mut self) {
        if !matches!(
            self.current_kind(),
            DelimiterKind::Call
                | DelimiterKind::Part
                | DelimiterKind::List
                | DelimiterKind::Association
        ) || self.current_state() != ExprState::Complete
        {
            self.distrust();
            return;
        }
        self.set_current_state(ExprState::AfterComma);
    }

    fn prefix_or_binary(&mut self) {
        if !self.count_operator() {
            return;
        }
        self.set_current_state(ExprState::NeedsOperand);
    }

    fn prefix_or_postfix(&mut self, kind: &SyntaxTokenKind, has_leading_trivia: bool) {
        if !self.count_operator() {
            return;
        }
        if self.current_state() == ExprState::Complete {
            if matches!(kind, SyntaxTokenKind::Bang) && has_leading_trivia {
                self.distrust();
            }
        } else {
            self.set_current_state(ExprState::NeedsOperand);
        }
    }

    fn binary(&mut self) {
        if !self.count_operator() {
            return;
        }
        if self.current_state() != ExprState::Complete {
            self.distrust();
            return;
        }
        self.set_current_state(ExprState::NeedsOperand);
    }

    fn postfix(&mut self, allowed: bool) {
        if !self.count_operator() {
            return;
        }
        if !allowed || self.current_state() != ExprState::Complete {
            self.distrust();
        }
    }

    fn count_operator(&mut self) -> bool {
        self.operator_tokens += 1;
        if self.operator_tokens > MAX_CERTIFIED_OPERATOR_TOKENS {
            self.distrust();
        }
        self.trusted_prefix
    }

    fn current_kind(&self) -> DelimiterKind {
        self.frames
            .last()
            .expect("root frame is always present")
            .kind
    }

    fn current_state(&self) -> ExprState {
        self.frames
            .last()
            .expect("root frame is always present")
            .state
    }

    fn set_current_state(&mut self, state: ExprState) {
        self.frames
            .last_mut()
            .expect("root frame is always present")
            .state = state;
    }

    fn distrust(&mut self) {
        self.trusted_prefix = false;
    }

    fn record_examined(&mut self, bytes: usize) {
        #[cfg(test)]
        {
            self.examined_bytes += bytes;
        }
        #[cfg(not(test))]
        let _ = bytes;
    }
}

#[cfg(test)]
#[path = "completeness_tests.rs"]
mod tests;
