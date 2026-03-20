use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rum_core::{Atom, BigInt, Expr, GLOBAL_CONTEXT, SYSTEM_CONTEXT, Symbol, heads};
use rum_syntax::{
    BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxError, SyntaxExpr, SyntaxExprKind,
    parse_expr,
};

mod attributes;
mod binding_substitution;
mod builtin_defaults;
mod builtins_context_loader;
mod builtins_messages;
mod builtins_messages_inspect;
mod context_and_load_helpers;
mod control_flow;
mod definition_analysis_helpers;
mod definition_application;
mod definition_catalog;
mod definition_clear;
mod definition_tags;
mod definitions;
mod eval_call_prepare;
mod eval_core;
mod eval_dispatch;
mod expr_helpers;
mod expr_runtime_helpers;
mod function_and_subst_helpers;
mod hold_thread;
mod lexical_substitution;
mod listable_dispatch;
mod lowering;
mod matcher_and_symbols;
mod message_catalog;
mod message_format;
mod message_selector_parsing;
mod metadata;
mod model;
mod parse_and_message_helpers;
mod pattern_matching;
#[cfg(test)]
mod pattern_matching_interpreted;
mod resolution;
mod rewrite_builtins;
mod scoping;
mod session_api;
mod session_eval;
mod session_helpers;
mod session_state;
mod slot_substitution;

#[allow(unused_imports)]
use self::{
    attributes::*, binding_substitution::*, builtin_defaults::*, builtins_context_loader::*,
    builtins_messages::*, builtins_messages_inspect::*, context_and_load_helpers::*,
    control_flow::*, definition_analysis_helpers::*, definition_application::*,
    definition_catalog::*, definition_clear::*, definition_tags::*, definitions::*,
    eval_call_prepare::*, eval_core::*, eval_dispatch::*, expr_helpers::*, expr_runtime_helpers::*,
    function_and_subst_helpers::*, hold_thread::*, lexical_substitution::*, listable_dispatch::*,
    lowering::*, matcher_and_symbols::*, message_catalog::*, message_format::*,
    message_selector_parsing::*, metadata::*, model::*, parse_and_message_helpers::*,
    pattern_matching::*, resolution::*, rewrite_builtins::*, scoping::*, session_api::*,
    session_eval::*, session_helpers::*, session_state::*, slot_substitution::*,
};
pub use self::{lowering::lower_syntax_expr, metadata::*, model::*, session_state::EvalError};

#[cfg(test)]
mod tests;
