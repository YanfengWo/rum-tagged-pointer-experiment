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
        ("<|a -> 1, b -> 2|>", "Association[Rule[a, 1], Rule[b, 2]]"),
        ("a; b;", "CompoundExpression[a, b, Null]"),
        ("f @ g @ x", "f[g[x]]"),
        ("x // f", "f[x]"),
        ("x // f // g", "g[f[x]]"),
        ("a + b // f", "f[Plus[a, b]]"),
        ("a = b // f", "Set[a, f[b]]"),
        ("a // f /. r", "ReplaceAll[f, r][a]"),
        ("f /@ expr", "Map[f, expr]"),
        ("f @@ expr", "Apply[f, expr]"),
        ("f @@@ expr", "MapApply[f, expr]"),
        ("f @* g @ x", "Composition[f, g][x]"),
        ("f /* g @ x", "RightComposition[f, g][x]"),
        (
            "! MatchQ[k, pat] && 0 <= i < depth",
            "And[Not[MatchQ[k, pat]], And[LessEqual[0, i], Less[i, depth]]]",
        ),
        ("a || b && c", "Or[a, And[b, c]]"),
        ("x === y === z", "SameQ[x, y, z]"),
        ("x =!= y =!= z", "UnsameQ[x, y, z]"),
        ("x == y != z", "And[Equal[x, y], Unequal[y, z]]"),
        (
            "typeSubstImpl[theta, #] & /@ args",
            "Map[Function[typeSubstImpl[theta, Slot[1]]], args]",
        ),
        (
            "ks = #[[1]] & /@ args",
            "Set[ks, Map[Function[Part[Slot[1], 1]], args]]",
        ),
        (
            "Map[instType[theta, #[[1]]] -> #[[2]] &, pairs]",
            "Map[Function[Rule[instType[theta, Part[Slot[1], 1]], Part[Slot[1], 2]]], pairs]",
        ),
        (
            "var[_, _] | const[_, _]",
            "Alternatives[var[Blank[], Blank[]], const[Blank[], Blank[]]]",
        ),
        ("_?propBoolAtomQ", "PatternTest[Blank[], propBoolAtomQ]"),
        ("\"a\" <> ToString[i]", "StringJoin[\"a\", ToString[i]]"),
        (
            "x_Integer := f[x]",
            "SetDelayed[Pattern[x, Blank[Integer]], f[x]]",
        ),
        (
            "typeSubst[theta : {___Rule}, ty_] := rhs",
            "SetDelayed[typeSubst[Pattern[theta, List[BlankNullSequence[Rule]]], Pattern[ty, Blank[]]], rhs]",
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
        ("-1", "-1"),
        ("-5", "-5"),
        ("-1/2", "-1/2"),
        ("Times[-1, b]", "Times[-1, b]"),
        ("-x", "Times[-1, x]"),
        ("-f[x]", "Times[-1, f[x]]"),
        ("-(a + b)", "Times[-1, Plus[a, b]]"),
        ("a / b", "Times[a, Power[b, -1]]"),
        ("x += 1", "AddTo[x, 1]"),
        ("x -= 1", "SubtractFrom[x, 1]"),
        ("x *= 2", "TimesBy[x, 2]"),
        ("x /= 2", "DivideBy[x, 2]"),
        ("x //= f", "ApplyTo[x, f]"),
        ("x++", "Increment[x]"),
        ("x--", "Decrement[x]"),
        ("++x", "PreIncrement[x]"),
        ("--x", "PreDecrement[x]"),
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
