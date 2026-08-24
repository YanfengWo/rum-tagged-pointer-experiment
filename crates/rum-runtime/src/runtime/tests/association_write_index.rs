use super::*;

fn association_with_integer_keys(keys: impl IntoIterator<Item = i64>) -> Expr {
    Expr::call(
        system_expr("Association"),
        keys.into_iter()
            .map(|key| {
                Expr::call(
                    system_expr("Rule"),
                    vec![Expr::small_integer(key), Expr::small_integer(key)],
                )
            })
            .collect(),
    )
}

fn install_association(
    session: &mut Session,
    name: &str,
    keys: impl IntoIterator<Item = i64>,
) -> Symbol {
    let target = Symbol::new(GLOBAL_CONTEXT, name);
    session.definitions.entry(target).or_default().own_value =
        Some(association_with_integer_keys(keys));
    target
}

fn current_own_value(session: &Session, target: &Symbol) -> Expr {
    session
        .definitions
        .get(target)
        .and_then(|definition| definition.own_value.clone())
        .expect("the target must hold an Association own value")
}

fn own_args_ptr_key(session: &Session, target: &Symbol) -> usize {
    match &current_own_value(session, target) {
        Expr::Call { head, args } if is_system_symbol_name(head, "Association") => args.ptr_key(),
        _ => panic!("the target must store an Association"),
    }
}

fn seed_direct_write_proof(session: &mut Session, target: &Symbol) {
    // Seed the proof exactly the way the evaluator's own stability check
    // would, then let the clone go so the only strong holders are the
    // definition and the proof the write path takes before mutating.
    let current = current_own_value(session, target);
    assert!(session.cached_expr_is_evaluation_stable(&current));
}

fn read_two_keys_to_build_the_index(session: &mut Session, target: &Symbol) {
    // Two wide reads attach the read-side index to the target's args
    // allocation; the scope drops the borrowed clone so the writes below
    // face no strong holders other than the definition and the proof the
    // write path takes.
    let current = current_own_value(session, target);
    for key in [1_i64, 2] {
        assert_eq!(
            session.apply_association_to_args(&current, &[Expr::small_integer(key)]),
            Some(Expr::small_integer(key))
        );
    }
}

/// The stale window the design brief named: a direct write that appends in
/// place while the read side's index is attached must record the new key, or
/// the next indexed read reports it `Missing`.
#[test]
fn appended_keys_after_a_wide_read_stay_readable() {
    let mut session = session_with_test_control_builtins();
    let target = install_association(&mut session, "appendedAssoc", 1..=32);
    seed_direct_write_proof(&mut session, &target);
    read_two_keys_to_build_the_index(&mut session, &target);

    assert_eq!(session.association_lookup_work().index_builds, 1);
    let args_before = own_args_ptr_key(&session, &target);

    let script = (1..=8_i64)
        .map(|value| format!("appendedAssoc[1{value:02}] = {value}"))
        .collect::<Vec<_>>()
        .join("; ");
    session.eval_text(&script).unwrap();

    assert_eq!(
        own_args_ptr_key(&session, &target),
        args_before,
        "the appends must stay on the indexed allocation"
    );
    let work = session.association_lookup_work();
    assert_eq!(
        work.index_builds, 1,
        "the appends must maintain the attached index, not detach it"
    );
    assert_eq!(
        work.indexed_probes, 9,
        "one probe for the index-building read plus one per appended write"
    );

    let after = current_own_value(&session, &target);
    for key in 101_i64..=108 {
        assert_eq!(
            session.apply_association_to_args(&after, &[Expr::small_integer(key)]),
            Some(Expr::small_integer(key - 100)),
            "appended key {key} must read back its value, not Missing"
        );
    }
}

/// The duplicate half of the same window: without append recording, a second
/// write of a freshly appended key would probe as absent and append again.
#[test]
fn rewriting_an_appended_key_replaces_it_through_the_index() {
    let mut session = session_with_test_control_builtins();
    let target = install_association(&mut session, "rewriteAssoc", 1..=32);
    seed_direct_write_proof(&mut session, &target);
    read_two_keys_to_build_the_index(&mut session, &target);

    session
        .eval_text("rewriteAssoc[101] = 1; rewriteAssoc[101] = 2; rewriteAssoc[101] = 3")
        .unwrap();

    let current = current_own_value(&session, &target);
    let Expr::Call { args, .. } = &current else {
        panic!("the target must store an Association");
    };
    assert_eq!(args.len(), 33, "rewrites must replace, never duplicate");
    let occurrences = args
        .iter()
        .filter(|entry| match entry {
            Expr::Call { head, args, .. } if is_system_symbol_name(head, "Rule") => {
                args.len() == 2 && args[0] == Expr::small_integer(101)
            }
            _ => false,
        })
        .count();
    assert_eq!(occurrences, 1);
    assert_eq!(
        session.apply_association_to_args(&current, &[Expr::small_integer(101)]),
        Some(Expr::small_integer(3))
    );
}

/// Replacement locates the key through the index every time and never
/// rebuilds it — the key-to-position map is unchanged by a value replacement.
#[test]
fn repeated_replacement_writes_stay_indexed() {
    let mut session = session_with_test_control_builtins();
    let target = install_association(&mut session, "replaceAssoc", 1..=32);
    seed_direct_write_proof(&mut session, &target);
    read_two_keys_to_build_the_index(&mut session, &target);
    let args_before = own_args_ptr_key(&session, &target);
    let before = session.association_lookup_work();

    session
        .eval_text("Do[replaceAssoc[5] = i, {i, 1, 64}]")
        .unwrap();

    assert_eq!(own_args_ptr_key(&session, &target), args_before);
    let after = session.association_lookup_work();
    assert_eq!(after.index_builds, before.index_builds);
    assert_eq!(after.indexed_probes - before.indexed_probes, 64);

    let current = current_own_value(&session, &target);
    let Expr::Call { args, .. } = &current else {
        panic!("the target must store an Association");
    };
    assert_eq!(args.len(), 32);
    assert_eq!(
        session.apply_association_to_args(&current, &[Expr::small_integer(5)]),
        Some(Expr::small_integer(64))
    );
}

/// At or below the index threshold the write path keeps the plain scan and
/// must not add lookup-cache traffic to small direct writes; crossing the
/// threshold from below is what admits an index.
#[test]
fn small_association_writes_bypass_the_lookup_cache() {
    let mut session = session_with_test_control_builtins();
    let target = install_association(&mut session, "smallAssoc", 1..=8);
    seed_direct_write_proof(&mut session, &target);
    let before = session.association_lookup_work();

    session
        .eval_text("Do[smallAssoc[i] = 99, {i, 1, 8}]")
        .unwrap();

    assert_eq!(
        session.association_lookup_work(),
        before,
        "small direct writes must not touch the lookup cache"
    );

    let current = current_own_value(&session, &target);
    let Expr::Call { args, .. } = &current else {
        panic!("the target must store an Association");
    };
    assert_eq!(args.len(), 8, "replacements must not append");
    assert_eq!(
        session.apply_association_to_args(&current, &[Expr::small_integer(1)]),
        Some(Expr::small_integer(99))
    );
}
