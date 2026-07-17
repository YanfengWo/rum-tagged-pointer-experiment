use super::BuiltinMessageSpec;

pub(super) const ARGTU: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argtu",
    template: "expected between `1` and `2` argument(s), got `3`",
};

pub(super) const ARGX_EXACT: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "argx",
    template: "expected exactly `1` argument(s), got `2`",
};

pub(super) const INNF: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "innf",
    template: "Non-negative integer or Infinity expected at position `1` in `2`.",
};

pub(super) const LEVEL: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "level",
    template: "Level specification `1` is not of the form n, {n} or {m, n}.",
};

pub(super) const NOSYM: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "nosym",
    template: "No valid tag symbol occurs in the assignment lhs `1`.",
};

pub(super) const OPTRS: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "optrs",
    template: "Option specification `1` in `2` is not a rule for a symbol or string.",
};

pub(super) const OPTTF: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "opttf",
    template: "Value of option `1` -> `2` should be True or False.",
};

pub(super) const PARTD: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "partd",
    template: "Part specification `1` is longer than depth of object.",
};

pub(super) const PARTW: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "partw",
    template: "Part `1` of `2` does not exist.",
};

pub(super) const PKSPEC1: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "pkspec1",
    template: "The expression `1` cannot be used as a part specification.",
};

pub(super) const PSPEC1: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "pspec1",
    template: "Part specification `1` is not applicable.",
};

pub(super) const REPEATED_RANGE: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "range",
    template: "`1` is not a valid repetition bound specification; expected n, {n}, or {min, max} with 0 <= min <= max, where max may be Infinity.",
};

pub(super) const RVALUE: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "rvalue",
    template: "`1` is not a variable with a value, so its value cannot be changed.",
};

pub(super) const TAGNF: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "tagnf",
    template: "tag symbol does not occur in the assignment lhs",
};

pub(super) const TAGPOS: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "tagpos",
    template: "tag symbol must appear within the outer two levels of the assignment lhs",
};

pub(super) const WRSYM: BuiltinMessageSpec = BuiltinMessageSpec {
    tag: "wrsym",
    template: "Symbol `1` is Protected.",
};
