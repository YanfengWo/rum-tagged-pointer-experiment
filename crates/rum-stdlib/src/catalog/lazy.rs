use rum_runtime::kernel_api::{
    Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec, HoldAttribute,
};

use crate::lazy;

pub(crate) const LAZY_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "LazyDrop",
        "Builds a lazy sequence that skips a non-negative number of source elements.",
        lazy::eval_lazy_drop,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a LazyList and a non-negative integer",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "LazyFoldList",
        "Builds a lazy sequence of successive accumulated values, including the initial value.",
        lazy::eval_lazy_fold_list,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 3 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a LazyList as the third argument",
        },
    ])
    .protected(),
    BuiltinSpec::with_attributes(
        "LazyList",
        "Represents an opaque pure lazy sequence descriptor.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
    )
    .protected(),
    BuiltinSpec::ordinary(
        "LazyMap",
        "Builds a lazy sequence that applies a function to each source element.",
        lazy::eval_lazy_map,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a LazyList as the second argument",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "LazyNestList",
        "Builds an infinite lazy sequence by repeatedly applying a function to an initial value.",
        lazy::eval_lazy_nest_list,
    )
    .with_messages(&[BuiltinMessageSpec {
        tag: "argx",
        template: "expected exactly 2 argument(s), got `1`",
    }])
    .protected(),
    BuiltinSpec::ordinary(
        "LazyRange",
        "Builds an infinite lazy arithmetic sequence from an optional start and step.",
        lazy::eval_lazy_range,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argtu",
            template: "expected between 0 and 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "start and step must be Integer or Rational values",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "LazySelect",
        "Builds a lazy sequence retaining source elements whose predicate returns True.",
        lazy::eval_lazy_select,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a LazyList as the second argument",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "LazySubsets",
        "Enumerates the subsets of a list lazily in size-major, index-lexicographic order.",
        lazy::eval_lazy_subsets,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 1 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a List",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary_with_hold(
        "LazyTable",
        "Builds a lazy arithmetic table whose held body evaluates once per realized element.",
        HoldAttribute::HoldAll,
        lazy::eval_lazy_table,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected an arithmetic iterator with a symbol, exact rational bounds, and a nonzero step",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "LazyTake",
        "Builds a finite lazy sequence capped by a non-negative element count.",
        lazy::eval_lazy_take,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected exactly 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a LazyList and a non-negative integer or UpTo[n]",
        },
    ])
    .protected(),
    BuiltinSpec::ordinary(
        "LazyTuples",
        "Enumerates repeated tuples or a list-of-lists cartesian product lazily with the last index varying fastest.",
        lazy::eval_lazy_tuples,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "expected 1 or 2 argument(s), got `1`",
        },
        BuiltinMessageSpec {
            tag: "argform",
            template: "expected a list of lists, or a list and a non-negative integer",
        },
    ])
    .protected(),
];
