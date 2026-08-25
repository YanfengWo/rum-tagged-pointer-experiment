use super::support::*;
use crate::{
    MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS, ScriptForm, parse_script_forms, parse_with_diagnostics,
};

#[test]
fn left_chain_homogeneous_boundary_is_exact() {
    let accepted = explicit_chain(&["+"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
    parse_expr(&accepted).expect("4,096 homogeneous links should parse");

    let rejected = explicit_chain(&["+"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    assert_limit_error(&rejected, last_explicit_operator_span(&rejected, "+"));
}

#[test]
fn left_chain_mixed_boundary_is_exact() {
    let accepted = explicit_chain(&["+", "-"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
    parse_expr(&accepted).expect("4,096 mixed links should parse");

    let rejected = explicit_chain(&["+", "-"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    let last_token = ["+", "-"][MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS % 2];
    assert_limit_error(
        &rejected,
        last_explicit_operator_span(&rejected, last_token),
    );
}

#[test]
fn left_chain_implicit_times_boundary_is_exact() {
    let accepted = implicit_times_chain(MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
    parse_expr(&accepted).expect("4,096 implicit Times links should parse");

    let rejected = implicit_times_chain(MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    let rejected_operand_span = Span::new(rejected.len() - 1, rejected.len());
    assert_limit_error(&rejected, rejected_operand_span);
}

#[test]
fn left_chain_independent_spines_have_independent_counts() {
    let first = explicit_chain(&["+"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
    let second = explicit_chain(&["-"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS);
    let accepted = format!("f[{first}] + g[{second}]");
    parse_expr(&accepted).expect("independent 4,096-link spines should parse");

    let over_limit = explicit_chain(&["+"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    let inner_span = last_explicit_operator_span(&over_limit, "+");
    let rejected = format!("f[{over_limit}] + g[a]");
    assert_limit_error(
        &rejected,
        Span::new(inner_span.start + 2, inner_span.end + 2),
    );
}

#[test]
fn left_chain_limit_error_is_terminal() {
    let span = Span::new(7, 8);
    let error = SyntaxError::LeftAssociativeChainTooLong {
        max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
        span,
    };

    assert!(!error.is_incomplete());
    assert_eq!(error.span(), Some(span));
    assert_eq!(
        error.to_string(),
        "left-associative chain exceeded limit 4096 at bytes 7..8"
    );
}

#[test]
fn left_chain_limit_diagnostic_marks_the_offending_operator() {
    let source = explicit_chain(&["+"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    let span = last_explicit_operator_span(&source, "+");
    let diagnostic = parse_with_diagnostics(source).expect_err("4,097 links should be rejected");

    assert_eq!(
        diagnostic.error(),
        &SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        }
    );

    let structured = diagnostic.structured();
    assert!(!structured.is_incomplete);
    assert_eq!(structured.spans.len(), 1);
    assert_eq!(structured.spans[0].span, span);
    assert_eq!(structured.spans[0].text, "+");
    assert!(
        structured
            .rendered
            .contains("error: left-associative chain exceeded limit 4096 at bytes")
    );
    assert!(
        structured
            .rendered
            .contains(&format!(" --> 1:{}", span.start + 1))
    );
    assert!(
        structured
            .rendered
            .lines()
            .last()
            .is_some_and(|line| line.ends_with('^'))
    );
}

#[test]
fn left_chain_limit_does_not_swallow_later_script_forms() {
    let over_limit = explicit_chain(&["+"], MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS + 1);
    let script = format!("{over_limit}\ny = 1\nz = 2");
    let forms = parse_script_forms(&script).expect("terminal limit error should split normally");

    assert_eq!(forms.len(), 3);
    assert!(matches!(forms[0], ScriptForm::Unparsed { .. }));
    assert_eq!(forms[0].text(), over_limit);
    assert!(matches!(forms[1], ScriptForm::Parsed { .. }));
    assert_eq!(forms[1].text(), "y = 1");
    assert!(matches!(forms[2], ScriptForm::Parsed { .. }));
    assert_eq!(forms[2].text(), "z = 2");
}

fn explicit_chain(tokens: &[&str], link_count: usize) -> String {
    let mut source = String::from("a");
    for index in 0..link_count {
        source.push_str(tokens[index % tokens.len()]);
        source.push('a');
    }
    source
}

fn implicit_times_chain(link_count: usize) -> String {
    let mut source = String::with_capacity(link_count * 2 + 1);
    source.push('a');
    for _ in 0..link_count {
        source.push_str(" a");
    }
    source
}

fn last_explicit_operator_span(source: &str, token: &str) -> Span {
    let start = source.len() - token.len() - 1;
    Span::new(start, start + token.len())
}

fn assert_limit_error(source: &str, span: Span) {
    let error = parse_expr(source).expect_err("one-past-the-limit chain should be rejected");
    assert_eq!(
        error,
        SyntaxError::LeftAssociativeChainTooLong {
            max: MAX_LEFT_ASSOCIATIVE_CHAIN_LINKS,
            span,
        }
    );
}
