use rum_runtime::kernel_api::{BuiltinSpec, HoldAttribute};

use crate::associations;

pub(crate) const ASSOCIATION_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Association",
        "Builds an association from Rule/RuleDelayed entries in the current first-pass subset.",
        associations::eval_association,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationQ",
        "Tests whether an expression is an Association with rule entries.",
        associations::eval_association_q,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationKeyDeflatten",
        "Builds nested associations from flat associations whose keys are path lists.",
        associations::eval_association_key_deflatten,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationKeyFlatten",
        "Flattens nested associations into associations whose keys are path lists.",
        associations::eval_association_key_flatten,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Lookup",
        "Looks up a key in an association in the current first-pass subset.",
        HoldAttribute::HoldAll,
        associations::eval_lookup,
    ).protected(),
    BuiltinSpec::ordinary(
        "Key",
        "Represents a keyed selector in the current first-pass association subset.",
        associations::eval_key,
    ).protected(),
    BuiltinSpec::ordinary(
        "Keys",
        "Returns association keys in structural order.",
        associations::eval_keys,
    ).protected(),
    BuiltinSpec::ordinary(
        "Values",
        "Returns association values in structural order.",
        associations::eval_values,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyExistsQ",
        "Tests whether an association contains a given key in the current first-pass subset.",
        associations::eval_key_exists_q,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyTake",
        "Keeps selected keys from an association in the current first-pass subset.",
        associations::eval_key_take,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyDrop",
        "Drops selected keys from an association in the current first-pass subset.",
        associations::eval_key_drop,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyMap",
        "Maps a function over association keys in the current first-pass subset.",
        associations::eval_key_map,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyValueMap",
        "Maps a binary function over association key-value pairs in the current first-pass subset.",
        associations::eval_key_value_map,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeySelect",
        "Selects association entries whose keys satisfy a criterion in the current first-pass subset.",
        associations::eval_key_select,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyMemberQ",
        "Tests whether an association contains a key in the current first-pass subset.",
        associations::eval_key_member_q,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyFreeQ",
        "Tests whether an association does not contain a key in the current first-pass subset.",
        associations::eval_key_free_q,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyGroupBy",
        "Groups association entries by transformed keys, including the current reducer form.",
        associations::eval_key_group_by,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeySort",
        "Sorts association entries by key in the current canonical ordering subset.",
        associations::eval_key_sort,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeySortBy",
        "Sorts association entries by transformed keys, optionally using a supplied binary ordering predicate.",
        associations::eval_key_sort_by,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationMap",
        "Builds an association by mapping a function over keys in the current first-pass subset.",
        associations::eval_association_map,
    ).protected(),
    BuiltinSpec::ordinary(
        "AssociationThread",
        "Threads rule or key/value inputs into an association in the current first-pass subset.",
        associations::eval_association_thread,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyUnion",
        "Builds keyed unions, including the current missing-key fill-function form.",
        associations::eval_key_union,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyIntersection",
        "Builds a key-intersection association in the current first-pass subset.",
        associations::eval_key_intersection,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyComplement",
        "Builds a key-complement association in the current first-pass subset.",
        associations::eval_key_complement,
    ).protected(),
    BuiltinSpec::ordinary(
        "Normal",
        "Converts an association into a list of rules in the current first-pass subset.",
        associations::eval_normal,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "AssociateTo",
        "Adds or replaces association entries, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        associations::eval_associate_to,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "KeyDropFrom",
        "Drops keys from an association, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        associations::eval_key_drop_from,
    ).protected(),
    BuiltinSpec::ordinary(
        "KeyValuePattern",
        "Represents an association key/value pattern specification in the current first-pass subset.",
        associations::eval_key_value_pattern,
    ).protected(),
    BuiltinSpec::ordinary(
        "Merge",
        "Merges associations by key using a combiner in the current first-pass subset.",
        associations::eval_merge,
    ).protected(),
    BuiltinSpec::ordinary(
        "MergeNested",
        "Recursively merges nested associations by key using a combiner at leaves.",
        associations::eval_merge_nested,
    ).protected(),
    BuiltinSpec::ordinary(
        "Catenate",
        "Concatenates lists and association values into a list in the current first-pass subset.",
        associations::eval_catenate,
    ).protected(),
    BuiltinSpec::ordinary(
        "Counts",
        "Counts repeated elements and returns an association in the current first-pass subset.",
        associations::eval_counts,
    ).protected(),
    BuiltinSpec::ordinary(
        "CountsBy",
        "Counts transformed elements and returns an association in the current first-pass subset.",
        associations::eval_counts_by,
    ).protected(),
    BuiltinSpec::ordinary(
        "GroupBy",
        "Groups list elements by transformed keys, including the current reducer form.",
        associations::eval_group_by,
    ).protected(),
    BuiltinSpec::ordinary(
        "PositionIndex",
        "Builds an association from elements to 1-based position lists in the current first-pass subset.",
        associations::eval_position_index,
    ).protected(),
    BuiltinSpec::ordinary(
        "Missing",
        "Represents missing-data markers in the current first-pass subset.",
        associations::eval_missing,
    ).protected(),
    BuiltinSpec::ordinary(
        "DeleteMissing",
        "Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.",
        associations::eval_delete_missing,
    ).protected(),
];
