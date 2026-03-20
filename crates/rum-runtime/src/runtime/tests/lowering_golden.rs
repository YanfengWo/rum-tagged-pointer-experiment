use super::*;

fn assert_lowered_display(source: &str, expected: &str) {
    let syntax = parse_expr(source).expect("golden lowering source should parse");
    let lowered = lower_syntax_expr(&syntax).expect("golden lowering source should lower");

    assert_eq!(
        lowered.to_string(),
        expected,
        "lowering mismatch for {source:?}"
    );
}

#[test]
fn lowering_golden_forms_cover_rules_patterns_slots_and_scoping() {
    let cases = [
        ("f[a, b]", "f[a, b]"),
        ("{a, b}", "List[a, b]"),
        (
            "x_Integer := f[x]",
            "SetDelayed[Pattern[x, Blank[Integer]], f[x]]",
        ),
        (
            "expr /. x_Integer -> y",
            "ReplaceAll[expr, Rule[Pattern[x, Blank[Integer]], y]]",
        ),
        (
            "expr //. x_Integer :> y",
            "ReplaceRepeated[expr, RuleDelayed[Pattern[x, Blank[Integer]], y]]",
        ),
        (
            "x_Integer /; test[x]",
            "Condition[Pattern[x, Blank[Integer]], test[x]]",
        ),
        ("#x &", "Function[Slot[\"x\"]]"),
        (
            "## + ##2 &",
            "Function[Plus[SlotSequence[1], SlotSequence[2]]]",
        ),
        ("a - b", "Plus[a, Times[-1, b]]"),
        ("a / b", "Times[a, Power[b, -1]]"),
        ("With[{x = 1}, x]", "With[List[Set[x, 1]], x]"),
        ("Module[{x = 1}, x]", "Module[List[Set[x, 1]], x]"),
        ("Block[{x = 1}, x]", "Block[List[Set[x, 1]], x]"),
    ];

    for (source, expected) in cases {
        assert_lowered_display(source, expected);
    }
}

#[test]
fn lowering_golden_forms_cover_messages_and_context_heads() {
    let syntax = parse_expr("FindFile::notfound").expect("message name source should parse");
    let lowered = lower_syntax_expr(&syntax).expect("message name source should lower");

    assert_eq!(
        lowered,
        Expr::call(
            system_expr(heads::messages::MESSAGE_NAME),
            vec![
                system_expr(heads::contexts::FIND_FILE),
                Expr::string("notfound"),
            ],
        )
    );

    let direct_head_cases = [
        ("BeginPackage[\"Demo`\"]", "BeginPackage[\"Demo`\"]"),
        ("Needs[\"Demo`\"]", "Needs[\"Demo`\"]"),
        ("Get[\"demo.rum\"]", "Get[\"demo.rum\"]"),
        ("Context[\"Identity\"]", "Context[\"Identity\"]"),
        ("File[\"demo.rum\"]", "File[\"demo.rum\"]"),
    ];

    for (source, expected) in direct_head_cases {
        assert_lowered_display(source, expected);
    }
}

#[test]
fn lowering_golden_forms_cover_tagged_definition_sugar() {
    let cases = [
        (
            "f /: g[x_] := rhs",
            "TagSetDelayed[f, g[Pattern[x, Blank[]]], rhs]",
        ),
        ("f /: g[x_] =.", "TagUnset[f, g[Pattern[x, Blank[]]]]"),
        ("f /: g[x_] = rhs", "TagSet[f, g[Pattern[x, Blank[]]], rhs]"),
    ];

    for (source, expected) in cases {
        assert_lowered_display(source, expected);
    }
}
