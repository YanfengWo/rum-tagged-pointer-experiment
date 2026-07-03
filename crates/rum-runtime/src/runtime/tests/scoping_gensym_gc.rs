use super::*;

#[test]
fn own_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "own value",
        "ClearAll[gcOwn078]; Module[{x}, x = 7; gcOwn078 = Hold[x]; 0]",
        "First[gcOwn078]",
        "ClearAll[gcOwn078]",
    );
}

#[test]
fn delayed_own_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "delayed own value",
        "ClearAll[gcDelayedOwn078]; Module[{x}, x = 7; gcDelayedOwn078 := Hold[x]; 0]",
        "First[gcDelayedOwn078]",
        "ClearAll[gcDelayedOwn078]",
    );
}

#[test]
fn down_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "down value",
        "ClearAll[gcDown078]; Module[{x}, x = 7; gcDown078[] := Hold[x]; 0]",
        "First[gcDown078[]]",
        "ClearAll[gcDown078]",
    );
}

#[test]
fn up_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "up value",
        "ClearAll[gcUpTag078, gcUpHead078]; Module[{x}, x = 7; gcUpTag078 /: gcUpHead078[gcUpTag078[]] := Hold[x]; 0]",
        "First[gcUpHead078[gcUpTag078[]]]",
        "ClearAll[gcUpTag078, gcUpHead078]",
    );
}

#[test]
fn sub_value_escape_keeps_candidate_until_definition_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "sub value",
        "ClearAll[gcSub078]; Module[{x}, x = 7; gcSub078[a_][b_] := Hold[x]; 0]",
        "First[gcSub078[1][2]]",
        "ClearAll[gcSub078]",
    );
}

#[test]
fn option_root_escape_keeps_candidate_until_options_are_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "option root",
        "ClearAll[gcOptions078, gcOption078]; Module[{x}, x = 7; Options[gcOptions078] = {gcOption078 -> Hold[x]}; 0]",
        "First[OptionValue[gcOptions078, gcOption078]]",
        "ClearAll[gcOptions078, gcOption078]",
    );
}

#[test]
fn syntax_information_root_escape_keeps_candidate_until_syntax_info_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "syntax information root",
        "ClearAll[gcSyntax078]; Module[{x}, x = 7; SyntaxInformation[gcSyntax078] = {\"LocalVariables\" -> Hold[x]}; 0]",
        "First[\"LocalVariables\" /. SyntaxInformation[gcSyntax078]]",
        "ClearAll[gcSyntax078]",
    );
}

#[test]
fn message_root_escape_keeps_candidate_until_message_is_cleared() {
    assert_definition_escape_kept_then_reclaimed(
        "message root",
        "ClearAll[gcMessage078]; Module[{x}, x = 7; gcMessage078::usage = Hold[x]; 0]",
        "First[MessageDefinitions[gcMessage078][[1]][[2]]]",
        "ClearAll[gcMessage078]",
    );
}

#[test]
fn returned_expr_extra_root_keeps_candidate_definition_live() {
    let mut session = Session::new();

    let outcome = session.eval_text("Module[{x}, x = 7; Hold[x]]").unwrap();
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);

    session.collect_and_sweep_dead_gensyms(std::slice::from_ref(&outcome.value));

    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate)),
        "returned Hold[x$] must keep x$ live while it is an extra root"
    );

    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(
        candidates
            .iter()
            .all(|candidate| !session.definitions.contains_key(candidate)),
        "candidate should sweep once the returned value is no longer a root"
    );
}

#[test]
fn outcome_extra_roots_keep_success_return_throw_and_cleanup_candidates_live() {
    assert_outcome_roots_keep_then_drop_candidates("Ok", 1, |exprs| Ok(exprs[0].clone()));
    assert_outcome_roots_keep_then_drop_candidates("Return", 1, |exprs| {
        Err(EvalError::Return {
            value: exprs[0].clone(),
        })
    });
    assert_outcome_roots_keep_then_drop_candidates("Thrown", 2, |exprs| {
        Err(EvalError::Thrown {
            value: exprs[0].clone(),
            tag: exprs[1].clone(),
            explicit_tag: true,
        })
    });
    assert_outcome_roots_keep_then_drop_candidates("ThrownBypassingCatch", 4, |exprs| {
        Err(EvalError::ThrownBypassingCatch {
            value: exprs[0].clone(),
            tag: exprs[1].clone(),
            explicit_tag: true,
            cleanup_value: exprs[2].clone(),
            cleanup_tag: exprs[3].clone(),
            cleanup_explicit_tag: true,
        })
    });
}

#[test]
fn outcome_extra_roots_are_empty_for_non_value_errors_and_candidate_is_reclaimed() {
    let mut session = Session::new();
    let candidate = allocate_retained_candidate(&mut session, 1);
    let outcome = Err(EvalError::Cancelled);
    let roots = gensym_gc_extra_roots_for_outcome(&outcome);

    assert!(roots.is_empty());
    assert_candidate_definition_count(&session, &[candidate], 1);

    session.collect_and_sweep_dead_gensyms(&roots);

    assert_candidate_definition_count(&session, &[candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&candidate));
}

#[test]
fn global_root_keeps_escaped_candidate_until_root_is_cleared() {
    let mut session = Session::new();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcRoot050]; Module[{x}, x = 7; gcRoot050 = Hold[x]; 0]"
        ),
        "0"
    );
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, "First[gcRoot050]"), "7");
    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate))
    );

    session.eval_text("Clear[gcRoot050]").unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(
        candidates
            .iter()
            .all(|candidate| !session.definitions.contains_key(candidate))
    );
}

#[test]
fn transitive_definition_kind_chain_keeps_all_reachable_candidates_live() {
    let mut session = Session::new();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcChainRoot078, gcChainHead078]; Module[{a, b}, b = 11; a /: gcChainHead078[a] := b; gcChainRoot078[] := a; 0]"
        ),
        "0"
    );
    let candidates = retained_candidates(&session);
    assert_eq!(candidates.len(), 2);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(
        eval_string(&mut session, "gcChainHead078[gcChainRoot078[]]"),
        "11"
    );
    assert_candidate_definition_count(&session, &candidates, 2);

    session
        .eval_text("ClearAll[gcChainRoot078, gcChainHead078]")
        .unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &candidates, 0);
}

#[test]
fn live_candidate_definition_marks_transitive_candidate_dependencies() {
    let mut session = Session::new();

    assert_eq!(
        eval_string(
            &mut session,
            "ClearAll[gcTrans050]; Module[{a, b}, b = 11; a := b; gcTrans050 = Hold[a]; 0]"
        ),
        "0"
    );
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 2);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, "First[gcTrans050]"), "11");
    assert!(
        candidates
            .iter()
            .all(|candidate| session.definitions.contains_key(candidate)),
        "rooted candidate a$ should keep candidate b$ live through a$'s definition"
    );
}

#[test]
fn empty_candidate_set_sweep_is_a_noop() {
    let mut session = Session::new();

    session
        .eval_text("ClearAll[gcNoop078]; gcNoop078 = 17")
        .unwrap();
    assert!(session.gc_candidate_gensyms.is_empty());

    session.collect_and_sweep_dead_gensyms(&[]);

    assert!(session.gc_candidate_gensyms.is_empty());
    assert_eq!(eval_string(&mut session, "gcNoop078"), "17");
}

#[test]
fn active_module_activation_keeps_candidate_until_activation_is_removed() {
    let mut session = Session::new();
    let candidate = session.allocate_fresh_symbol(&Symbol::from_name("gcActive078"));

    session.gc_candidate_gensyms.insert(candidate);
    session.definitions.entry(candidate).or_default().own_value = Some(Expr::small_integer(7));
    session.module_activations.push(ModuleActivation {
        gensyms: vec![candidate],
        escaped: false,
        side_table_stamp: session.current_side_table_stamp(),
    });

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[candidate], 1);
    assert!(session.gc_candidate_gensyms.contains(&candidate));

    session.module_activations.clear();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&candidate));
}

#[test]
fn attribute_state_on_dead_candidate_is_cleared_during_reclaim() {
    let mut session = Session::new();
    let candidate = allocate_retained_candidate(&mut session, 7);
    let mut attributes = session.attribute_set_for_symbol(&candidate);
    attributes.insert(Attribute::HoldFirst);
    session.set_attribute_set_for_symbol(candidate, attributes);

    assert!(session.attributes.contains_key(&candidate));

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &[candidate], 0);
    assert!(!session.gc_candidate_gensyms.contains(&candidate));
    assert!(!session.attributes.contains_key(&candidate));
}

#[test]
fn unique_generated_symbols_are_not_module_gc_candidates() {
    let mut session = Session::new();

    let generated = eval_string(&mut session, "Unique[gcUnique050]");
    assert_eq!(session.gc_candidate_gensyms.len(), 0);

    assert_eq!(
        eval_string(&mut session, &format!("{generated} = 17")),
        "17"
    );
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, &generated), "17");
    assert_eq!(session.gc_candidate_gensyms.len(), 0);
}

#[test]
fn gensym_gc_threshold_parses_env_in_fresh_processes() {
    if let Ok(subcase) = std::env::var("RUM_GENSYM_GC_THRESHOLD_SUBCASE") {
        match subcase.as_str() {
            "valid" => assert_eq!(gensym_gc_threshold(), 3),
            "invalid" | "zero" | "default" => assert_eq!(gensym_gc_threshold(), 200_000),
            other => panic!("unknown threshold subcase {other}"),
        }
        return;
    }

    let current_exe = std::env::current_exe().unwrap();
    let test_name = format!(
        "{}::gensym_gc_threshold_parses_env_in_fresh_processes",
        module_path!()
            .strip_prefix("rum_runtime::")
            .unwrap_or(module_path!())
    );
    for (subcase, threshold) in [
        ("valid", Some("3")),
        ("invalid", Some("not-a-number")),
        ("zero", Some("0")),
        ("default", None),
    ] {
        let mut command = std::process::Command::new(&current_exe);
        command
            .arg("--exact")
            .arg(&test_name)
            .arg("--test-threads=1")
            .env("RUM_GENSYM_GC_THRESHOLD_SUBCASE", subcase)
            .env_remove("RUM_GENSYM_GC_THRESHOLD");
        if let Some(threshold) = threshold {
            command.env("RUM_GENSYM_GC_THRESHOLD", threshold);
        }

        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "threshold subcase {subcase} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn eval_string(session: &mut Session, source: &str) -> String {
    session.eval_text(source).unwrap().value.to_string()
}

fn assert_definition_escape_kept_then_reclaimed(
    label: &str,
    setup_source: &str,
    verify_source: &str,
    clear_source: &str,
) {
    let mut session = Session::new();

    assert_eq!(eval_string(&mut session, setup_source), "0");
    let candidates = retained_candidates(&session);
    assert_eq!(candidates.len(), 1, "{label} should retain one candidate");

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_eq!(eval_string(&mut session, verify_source), "7");
    assert_candidate_definition_count(&session, &candidates, 1);

    session.eval_text(clear_source).unwrap();
    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &candidates, 0);
    assert!(
        candidates
            .iter()
            .all(|candidate| !session.gc_candidate_gensyms.contains(candidate)),
        "{label} candidate should be removed from the candidate set after sweep"
    );
}

fn assert_outcome_roots_keep_then_drop_candidates<F>(label: &str, count: usize, make_outcome: F)
where
    F: FnOnce(&[Expr]) -> Result<Expr, EvalError>,
{
    let mut session = Session::new();
    let candidates = (0..count)
        .map(|index| allocate_retained_candidate(&mut session, index as i64))
        .collect::<Vec<_>>();
    let exprs = candidates
        .iter()
        .copied()
        .map(symbol_expr)
        .collect::<Vec<_>>();
    let outcome = make_outcome(&exprs);
    let roots = gensym_gc_extra_roots_for_outcome(&outcome);

    assert_eq!(
        roots.len(),
        count,
        "{label} should expose every outcome value as an extra root"
    );
    assert_candidate_definition_count(&session, &candidates, count);

    session.collect_and_sweep_dead_gensyms(&roots);

    assert_candidate_definition_count(&session, &candidates, count);

    session.collect_and_sweep_dead_gensyms(&[]);

    assert_candidate_definition_count(&session, &candidates, 0);
    assert!(
        candidates
            .iter()
            .all(|candidate| !session.gc_candidate_gensyms.contains(candidate)),
        "{label} candidates should be removed from the candidate set after the extra roots drop"
    );
}

fn allocate_retained_candidate(session: &mut Session, value: i64) -> Symbol {
    let before = session.gc_candidate_gensyms.clone();
    let source = format!("Module[{{x}}, x = {value}; Hold[x]]");
    session.eval_text(&source).unwrap();
    let candidates = session
        .gc_candidate_gensyms
        .iter()
        .filter(|candidate| !before.contains(candidate))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(candidates.len(), 1);
    assert!(session.definitions.contains_key(&candidates[0]));
    candidates[0]
}

fn retained_candidates(session: &Session) -> Vec<Symbol> {
    session
        .gc_candidate_gensyms
        .iter()
        .copied()
        .collect::<Vec<_>>()
}

fn symbol_expr(symbol: Symbol) -> Expr {
    Expr::Atom(Atom::Symbol(symbol))
}

fn assert_candidate_definition_count(session: &Session, candidates: &[Symbol], count: usize) {
    assert_eq!(
        candidates
            .iter()
            .filter(|candidate| session.definitions.contains_key(candidate))
            .count(),
        count
    );
}
