use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rum_core::{
    Atom, BigInt, Expr, GLOBAL_CONTEXT, Rational, SYSTEM_CONTEXT, SharedArgs, SharedExpr, Symbol,
    heads,
};
use rum_syntax::{
    BinaryOp, BlankKind, ParsedSource, PostfixOp, PrefixOp, SlotSpec, StructuredSourceDiagnostic,
    SyntaxError, SyntaxExpr, SyntaxExprKind, parse_expr, parse_with_diagnostics,
};

mod arithmetic;
mod associations;
mod attributes;
mod binding_substitution;
mod builtin_adapters;
mod builtin_catalog;
mod builtins_context_loader;
mod builtins_input;
mod builtins_messages;
mod builtins_messages_inspect;
mod capabilities;
mod context_and_load_helpers;
mod control_flow;
mod debug_tools;
mod definition_analysis_helpers;
mod definition_application;
mod definition_catalog;
mod definition_clear;
mod definition_tags;
mod definition_updates;
mod definitions;
mod eval_call_prepare;
mod eval_core;
mod eval_dispatch;
mod evaluation_cache;
mod expr_helpers;
mod expr_runtime_helpers;
mod function_and_subst_helpers;
mod functional_ops;
mod hold_thread;
pub mod kernel_api;
mod lexical_presence;
mod lexical_substitution;
mod listable_dispatch;
mod logic;
mod lowering;
mod matcher_and_symbols;
mod message_catalog;
mod message_format;
mod message_selector_parsing;
mod metadata;
mod model;
mod namespace_tools;
mod options;
mod package_tools;
mod parse_and_message_helpers;
mod pattern_matching;
#[cfg(test)]
mod pattern_matching_interpreted;
mod predicates;
mod profiling;
mod resolution;
mod rewrite_builtins;
mod scoping;
mod session_api;
mod session_eval;
mod session_helpers;
mod session_requests;
mod session_state;
mod slot_substitution;
mod structure_ops;
mod suspension;
mod text_rendering;
mod tree_walkers;
mod vm_rhs_template;

#[allow(unused_imports)]
use self::{
    arithmetic::*, associations::*, attributes::*, binding_substitution::*, builtin_adapters::*,
    builtin_catalog::*, builtins_context_loader::*, builtins_input::*, builtins_messages::*,
    builtins_messages_inspect::*, capabilities::*, context_and_load_helpers::*, control_flow::*,
    debug_tools::*, definition_analysis_helpers::*, definition_application::*,
    definition_catalog::*, definition_clear::*, definition_tags::*, definition_updates::*,
    definitions::*, eval_call_prepare::*, eval_core::*, eval_dispatch::*, evaluation_cache::*,
    expr_helpers::*, expr_runtime_helpers::*, function_and_subst_helpers::*, functional_ops::*,
    hold_thread::*, lexical_substitution::*, listable_dispatch::*, logic::*, lowering::*,
    matcher_and_symbols::*, message_catalog::*, message_format::*, message_selector_parsing::*,
    metadata::*, model::*, namespace_tools::*, options::*, package_tools::*,
    parse_and_message_helpers::*, pattern_matching::*, predicates::*, profiling::*, resolution::*,
    rewrite_builtins::*, scoping::*, session_api::*, session_eval::*, session_helpers::*,
    session_requests::*, session_state::*, slot_substitution::*, structure_ops::*, suspension::*,
    text_rendering::*, tree_walkers::*, vm_rhs_template::*,
};
pub use self::{
    builtin_catalog::core_builtin_specs, capabilities::*, lowering::lower_syntax_expr, metadata::*,
    model::*, session_state::EvalError,
};

#[cfg(test)]
mod tests;
