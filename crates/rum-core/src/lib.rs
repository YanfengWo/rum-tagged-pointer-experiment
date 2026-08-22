//! Runtime-neutral core values shared across `rum`.
//!
//! This crate owns the expression, atom, symbol, shared-expression, and core
//! head types that syntax, runtime, standard library, and host layers exchange
//! without depending on evaluator state.

pub mod expr;
pub mod heads;
pub mod profiling;

pub use expr::{
    Atom, Expr, GLOBAL_CONTEXT, IntValue, Rational, SYSTEM_CONTEXT, SharedArgs, SharedExpr, Symbol,
    WeakSharedArgs,
};
pub use heads::{CANONICAL_CORE_HEADS, is_canonical_core_head_name};
pub use num_bigint::BigInt;
pub use profiling::{
    ReprProfile, record_into_owned, record_into_vec, record_shared_args_alloc,
    record_shared_expr_alloc, repr_profile_enabled, repr_profile_snapshot,
};
