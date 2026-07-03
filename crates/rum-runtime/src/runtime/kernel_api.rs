//! Kernel API contract between the evaluator kernel and builtin families.
//!
//! Builtin families import only from this module, plus `rum_core`,
//! `rum_syntax`, and standard library modules. This surface grows one family
//! conversion at a time. Items exported here become semver-relevant when the
//! crate split ships.

// arithmetic
pub use super::arithmetic::{expr_as_exact_integer, expr_as_exact_rational};

// associations/basic
pub use super::associations::missing_key_absent_expr;

// associations/data
pub use super::associations::{
    AssociationEntryForm, association_entries_from_expr, association_entry_forms_from_expr,
    list_items, parse_association_entry_form, predicate_bool_expr, render_association,
    render_association_entry_forms_preserving_heads,
};

// associations/missing
pub use super::associations::is_missing_expr;

// expr_helpers
pub use super::expr_helpers::{
    expr_as_non_negative_usize, expr_as_signed_isize, expr_as_string, expr_as_symbol,
    is_negative_one_expr, system_expr,
};

// expr_runtime_helpers
pub use super::expr_runtime_helpers::{
    call_items_with_head, compare_expr_canonical, is_system_symbol_name,
};

// message_selector_parsing
pub use super::message_selector_parsing::truthiness_symbol;

// metadata
pub use super::metadata::{Attribute, AttributeSet};

// model
pub use super::model::{
    CasesLevelSpec, CasesTraversalConfig, RewriteRule, RuleApplicationSource, Session,
    SymbolExprMap,
};

// parse_and_message_helpers
pub use super::parse_and_message_helpers::{parse_heads_option, parse_standard_level_spec};

// polynomials
pub use super::polynomials::parse_indeterminate_expr;

// session_state
pub use super::session_state::{EvalError, ThreadArgSpec, ThreadingDispatch};

// structure_ops
pub use super::structure_ops::{
    exact_expr_at_path, exact_position_paths_from_expr, replace_exact_path,
};

// suspension/core
pub use super::suspension::ContinuationValue;
