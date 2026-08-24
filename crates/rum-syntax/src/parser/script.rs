//! Boundary-aware script splitting: one lex pass plus one parse pass.
//!
//! [`parse_script_forms`] drives the (guard-instrumented) recursive-descent
//! parser over the whole token stream, form by form, so every token is parsed
//! exactly once — and each parsed form keeps the [`SyntaxExpr`] that parse
//! built, so evaluation lowers it without parsing a second time. The stop
//! rule and its equivalence to the line-driven splitter the runtime keeps as
//! its lex-failure fallback:
//!
//! - **Prefix determinism.** The parser never backtracks, so its decision at
//!   token T depends only on the tokens before T. Its two next-token
//!   lookaheads never cross a line boundary here: `!!`/slot absorption are
//!   adjacency-gated (a line-first token always carries trivia), and `]]`
//!   pairing is suppressed across a line break in split mode — the
//!   line-driven splitter's line prefix ends between the two brackets and
//!   hard-errors on the lone `]`, which the suppressed pairing reproduces.
//! - **Optional/required dichotomy.** At delimiter depth 0 the parser's
//!   decision about a new-line token is either *optional* (stopping yields a
//!   complete expression — the boundary guards decline and the form ends) or
//!   *required* (stopping yields an incomplete/hard error — the token is
//!   consumed and the form continues). That is precisely when the line-driven
//!   loop keeps accumulating versus flushes a form. Inside a committed
//!   construct whose continuation is required (`~`-infix heads, brackets)
//!   the guard stays off, so an inner stop can never fabricate a hard error
//!   the line-driven loop does not produce.

use super::*;
use crate::lexer::lex_with_trivia;
use crate::trivia::{LexedSource, TriviaKind};

/// One script form produced by [`parse_script_forms`]: its text always, and
/// the parsed syntax when the splitter built it.
///
/// Text is the form's primary view, reached through
/// [`text`](ScriptForm::text); the type deliberately does not deref to `&str`,
/// so a caller cannot silently lose track of which state it holds. Evaluation
/// lowers a [`Parsed`](ScriptForm::Parsed) form directly instead of re-parsing
/// its text; an [`Unparsed`](ScriptForm::Unparsed) form is parsed at evaluation
/// time exactly as a form was before forms carried syntax.
#[derive(Clone, Debug)]
pub enum ScriptForm {
    /// The splitter parsed this form; `syntax` is the tree its parse built
    /// (spans are whole-source offsets).
    Parsed { text: String, syntax: SyntaxExpr },
    /// Text only. Produced for a form the splitter could not parse (a form
    /// poisoned by a hard error is emitted as text precisely so the error
    /// surfaces downstream when evaluation parses it) and for every form the
    /// legacy line-driven fallback yields.
    Unparsed { text: String },
}

impl ScriptForm {
    /// The form's source text: whole source lines, `\n`-joined and trimmed.
    pub fn text(&self) -> &str {
        match self {
            ScriptForm::Parsed { text, .. } => text,
            ScriptForm::Unparsed { text } => text,
        }
    }
}

/// Why [`parse_script_forms`] could not produce forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScriptFormsError {
    /// The whole-source lex failed (unterminated string or comment, stray
    /// character). Callers fall back to the line-driven splitter, which
    /// evaluates the earlier forms before surfacing the tail error.
    LexFailed(SyntaxError),
    /// The final form is incomplete. Carries the error a parse of the
    /// accumulated tail text produces, so message and span are byte-identical
    /// to the line-driven splitter's final parse.
    IncompleteTail(SyntaxError),
}

/// Splits `source` into script forms in one lex plus one parse pass.
///
/// Form text is whole source lines — joined with `\n` (so CRLF sources lose
/// their `\r`, exactly as the line-driven splitter's `lines()`-based
/// accumulation does) and trimmed. A form whose parse succeeded carries the
/// [`SyntaxExpr`] that parse built, so evaluation can lower it without
/// parsing again; a form the splitter could not parse carries text only.
pub fn parse_script_forms(source: &str) -> Result<Vec<ScriptForm>, ScriptFormsError> {
    let lexed = match lex_with_trivia(source) {
        Ok(lexed) => lexed,
        Err(err) => return Err(ScriptFormsError::LexFailed(err)),
    };

    let tokens = lexed.tokens();
    let line_breaks = line_break_flags(&lexed);
    let mut forms = Vec::new();
    let mut start = 0;

    while !matches!(tokens[start].kind, TokenKind::End) {
        let mut parser =
            Parser::<true>::new_with_line_breaks(&tokens[start..], &line_breaks[start..]);

        match parser.parse_root() {
            Ok(syntax) => {
                // The parser stopped at the End token or at a boundary guard
                // (a line-first token declined at an optional decision), so
                // the form is exactly the tokens the slice parser consumed.
                let end = start + parser.index;
                let mut boundary = line_end(source, tokens[end - 1].span.end);
                extend_through_straddling_comments(source, &lexed, end, &mut boundary);
                forms.push(ScriptForm::Parsed {
                    text: form_text(source, form_start(source, &lexed, start), boundary),
                    syntax,
                });
                start = end;
            }
            Err(err) if err.is_incomplete() => {
                // Reachable only at the stream's End token: the tail form.
                // Re-parse the legacy-shaped accumulation (every line from
                // the form's first line to EOF, `\n`-joined the way
                // `lines()`-based accumulation joins them — no trailing
                // newline survives) so the error's message and span stay
                // byte-identical at O(tail) cost.
                let tail = source[form_start(source, &lexed, start)..]
                    .lines()
                    .collect::<Vec<_>>()
                    .join("\n");

                return match parse_expr(&tail) {
                    Ok(syntax) => {
                        forms.push(ScriptForm::Parsed {
                            text: tail.trim().to_string(),
                            syntax,
                        });
                        Ok(forms)
                    }
                    Err(SyntaxError::EmptyInput) => Ok(forms),
                    Err(tail_err) if tail_err.is_incomplete() => {
                        Err(ScriptFormsError::IncompleteTail(tail_err))
                    }
                    Err(_) => {
                        forms.push(ScriptForm::Unparsed {
                            text: tail.trim().to_string(),
                        });
                        Ok(forms)
                    }
                };
            }
            Err(err) => {
                // Hard error: the form is poisoned. It swallows every token
                // starting on or before the offending token's line, extended
                // through the end line of anything straddling those lines
                // (multi-line strings, multi-line comments) — exactly the
                // line prefix at which the line-driven splitter's
                // accumulated parse first fails hard; the next form resumes
                // after that line.
                let offending = err
                    .span()
                    .expect("hard parse errors carry a token span")
                    .start;
                let mut boundary = line_end(source, offending);
                let mut end = start;

                loop {
                    while end + 1 < tokens.len() && tokens[end].span.start < boundary {
                        end += 1;
                    }

                    let mut extended = false;
                    let token_end_line = line_end(source, tokens[end - 1].span.end);

                    if token_end_line > boundary {
                        boundary = token_end_line;
                        extended = true;
                    }

                    if extend_through_straddling_comments(source, &lexed, end, &mut boundary) {
                        extended = true;
                    }

                    if !extended {
                        break;
                    }
                }

                forms.push(ScriptForm::Unparsed {
                    text: form_text(source, form_start(source, &lexed, start), boundary),
                });
                start = end;
            }
        }
    }

    Ok(forms)
}

/// `flags[i]`: token `i` may start a new form — its leading trivia contains a
/// real line break, i.e. a newline outside comments. A newline inside a
/// comment does not count: the line-driven accumulator cannot flush at a
/// boundary that its accumulated prefix fails to lex past.
fn line_break_flags(lexed: &LexedSource) -> Vec<bool> {
    (0..lexed.tokens().len())
        .map(|index| {
            index > 0
                && lexed.leading_trivia(index).is_some_and(|pieces| {
                    pieces.iter().any(|piece| {
                        piece.kind == TriviaKind::Whitespace && piece.text.contains('\n')
                    })
                })
        })
        .collect()
}

/// Byte offset where the form whose first token is `index` starts: the start
/// of the token's line, pulled back to the first line of any leading comment
/// that straddles the line boundary before it (those comment lines were
/// accumulated into this form, never flushed or cleared separately).
fn form_start(source: &str, lexed: &LexedSource, index: usize) -> usize {
    let mut start = line_start(source, lexed.tokens()[index].span.start);

    for piece in lexed.leading_trivia(index).unwrap_or(&[]).iter().rev() {
        if piece.kind == TriviaKind::Comment && piece.span.start < start && piece.span.end >= start
        {
            start = line_start(source, piece.span.start);
        }
    }

    start
}

/// Extends a form's end-line `boundary` through comments in token `index`'s
/// leading trivia that straddle out of it; returns whether anything moved.
fn extend_through_straddling_comments(
    source: &str,
    lexed: &LexedSource,
    index: usize,
    boundary: &mut usize,
) -> bool {
    let mut extended = false;

    for piece in lexed.leading_trivia(index).unwrap_or(&[]) {
        if piece.kind == TriviaKind::Comment
            && piece.span.start < *boundary
            && piece.span.end > *boundary
        {
            *boundary = line_end(source, piece.span.end);
            extended = true;
        }
    }

    extended
}

/// Offset where the line containing `offset` starts.
fn line_start(source: &str, offset: usize) -> usize {
    source[..offset].rfind('\n').map_or(0, |index| index + 1)
}

/// Offset where the line containing `offset` ends (at its `\n`, or EOF).
fn line_end(source: &str, offset: usize) -> usize {
    source[offset..]
        .find('\n')
        .map_or(source.len(), |index| offset + index)
}

/// Form text for the line range `from_byte..=boundary`: `\n`-joined with
/// `\r` stripped per line (CRLF drift guard), then trimmed — the same shape
/// the line-driven accumulation produced.
fn form_text(source: &str, from_byte: usize, boundary: usize) -> String {
    joined_lines(&source[from_byte..boundary])
        .trim()
        .to_string()
}

/// `slice.lines().join("\n")` with a no-`\r` fast path, matching what
/// `str::lines()`-based accumulation does to CRLF sources.
fn joined_lines(slice: &str) -> String {
    if slice.contains('\r') {
        slice.lines().collect::<Vec<_>>().join("\n")
    } else {
        slice.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn forms(source: &str) -> Vec<String> {
        parse_script_forms(source)
            .expect("source should split")
            .iter()
            .map(|form| form.text().to_string())
            .collect()
    }

    fn tail_error(source: &str) -> String {
        match parse_script_forms(source) {
            Err(ScriptFormsError::IncompleteTail(err)) => err.to_string(),
            other => panic!("expected an incomplete tail, got {other:?}"),
        }
    }

    #[test]
    fn splits_on_new_line_tokens_at_depth_zero() {
        assert_eq!(forms("1\n2\n3"), vec!["1", "2", "3"]);
        assert_eq!(
            forms("a = 1\nb = a + 1\na + b"),
            vec!["a = 1", "b = a + 1", "a + b"]
        );
        // Depth 0 juxtaposition is optional, so `a` and `b` are two forms —
        // but inside brackets the continuation is at depth >= 1 and one form
        // spans the lines.
        assert_eq!(forms("a\nb"), vec!["a", "b"]);
        assert_eq!(forms("{a\nb}"), vec!["{a\nb}"]);
    }

    #[test]
    fn required_continuations_cross_line_boundaries() {
        assert_eq!(forms("1 +\n2 +\n3"), vec!["1 +\n2 +\n3"]);
        assert_eq!(forms("f[1,\n2,\n3]"), vec!["f[1,\n2,\n3]"]);
        assert_eq!(
            forms("1 +\n(* comment\nlines *) 2"),
            vec!["1 +\n(* comment\nlines *) 2"]
        );
    }

    #[test]
    fn stops_before_new_line_operators_and_postfixes() {
        assert_eq!(forms("a\n+ b"), vec!["a", "+ b"]);
        assert_eq!(forms("f &\n@ g"), vec!["f &", "@ g"]);
        assert_eq!(forms("x = 1\nw = 1 ! 2"), vec!["x = 1", "w = 1 ! 2"]);
        assert_eq!(forms("a;;\nb"), vec!["a;;", "b"]);
        assert_eq!(forms("a; b\n; c"), vec!["a; b", "; c"]);
        assert_eq!(forms("f[x]\n[y]"), vec!["f[x]", "[y]"]);
    }

    #[test]
    fn shebang_blank_and_comment_lines_never_start_a_form() {
        assert_eq!(forms("#!/usr/bin/env rum\n\n1\n2"), vec!["1", "2"]);
        assert_eq!(forms("(* header *)\n1\n(* mid *)\n2"), vec!["1", "2"]);
        assert_eq!(
            forms("1 +\n(* keeps accumulating *)\n2"),
            vec!["1 +\n(* keeps accumulating *)\n2"]
        );
    }

    #[test]
    fn hard_errors_poison_through_the_offending_line() {
        // `1 2` parses fine (Times[1, 2]); the poison path needs a hard error.
        assert_eq!(forms("1 !\n3"), vec!["1 !", "3"]);
        assert_eq!(forms("q = 1\nw = 1 ! 2"), vec!["q = 1", "w = 1 ! 2"]);
        // A multi-line string opened on the poisoned line drags the form
        // through its closing line, as the line-driven splitter does.
        assert_eq!(forms("1 ! \"a\nb\" x\ny"), vec!["1 ! \"a\nb\" x", "y"]);
    }

    #[test]
    fn incomplete_tail_reports_the_legacy_shaped_error() {
        assert_eq!(
            tail_error("x = 1\ny +"),
            "incomplete input: expected expression at bytes 3..3"
        );
        assert_eq!(
            tail_error("data = {"),
            "incomplete input: expected } at bytes 8..8"
        );
    }

    #[test]
    fn lex_failures_are_reported_for_the_caller_to_fall_back() {
        for source in ["s = \"unterminated", "(* open comment\nx = 1", "x = `y`?"] {
            assert!(
                matches!(
                    parse_script_forms(source),
                    Err(ScriptFormsError::LexFailed(_))
                ),
                "{source:?} should fail the whole-source lex"
            );
        }
    }

    #[test]
    fn crlf_sources_lose_their_carriage_returns() {
        assert_eq!(
            forms("a = 1\r\nb = {1,\r\n2}\r\nc"),
            vec!["a = 1", "b = {1,\n2}", "c"]
        );
    }

    #[test]
    fn parsed_forms_carry_syntax_and_poisoned_forms_do_not() {
        let forms = parse_script_forms("a = 1\n2 !").expect("source should split");

        assert_eq!(forms.len(), 2);
        // `a = 1` parsed, so the splitter hands its syntax along.
        assert!(matches!(forms[0], ScriptForm::Parsed { .. }));
        assert_eq!(forms[0].text(), "a = 1");
        assert_eq!(forms[0].text(), "a = 1");
        // `2 !` is a hard error (trailing `!`), so the form is text only and
        // the error surfaces downstream when evaluation parses it.
        assert!(matches!(forms[1], ScriptForm::Unparsed { .. }));
        assert_eq!(forms[1].text(), "2 !");
    }
}
