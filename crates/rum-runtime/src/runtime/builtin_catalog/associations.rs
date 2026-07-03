use super::*;

pub(super) const ASSOCIATION_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Association",
        "Builds an association from Rule/RuleDelayed entries in the current first-pass subset.",
        Session::eval_association,
    ),
    BuiltinSpec::ordinary(
        "AssociationQ",
        "Tests whether an expression is an Association with rule entries.",
        Session::eval_association_q,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Lookup",
        "Looks up a key in an association in the current first-pass subset.",
        HoldAttribute::HoldAll,
        Session::eval_lookup,
    ),
    BuiltinSpec::ordinary(
        "Key",
        "Represents a keyed selector in the current first-pass association subset.",
        Session::eval_key,
    ),
    BuiltinSpec::ordinary(
        "Keys",
        "Returns association keys in structural order.",
        Session::eval_keys,
    ),
    BuiltinSpec::ordinary(
        "Values",
        "Returns association values in structural order.",
        Session::eval_values,
    ),
    BuiltinSpec::ordinary(
        "KeyExistsQ",
        "Tests whether an association contains a given key in the current first-pass subset.",
        Session::eval_key_exists_q,
    ),
    BuiltinSpec::ordinary(
        "KeyTake",
        "Keeps selected keys from an association in the current first-pass subset.",
        Session::eval_key_take,
    ),
    BuiltinSpec::ordinary(
        "KeyDrop",
        "Drops selected keys from an association in the current first-pass subset.",
        Session::eval_key_drop,
    ),
    BuiltinSpec::ordinary(
        "KeyMap",
        "Maps a function over association keys in the current first-pass subset.",
        Session::eval_key_map,
    ),
    BuiltinSpec::ordinary(
        "KeyValueMap",
        "Maps a binary function over association key-value pairs in the current first-pass subset.",
        Session::eval_key_value_map,
    ),
    BuiltinSpec::ordinary(
        "KeySelect",
        "Selects association entries whose keys satisfy a criterion in the current first-pass subset.",
        Session::eval_key_select,
    ),
    BuiltinSpec::ordinary(
        "KeyMemberQ",
        "Tests whether an association contains a key in the current first-pass subset.",
        Session::eval_key_member_q,
    ),
    BuiltinSpec::ordinary(
        "KeyFreeQ",
        "Tests whether an association does not contain a key in the current first-pass subset.",
        Session::eval_key_free_q,
    ),
    BuiltinSpec::ordinary(
        "KeySort",
        "Sorts association entries by key in the current canonical ordering subset.",
        Session::eval_key_sort,
    ),
    BuiltinSpec::ordinary(
        "KeySortBy",
        "Sorts association entries by transformed keys in the current first-pass subset.",
        Session::eval_key_sort_by,
    ),
    BuiltinSpec::ordinary(
        "AssociationMap",
        "Builds an association by mapping a function over keys in the current first-pass subset.",
        Session::eval_association_map,
    ),
    BuiltinSpec::ordinary(
        "AssociationThread",
        "Threads rule or key/value inputs into an association in the current first-pass subset.",
        Session::eval_association_thread,
    ),
    BuiltinSpec::ordinary(
        "KeyUnion",
        "Builds keyed unions, including the current missing-key fill-function form.",
        Session::eval_key_union,
    ),
    BuiltinSpec::ordinary(
        "KeyIntersection",
        "Builds a key-intersection association in the current first-pass subset.",
        Session::eval_key_intersection,
    ),
    BuiltinSpec::ordinary(
        "KeyComplement",
        "Builds a key-complement association in the current first-pass subset.",
        Session::eval_key_complement,
    ),
    BuiltinSpec::ordinary(
        "Normal",
        "Converts an association into a list of rules in the current first-pass subset.",
        Session::eval_normal,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AssociateTo",
        "Adds or replaces association entries, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        Session::eval_associate_to,
    ),
    BuiltinSpec::ordinary_with_hold(
        "KeyDropFrom",
        "Drops keys from an association, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        Session::eval_key_drop_from,
    ),
    BuiltinSpec::ordinary(
        "KeyValuePattern",
        "Represents an association key/value pattern specification in the current first-pass subset.",
        Session::eval_key_value_pattern,
    ),
    BuiltinSpec::ordinary(
        "Merge",
        "Merges associations by key using a combiner in the current first-pass subset.",
        Session::eval_merge,
    ),
    BuiltinSpec::ordinary(
        "Catenate",
        "Concatenates lists and association values into a list in the current first-pass subset.",
        Session::eval_catenate,
    ),
    BuiltinSpec::ordinary(
        "Counts",
        "Counts repeated elements and returns an association in the current first-pass subset.",
        Session::eval_counts,
    ),
    BuiltinSpec::ordinary(
        "CountsBy",
        "Counts transformed elements and returns an association in the current first-pass subset.",
        Session::eval_counts_by,
    ),
    BuiltinSpec::ordinary(
        "GroupBy",
        "Groups list elements by transformed keys, including the current reducer form.",
        Session::eval_group_by,
    ),
    BuiltinSpec::ordinary(
        "PositionIndex",
        "Builds an association from elements to 1-based position lists in the current first-pass subset.",
        Session::eval_position_index,
    ),
    BuiltinSpec::ordinary(
        "Missing",
        "Represents missing-data markers in the current first-pass subset.",
        Session::eval_missing,
    ),
    BuiltinSpec::ordinary(
        "DeleteMissing",
        "Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.",
        Session::eval_delete_missing,
    ),
    BuiltinSpec::ordinary(
        "MissingQ",
        "Tests whether an expression is a Missing object.",
        Session::eval_missing_q,
    ),
];
