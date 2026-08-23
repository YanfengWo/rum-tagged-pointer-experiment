use super::support::*;

use super::super::front::MAX_STATEMENT_CHAIN_DEPTH;

#[test]
fn parses_right_associative_apply_sugar() {
    let parsed = parse_expr("f @ g @ x").unwrap();
    assert_eq!(parsed.to_string(), "f @ g @ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::Apply,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::symbol("x", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_postfix_apply_as_left_associative() {
    let parsed = parse_expr("a // f // g").unwrap();
    assert_eq!(parsed.to_string(), "a // f // g");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::PostfixApply,
            SyntaxExpr::binary(
                BinaryOp::PostfixApply,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("f", z()),
                z(),
            ),
            SyntaxExpr::symbol("g", z()),
            z(),
        )
    );
}

#[test]
fn parses_postfix_apply_between_set_and_replace_all_precedence() {
    let set = parse_expr("a = b // f").unwrap();
    assert_eq!(
        set,
        SyntaxExpr::binary(
            BinaryOp::Set,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::PostfixApply,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("f", z()),
                z(),
            ),
            z(),
        )
    );

    let replace_all = parse_expr("a // f /. r").unwrap();
    assert_eq!(
        replace_all,
        SyntaxExpr::binary(
            BinaryOp::PostfixApply,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::ReplaceAll,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("r", z()),
                z(),
            ),
            z(),
        )
    );

    let plus = parse_expr("a + b // f").unwrap();
    assert_eq!(
        plus,
        SyntaxExpr::binary(
            BinaryOp::PostfixApply,
            SyntaxExpr::binary(
                BinaryOp::Add,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("b", z()),
                z(),
            ),
            SyntaxExpr::symbol("f", z()),
            z(),
        )
    );
}

#[test]
fn parses_compound_expression_sugar() {
    let parsed = parse_expr("a; b;").unwrap();
    assert_eq!(parsed.to_string(), "a; b; Null");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::CompoundExpression,
            SyntaxExpr::symbol("a", z()),
            SyntaxExpr::binary(
                BinaryOp::CompoundExpression,
                SyntaxExpr::symbol("b", z()),
                SyntaxExpr::symbol("Null", z()),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_long_single_form_statement_chain() {
    // brief-301 regression pin: `;` links are collected iteratively, so a
    // single form may hold more statements than the delimiter guard's depth
    // allows, up to the statement cap (brief-302). Count the statements by
    // walking the right spine iteratively — a recursive Display call on a
    // long chain would itself overflow a ~2 MiB test thread in debug builds.
    let count = 200;
    let mut input = String::new();
    for i in 0..count {
        if i > 0 {
            input.push_str("; ");
        }
        input.push_str(&format!("x{i} = {i}"));
    }

    let parsed = parse_expr(&input).expect("long statement chain should parse");

    let mut statements = 1;
    let mut node = &parsed;
    while let SyntaxExprKind::Binary {
        op: BinaryOp::CompoundExpression,
        rhs,
        ..
    } = &node.kind
    {
        statements += 1;
        node = rhs;
    }

    assert_eq!(statements, count);
}

#[test]
fn pins_statement_chain_boundary_with_dedicated_error() {
    // brief-302: the cap is counted along the whole spine, and one statement
    // past it fails with its own variant — NOT ParseDepthExceeded — whose
    // message names the limit and the "separate lines" way around it.
    fn statement_chain(count: usize) -> String {
        (0..count)
            .map(|i| format!("x{i} = {i}"))
            .collect::<Vec<_>>()
            .join("; ")
    }

    let parsed = parse_expr(&statement_chain(MAX_STATEMENT_CHAIN_DEPTH))
        .expect("a chain at the cap should parse");

    let mut statements = 1;
    let mut node = &parsed;
    while let SyntaxExprKind::Binary {
        op: BinaryOp::CompoundExpression,
        rhs,
        ..
    } = &node.kind
    {
        statements += 1;
        node = rhs;
    }
    assert_eq!(statements, MAX_STATEMENT_CHAIN_DEPTH);

    let error = parse_expr(&statement_chain(MAX_STATEMENT_CHAIN_DEPTH + 1))
        .expect_err("one statement past the cap should fail");
    assert!(matches!(
        error,
        SyntaxError::StatementChainTooLong {
            max: MAX_STATEMENT_CHAIN_DEPTH,
            ..
        }
    ));
    // If this ever flipped to "incomplete", split_script_into_forms would
    // swallow every later line of the script into the offending one.
    assert!(!error.is_incomplete());
    let message = error.to_string();
    assert!(message.contains(&MAX_STATEMENT_CHAIN_DEPTH.to_string()));
    assert!(message.contains("separate lines"));
}

#[test]
fn rejects_empty_statement_spine_without_overflowing() {
    // brief-302 regression, the reason the cap is counted in the climbing
    // loop: consecutive EMPTY statements push `Null`, return, and re-enter
    // per `;`, so a guard inside the chain helper would see at most two
    // operands while the spine grows without bound — 10 000 consecutive `;`
    // abort the process with a stack overflow without this one.
    let input = format!("a{}", " ;".repeat(10_000));

    let error =
        parse_expr(&input).expect_err("empty-statement spine should exceed the statement cap");
    assert!(matches!(
        error,
        SyntaxError::StatementChainTooLong {
            max: MAX_STATEMENT_CHAIN_DEPTH,
            ..
        }
    ));
}

#[test]
fn independent_statement_spines_do_not_accumulate() {
    parse_expr("f[a; b]; g[c; d]").expect("short independent spines parse");

    // Two bracketed spines, each well under the cap, inside one outer
    // spine: the link count is local to one climbing loop, so they must not
    // add up (together they would exceed the cap if it were shared).
    fn bracketed_spine(head: &str, count: usize) -> String {
        format!(
            "{head}[{}]",
            (0..count)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        )
    }

    let input = format!(
        "{}; {}",
        bracketed_spine("f", 200),
        bracketed_spine("g", 200)
    );
    parse_expr(&input).expect("independent spines must not accumulate");
}

#[test]
fn parses_empty_mid_chain_statement_as_locally_left_leaning() {
    // In the recursive parser, the `Null` produced for an empty statement
    // returned to the outer climbing loop, which then consumed the next `;`:
    // `a; ; b` parsed as `(a; Null); b`. The iterative chain collector keeps
    // that exact shape. (`a;;b` is not this case — `;;` lexes as one Span.)
    let parsed = parse_expr("a; ; b").unwrap();

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::CompoundExpression,
            SyntaxExpr::binary(
                BinaryOp::CompoundExpression,
                SyntaxExpr::symbol("a", z()),
                SyntaxExpr::symbol("Null", z()),
                z(),
            ),
            SyntaxExpr::symbol("b", z()),
            z(),
        )
    );
}

#[test]
fn parses_apply_and_composition_operator_precedence() {
    let parsed = parse_expr("f @* g @ x").unwrap();
    assert_eq!(parsed.to_string(), "f @* g @ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::binary(
                BinaryOp::Composition,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("g", z()),
                z(),
            ),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );

    let right = parse_expr("f /* g @ x").unwrap();
    assert_eq!(right.to_string(), "f /* g @ x");
    assert_eq!(
        right,
        SyntaxExpr::binary(
            BinaryOp::Apply,
            SyntaxExpr::binary(
                BinaryOp::RightComposition,
                SyntaxExpr::symbol("f", z()),
                SyntaxExpr::symbol("g", z()),
                z(),
            ),
            SyntaxExpr::symbol("x", z()),
            z(),
        )
    );
}

#[test]
fn parses_apply_level_operators_as_right_associative() {
    let parsed = parse_expr("f /@ g @@ h @@@ x").unwrap();
    assert_eq!(parsed.to_string(), "f /@ g @@ h @@@ x");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Map,
            SyntaxExpr::symbol("f", z()),
            SyntaxExpr::binary(
                BinaryOp::ApplyHead,
                SyntaxExpr::symbol("g", z()),
                SyntaxExpr::binary(
                    BinaryOp::MapApply,
                    SyntaxExpr::symbol("h", z()),
                    SyntaxExpr::symbol("x", z()),
                    z(),
                ),
                z(),
            ),
            z(),
        )
    );
}

#[test]
fn parses_function_postfix_as_map_lhs() {
    let parsed = parse_expr("typeSubstImpl[theta, #] & /@ args").unwrap();
    assert_eq!(parsed.to_string(), "(typeSubstImpl[theta, #] &) /@ args");

    assert_eq!(
        parsed,
        SyntaxExpr::binary(
            BinaryOp::Map,
            SyntaxExpr::postfix(
                PostfixOp::Function,
                SyntaxExpr::call(
                    SyntaxExpr::symbol("typeSubstImpl", z()),
                    vec![
                        SyntaxExpr::symbol("theta", z()),
                        SyntaxExpr::slot(SlotSpec::Anonymous, z())
                    ],
                    z(),
                ),
                z(),
            ),
            SyntaxExpr::symbol("args", z()),
            z(),
        )
    );
}

#[test]
fn parses_function_postfix_on_set_rhs_before_map() {
    let parsed = parse_expr("ks = #[[1]] & /@ args").unwrap();
    assert_eq!(parsed.to_string(), "ks = (Part[#, 1] &) /@ args");
}
