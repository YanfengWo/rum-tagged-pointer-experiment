use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinMessageSpec, BuiltinSpec};

use crate::lazy;

pub(crate) const LAZY_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::with_attributes(
        "LazyList",
        "Represents an opaque pure lazy sequence descriptor.",
        AttributeSet::from_attribute(Attribute::HoldAllComplete),
    )
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
];
