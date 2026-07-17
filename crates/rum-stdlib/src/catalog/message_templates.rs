use rum_runtime::kernel_api::BuiltinMessageSpec;

pub(crate) const ARGFORM_ITERATOR: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argform",
    template: "expected a supported repetition count or iterator specification",
};

pub(crate) const ARGTU: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argtu",
    template: "expected between `1` and `2` argument(s), got `3`",
};

pub(crate) const ARGX_AT_LEAST_TWO: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argx",
    template: "expected at least 2 argument(s), got `1`",
};

pub(crate) const ARGX_EXACT: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argx",
    template: "expected exactly `1` argument(s), got `2`",
};

pub(crate) const ARGX_OPERATOR: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argx",
    template: "`1` called with `2` arguments; 1 argument is expected.",
};

pub(crate) const BYTEARRAY: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "bytearray",
    template: "invalid ByteArray payload",
};

#[cfg(feature = "hash")]
pub(crate) const DIGEST_FORMAT: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "format",
    template: "unknown digest output format; use \"Hex\" (default) or \"Bytes\"",
};

pub(crate) const ENTROPY: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "entropy",
    template: "could not read entropy: `1`",
};

pub(crate) const IBASE: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "ibase",
    template: "Base `1` is not an integer greater than 1 in `2`.",
};

pub(crate) const INNF: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "innf",
    template: "Non-negative integer or Infinity expected at position `1` in `2`.",
};

pub(crate) const LEVEL: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "level",
    template: "Level specification `1` is not of the form n, {n} or {m, n}.",
};

pub(crate) const OPTRS: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "optrs",
    template: "Option specification `1` in `2` is not a rule for a symbol or string.",
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
