use super::support::*;

#[test]
fn reports_incomplete_call_input() {
    let err = parse_expr("f[1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: "]",
            span: Span::point(3),
            opening_span: Span::new(1, 2),
            opening: "[",
        }
    );
    assert!(err.is_incomplete());
    assert_eq!(err.span(), Some(Span::point(3)));
    assert_eq!(err.related_span(), Some(Span::new(1, 2)));
}

#[test]
fn reports_incomplete_double_bracket_input() {
    let err = parse_expr("expr[[1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: "]]",
            span: Span::point(7),
            opening_span: Span::new(4, 6),
            opening: "[[",
        }
    );
}

#[test]
fn reports_incomplete_group_input() {
    let err = parse_expr("(x + 1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: ")",
            span: Span::point(6),
            opening_span: Span::new(0, 1),
            opening: "(",
        }
    );
}

#[test]
fn reports_incomplete_rhs_after_binary_operator() {
    let err = parse_expr("x +").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::Incomplete {
            expected: "expression",
            span: Span::point(3),
        }
    );
}

#[test]
fn reports_missing_closing_tilde_in_infix_application() {
    let err = parse_expr("a ~ f").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::Incomplete {
            expected: "~",
            span: Span::point(5),
        }
    );
}

#[test]
fn reports_malformed_call_arguments_without_marking_input_incomplete() {
    let err = parse_expr("f[1 ! 2]").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: ",",
            found: "!".to_string(),
            span: Span::new(4, 5),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_malformed_list_items_without_marking_input_incomplete() {
    let err = parse_expr("{1,,2}").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: "expression",
            found: ",".to_string(),
            span: Span::new(3, 4),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_trailing_pattern_tokens_as_hard_errors() {
    let err = parse_expr("x_,").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::TrailingInput {
            span: Span::new(2, 3),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_bare_pure_function_marker_as_hard_error() {
    let err = parse_expr("&").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: "expression",
            found: "&".to_string(),
            span: Span::new(0, 1),
        }
    );
    assert!(!err.is_incomplete());
}

#[test]
fn reports_unexpected_closing_delimiters_with_token_labels() {
    let err = parse_expr(")").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnexpectedToken {
            expected: "expression",
            found: ")".to_string(),
            span: Span::new(0, 1),
        }
    );
}

#[test]
fn distinguishes_empty_input_from_incomplete_input() {
    let err = parse_expr("").unwrap_err();

    assert_eq!(err, SyntaxError::EmptyInput);
    assert!(!err.is_incomplete());
    assert_eq!(err.span(), None);
}

#[test]
fn reports_incomplete_association_input() {
    let err = parse_expr("<|a -> 1").unwrap_err();

    assert_eq!(
        err,
        SyntaxError::UnclosedDelimiter {
            expected: "|>",
            span: Span::point(8),
            opening_span: Span::new(0, 2),
            opening: "<|",
        }
    );
    assert!(err.is_incomplete());
}
