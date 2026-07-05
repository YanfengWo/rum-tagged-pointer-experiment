//! Pure `rum` kernel.
//!
//! This crate owns lowering, evaluation sessions, definitions, scoping,
//! messages, matching, tracing, resource limits, capability policy enforcement,
//! and the host-facing request APIs used by embedding layers.

mod runtime;

pub use runtime::*;
