use rum_runtime::kernel_api::{BuiltinMessageSpec, BuiltinSpec, HoldAttribute};

use super::message_templates::*;

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
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "AssociationQ called with `1` arguments; 1 argument is expected.",
        },
    ]).protected(),
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
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "Lookup called with `1` arguments; between 2 and 4 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Key",
        "Represents a keyed selector in the current first-pass association subset.",
        associations::eval_key,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "Key called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Keys",
        "Returns association keys in structural order.",
        associations::eval_keys,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Keys called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Values",
        "Returns association values in structural order.",
        associations::eval_values,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Values called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyExistsQ",
        "Tests whether an association contains a given key in the current first-pass subset.",
        associations::eval_key_exists_q,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyExistsQ called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyTake",
        "Keeps selected keys from an association in the current first-pass subset.",
        associations::eval_key_take,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyTake called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "invru",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyDrop",
        "Drops selected keys from an association in the current first-pass subset.",
        associations::eval_key_drop,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyDrop called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "invru",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyMap",
        "Maps a function over association keys in the current first-pass subset.",
        associations::eval_key_map,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyMap called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyValueMap",
        "Maps a binary function over association key-value pairs in the current first-pass subset.",
        associations::eval_key_value_map,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyValueMap called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeySelect",
        "Selects association entries whose keys satisfy a criterion in the current first-pass subset.",
        associations::eval_key_select,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeySelect called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invru",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyMemberQ",
        "Tests whether an association contains a key in the current first-pass subset.",
        associations::eval_key_member_q,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argrx",
            template: "KeyMemberQ called with `1` arguments; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
        BuiltinMessageSpec {
            tag: "invrk",
            template: "The argument `1` is not a valid rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyFreeQ",
        "Tests whether an association does not contain a key in the current first-pass subset.",
        associations::eval_key_free_q,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argrx",
            template: "KeyFreeQ called with `1` arguments; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
        BuiltinMessageSpec {
            tag: "invrk",
            template: "The argument `1` is not a valid rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyGroupBy",
        "Groups association entries by transformed keys, including the current reducer form.",
        associations::eval_key_group_by,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeySort",
        "Sorts association entries by key in the current canonical ordering subset.",
        associations::eval_key_sort,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeySort called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeySortBy",
        "Sorts association entries by transformed keys, optionally using a supplied binary ordering predicate.",
        associations::eval_key_sort_by,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeySortBy called with `1` arguments; 1, 2, or 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "AssociationMap",
        "Builds an association by mapping a function over keys in the current first-pass subset.",
        associations::eval_association_map,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "AssociationMap called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrlf",
            template: "Applying `1` to `2` yields `3`, which is not a valid rule, list of rules or association.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "AssociationThread",
        "Threads rule or key/value inputs into an association in the current first-pass subset.",
        associations::eval_association_thread,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "AssociationThread called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "idim",
            template: "`1` and `2` must have the same length.",
        },
        BuiltinMessageSpec {
            tag: "invrk",
            template: "The argument `1` is not a valid rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyUnion",
        "Builds keyed unions, including the current missing-key fill-function form.",
        associations::eval_key_union,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "KeyUnion called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invar",
            template: "The argument `1` is not a valid list of Associations or rules.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyIntersection",
        "Builds a key-intersection association in the current first-pass subset.",
        associations::eval_key_intersection,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "KeyIntersection called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invar",
            template: "The argument `1` is not a valid list of Associations or rules.",
        },
        BuiltinMessageSpec {
            tag: "invas",
            template: "The argument `1` is not a valid Association or rule.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyComplement",
        "Builds a key-complement association in the current first-pass subset.",
        associations::eval_key_complement,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "KeyComplement called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "empt",
            template: "Argument `1` should be a nonempty list.",
        },
        BuiltinMessageSpec {
            tag: "invar",
            template: "The argument `1` is not a valid list of Associations or rules.",
        },
        BuiltinMessageSpec {
            tag: "invrl",
            template: "The argument `1` is not a valid Association or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "nonopt",
            template: "Options expected (instead of `1`) beyond position `2` in `3`. An option must be a rule or a list of rules.",
        },
        BuiltinMessageSpec {
            tag: "optx",
            template: "Unknown option `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Normal",
        "Converts an association into a list of rules in the current first-pass subset.",
        associations::eval_normal,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "Normal called with `1` arguments; between 1 and 3 arguments are expected.",
        },
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "AssociateTo",
        "Adds or replaces association entries, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        associations::eval_associate_to,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argr",
            template: "AssociateTo called with 1 argument; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "argrx",
            template: "AssociateTo called with `1` arguments; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "blnoval",
            template: "The symbol `1` at position `2` should have an immediate value defined.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
        BuiltinMessageSpec {
            tag: "invlb",
            template: "The argument `1` is not a list, Rule or Association.",
        },
        BuiltinMessageSpec {
            tag: "keynx",
            template: "Key `1` does not exist in `2`.",
        },
        PARTW,
        RVALUE,
        WRSYM,
    ]).protected(),
    BuiltinSpec::ordinary_with_hold(
        "KeyDropFrom",
        "Drops keys from an association, including held symbol mutation in the current first-pass subset.",
        HoldAttribute::HoldFirst,
        associations::eval_key_drop_from,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argr",
            template: "KeyDropFrom called with 1 argument; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "argrx",
            template: "KeyDropFrom called with `1` arguments; 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "blnoval",
            template: "The symbol `1` at position `2` should have an immediate value defined.",
        },
        BuiltinMessageSpec {
            tag: "invak",
            template: "The argument `1` is not a valid Association.",
        },
        RVALUE,
        WRSYM,
    ]).protected(),
    BuiltinSpec::ordinary(
        "KeyValuePattern",
        "Represents an association key/value pattern specification in the current first-pass subset.",
        associations::eval_key_value_pattern,
    ).protected(),
    BuiltinSpec::ordinary(
        "Merge",
        "Merges associations by key using a combiner in the current first-pass subset.",
        associations::eval_merge,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Merge called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "list1",
            template: "The argument `1` is not a valid list of Associations or rules or lists of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "MergeNested",
        "Recursively merges nested associations by key using a combiner at leaves.",
        associations::eval_merge_nested,
    )
    .with_messages(&[
        ARGX_OPERATOR,
    ]).protected(),
    BuiltinSpec::ordinary(
        "Catenate",
        "Concatenates lists and association values into a list in the current first-pass subset.",
        associations::eval_catenate,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "Catenate called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Counts",
        "Counts repeated elements and returns an association in the current first-pass subset.",
        associations::eval_counts,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "Counts called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invl",
            template: "The argument `1` is not a list.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "CountsBy",
        "Counts transformed elements and returns an association in the current first-pass subset.",
        associations::eval_counts_by,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argt",
            template: "CountsBy called with `1` arguments; 1 or 2 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "list",
            template: "List expected at position `1` in `2`.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "GroupBy",
        "Groups list elements by transformed keys, including the current reducer form.",
        associations::eval_group_by,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argb",
            template: "GroupBy called with `1` arguments; between 1 and 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "list1",
            template: "The argument `1` is not a valid list of Associations or rules or lists of rules.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "PositionIndex",
        "Builds an association from elements to 1-based position lists in the current first-pass subset.",
        associations::eval_position_index,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "argx",
            template: "PositionIndex called with `1` arguments; 1 argument is expected.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
    BuiltinSpec::ordinary(
        "Missing",
        "Represents missing-data markers in the current first-pass subset.",
        associations::eval_missing,
    ).protected(),
    BuiltinSpec::ordinary(
        "DeleteMissing",
        "Deletes Missing entries from lists, associations, and rule-list data in the current first-pass subset.",
        associations::eval_delete_missing,
    )
    .with_messages(&[
        BuiltinMessageSpec {
            tag: "arg2",
            template: "The second argument `1` is expected to be a positive integer or Infinity.",
        },
        BuiltinMessageSpec {
            tag: "arg3",
            template: "The third argument `1` is expected to be a non-negative integer or Infinity.",
        },
        BuiltinMessageSpec {
            tag: "argb",
            template: "DeleteMissing called with `1` arguments; between 1 and 3 arguments are expected.",
        },
        BuiltinMessageSpec {
            tag: "invrp",
            template: "The argument `1` is not a valid Association or a list.",
        },
    ]).protected(),
];
