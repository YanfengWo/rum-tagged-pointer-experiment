use super::*;

impl<'a> Lexer<'a> {
    pub(super) fn collect_trivia(
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
}
