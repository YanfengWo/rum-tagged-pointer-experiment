use super::*;

fn integer(value: usize) -> Expr {
    Expr::small_integer(value as i64)
}

fn integer_association(count: usize) -> Expr {
    render_association(
        (1..=count)
            .map(|value| (integer(value), integer(value)))
            .collect(),
    )
}

#[test]
fn repeated_wide_association_reads_build_one_linear_index() {
    let count = 2_000usize;
    let association = integer_association(count);
    let mut session = Session::empty();

    for key in 1..=count {
        assert_eq!(
            session.apply_association_to_args(&association, &[integer(key)]),
            Some(integer(key))
        );
    }

    let work = session.association_lookup_work();
    let uncached_comparisons = count * (count + 1) / 2;
    assert_eq!(work.linear_comparisons, count);
    assert_eq!(work.index_builds, 1);
    assert_eq!(work.indexed_keys, count);
    assert_eq!(work.indexed_probes, count - 1);
    assert!(work.indexed_comparisons <= count * 2);
    assert!(work.linear_comparisons + work.indexed_comparisons < uncached_comparisons / 100);
}

#[test]
fn association_lookup_index_weak_identity_survives_cow_updates() {
    let mut association = integer_association(64);
    let mut session = Session::empty();

    assert_eq!(
        session.apply_association_to_args(&association, &[integer(1)]),
        Some(integer(1))
    );
    assert_eq!(
        session.apply_association_to_args(&association, &[integer(1)]),
        Some(integer(1))
    );

    let Expr::Call { args, .. } = &mut association else {
        unreachable!("rendered associations are calls");
    };
    args.make_mut()[0] = Expr::call(
        system_expr("Rule"),
        vec![integer(1), Expr::small_integer(99)],
    );

    assert_eq!(
        session.apply_association_to_args(&association, &[integer(1)]),
        Some(Expr::small_integer(99))
    );
    assert_eq!(session.association_lookup_work().index_builds, 1);
    assert_eq!(
        session.apply_association_to_args(&association, &[integer(1)]),
        Some(Expr::small_integer(99))
    );
    assert_eq!(session.association_lookup_work().index_builds, 2);
}
