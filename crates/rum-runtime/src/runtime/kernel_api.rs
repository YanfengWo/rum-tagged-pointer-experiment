//! Kernel API contract between the evaluator kernel and builtin families.
//!
//! Builtin families import only from this module, plus `rum_core`,
//! `rum_syntax`, and standard library modules. This surface grows one family
//! conversion at a time. Items exported here become semver-relevant when the
//! crate split ships.

// capabilities
pub use super::capabilities::Capability;

// arithmetic
pub use super::arithmetic::{expr_as_exact_integer, expr_as_exact_rational};

// association_data
pub use super::association_data::{
    AssociationEntryForm, association_entries_from_expr, association_entry_forms_from_expr,
    is_missing_expr, list_items, missing_key_absent_expr, parse_association_entry_form,
    render_association, render_association_entry_forms_preserving_heads,
};

// expr_helpers
pub use super::expr_helpers::{
    expr_as_non_negative_usize, expr_as_signed_isize, expr_as_string, expr_as_string_list,
    expr_as_symbol, is_negative_one_expr, system_expr,
};

// expr_runtime_helpers
pub use super::expr_runtime_helpers::{
    base64_encode, call_items_with_head, compare_expr_canonical, is_system_symbol_name,
};

// lexical_substitution
pub use super::lexical_substitution::substitute_lexical_bindings;

// message_selector_parsing
pub use super::message_selector_parsing::truthiness_symbol;

// metadata
pub use super::metadata::{Attribute, AttributeSet, BuiltinSpec, HoldAttribute};

// numeric traits
pub use num_integer::{Integer, Roots};
pub use num_traits::{One, Signed, ToPrimitive, Zero};

// model
pub use super::model::{
    CasesLevelSpec, CasesTraversalConfig, RewriteRule, RuleApplicationSource, Session,
    SymbolExprMap,
};

// parse_and_message_helpers
pub use super::parse_and_message_helpers::{parse_heads_option, parse_standard_level_spec};

// session_state
pub use super::session_state::{EvalError, ThreadArgSpec, ThreadingDispatch};

// structure_ops
pub use super::structure_ops::{
    exact_expr_at_path, exact_position_paths_from_expr,
    exact_position_paths_from_part_selector_exprs, replace_exact_path,
};

// tree_walkers
pub use super::tree_walkers::collect_level_parts_owned;

// suspension/core
pub use super::suspension::ContinuationValue;

// text_rendering
pub use super::text_rendering::{ToStringFormat, render_input_like_expr, render_to_string_expr};
