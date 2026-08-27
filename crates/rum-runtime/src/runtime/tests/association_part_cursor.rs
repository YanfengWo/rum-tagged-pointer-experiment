use super::*;

fn key_selector(key: Expr) -> Expr {
    Expr::call(system_expr("Key"), vec![key])
}

fn part_value(session: &mut Session, target: &Expr, selector: Expr) -> Expr {
    let args = SharedArgs::new(vec![target.clone(), selector]);
    session
        .eval_part(&system_expr("Part"), &args)
        .expect("Part selection should not fail evaluation")
}

fn integer_association(count: usize) -> Expr {
    render_association(
        (1..=count)
            .map(|value| {
                let value = Expr::small_integer(value as i64);
                (value.clone(), value)
            })
            .collect(),
    )
}

#[test]
fn duplicate_association_keys_keep_first_position_and_last_value() {
    let a = Expr::symbol("a");
    let b = Expr::symbol("b");
    let association = render_association(vec![
        (a.clone(), Expr::small_integer(1)),
        (b.clone(), Expr::small_integer(3)),
        (a.clone(), Expr::small_integer(2)),
    ]);
    assert_eq!(
        association.to_string(),
        "Association[Rule[a, 2], Rule[b, 3]]"
    );

    let mut session = Session::empty();
    assert_eq!(
        part_value(&mut session, &association, key_selector(a)),
        Expr::small_integer(2)
    );
    assert_eq!(
        part_value(&mut session, &association, Expr::small_integer(1)),
        Expr::small_integer(2)
    );
    assert_eq!(
        part_value(&mut session, &association, Expr::small_integer(2)),
        Expr::small_integer(3)
    );
}

#[test]
fn wide_association_single_key_and_position_reads_return_exact_values() {
    let association = integer_association(100_000);
    let mut session = Session::empty();

    assert_eq!(
        part_value(
            &mut session,
            &association,
            key_selector(Expr::small_integer(99_999)),
        ),
        Expr::small_integer(99_999)
    );
    assert_eq!(
        part_value(&mut session, &association, Expr::small_integer(100_000),),
        Expr::small_integer(100_000)
    );
}

#[test]
fn part_reads_reuse_the_lookup_cache_and_observe_cow_writes() {
    let mut association = integer_association(64);
    let mut session = Session::empty();
    let key = Expr::small_integer(1);

    for _ in 0..2 {
        assert_eq!(
            part_value(&mut session, &association, key_selector(key.clone())),
            key
        );
    }
    assert_eq!(session.association_lookup_work().index_builds, 1);

    assert_eq!(
        part_value(&mut session, &association, Expr::small_integer(3)),
        Expr::small_integer(3)
    );

    let Expr::Call { args, .. } = &mut association else {
        unreachable!("rendered associations are calls");
    };
    args.make_mut()[0] = Expr::call(
        system_expr("Rule"),
        vec![key.clone(), Expr::small_integer(99)],
    );

    assert_eq!(
        part_value(&mut session, &association, key_selector(key.clone())),
        Expr::small_integer(99)
    );
    assert_eq!(
        part_value(&mut session, &association, key_selector(key)),
        Expr::small_integer(99)
    );
    assert_eq!(session.association_lookup_work().index_builds, 2);
}

#[test]
fn below_threshold_part_reads_stay_on_the_linear_scan_path() {
    let association = integer_association(ASSOCIATION_INDEX_THRESHOLD);
    let mut session = Session::empty();

    for _ in 0..3 {
        assert_eq!(
            part_value(
                &mut session,
                &association,
                key_selector(Expr::small_integer(1)),
            ),
            Expr::small_integer(1)
        );
    }
    assert_eq!(
        part_value(&mut session, &association, Expr::small_integer(-1)),
        Expr::small_integer(ASSOCIATION_INDEX_THRESHOLD as i64)
    );
    assert_eq!(
        part_value(
            &mut session,
            &association,
            key_selector(Expr::small_integer(99)),
        )
        .to_string(),
        "Missing[\"KeyAbsent\", Key[99]]"
    );

    let work = session.association_lookup_work();
    assert_eq!(work.index_builds, 0);
    assert_eq!(work.indexed_probes, 0);
    assert!(work.linear_comparisons > 0);
}
