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
fn lowering_golden_forms_cover_three_link_left_chains() {
    let cases = [
        ("a + b + c + d", "Plus[Plus[Plus[a, b], c], d]"),
        (
            "a - b - c - d",
            "Plus[Plus[Plus[a, Times[-1, b]], Times[-1, c]], Times[-1, d]]",
        ),
        ("a * b * c * d", "Times[Times[Times[a, b], c], d]"),
        (
            "a / b / c / d",
            "Times[Times[Times[a, Power[b, -1]], Power[c, -1]], Power[d, -1]]",
        ),
        ("8 / 2 / 4 / 2", "1/2"),
        ("a . b . c . d", "Dot[Dot[Dot[a, b], c], d]"),
        (
            "a ** b ** c ** d",
            "NonCommutativeMultiply[NonCommutativeMultiply[NonCommutativeMultiply[a, b], c], d]",
        ),
        (
            "a <> b <> c <> d",
            "StringJoin[StringJoin[StringJoin[a, b], c], d]",
        ),
        ("a::b::c::d", "a::b::c::d"),
        (
            "a /. b /. c /. d",
            "ReplaceAll[ReplaceAll[ReplaceAll[a, b], c], d]",
        ),
        (
            "a //. b //. c //. d",
            "ReplaceRepeated[ReplaceRepeated[ReplaceRepeated[a, b], c], d]",
        ),
        ("a : b : c : d", "Optional[Optional[Pattern[a, b], c], d]"),
        (
            "a | b | c | d",
            "Alternatives[Alternatives[Alternatives[a, b], c], d]",
        ),
        (
            "a ? b ? c ? d",
            "PatternTest[PatternTest[PatternTest[a, b], c], d]",
        ),
        (
            "a ~~ b ~~ c ~~ d",
            "StringExpression[StringExpression[StringExpression[a, b], c], d]",
        ),
        ("a // b // c // d", "d[c[b[a]]]"),
        (
            "a /* b /* c /* d",
            "RightComposition[RightComposition[RightComposition[a, b], c], d]",
        ),
    ];

    for (source, expected) in cases {
        assert_lowered_display(source, expected);
    }
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
        ("a ^= b", "UpSet[a, b]"),
        ("a ^:= b", "UpSetDelayed[a, b]"),
        ("a . b . c", "Dot[Dot[a, b], c]"),
        ("a . b * c", "Times[Dot[a, b], c]"),
        ("a . b ^ c", "Dot[a, Power[b, c]]"),
        ("a ** b * c", "Times[NonCommutativeMultiply[a, b], c]"),
        ("a ** b ^ c", "NonCommutativeMultiply[a, Power[b, c]]"),
        ("f /@ expr", "Map[f, expr]"),
        ("f //@ expr", "MapAll[f, expr]"),
        ("f @@ expr", "Apply[f, expr]"),
        ("f @@@ expr", "MapApply[f, expr]"),
        ("x |-> a + b", "Function[x, Plus[a, b]]"),
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
        // Parentheses are transparent to comparison-chain collection. These four
        // rows record that as it stands today; they are not the intended
        // semantics. `1 == (1 == 1)` yields True, while the `1 == True` it
        // should reduce to stays unevaluated — rum disagrees with itself here.
        // The right-hand cases are reachable: a left-associative chain only
        // extends through its lhs, so a comparison on the rhs came from explicit
        // grouping. The left-hand ones are not distinguishable after parsing,
        // since `(a == b) == c` and `a == b == c` share one AST.
        ("(a == b) == c", "Equal[a, b, c]"),
        ("a == (b == c)", "Equal[a, b, c]"),
        ("(a < b) <= c", "And[Less[a, b], LessEqual[b, c]]"),
        ("a < (b <= c)", "And[Less[a, b], LessEqual[b, c]]"),
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
        ("a ~~ b <> c", "StringExpression[a, StringJoin[b, c]]"),
        ("a ~ g ~ b", "g[a, b]"),
        ("a ~ g ~ b + c", "Plus[g[a, b], c]"),
        ("x ~ f ~ y ~ g ~ z", "g[f[x, y], z]"),
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
        ("a!", "Factorial[a]"),
        ("a!!", "Factorial2[a]"),
        ("2 a!", "Times[2, Factorial[a]]"),
        ("a!^2", "Power[Factorial[a], 2]"),
        ("f''", "Derivative[2][f]"),
        ("2 f'", "Times[2, Derivative[1][f]]"),
        ("{a ..}", "List[Repeated[a]]"),
        ("{a ...}", "List[RepeatedNull[a]]"),
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
