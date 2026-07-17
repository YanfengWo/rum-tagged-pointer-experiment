use rum_runtime::kernel_api::BuiltinMessageSpec;

pub(crate) const ARGTU: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argtu",
    template: "expected between `1` and `2` argument(s), got `3`",
};

pub(crate) const ARGX_EXACT: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argx",
    template: "expected exactly `1` argument(s), got `2`",
};

pub(crate) const ARGX_OPERATOR: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argx",
    template: "`1` called with `2` arguments; 1 argument is expected.",
};

pub(crate) const OPTTF: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "opttf",
    template: "Value of option `1` -> `2` should be True or False.",
};

pub(crate) const PARTW: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "partw",
    template: "Part `1` of `2` does not exist.",
};

pub(crate) const RVALUE: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "rvalue",
    template: "`1` is not a variable with a value, so its value cannot be changed.",
};

pub(crate) const WRSYM: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "wrsym",
    template: "Symbol `1` is Protected.",
};
