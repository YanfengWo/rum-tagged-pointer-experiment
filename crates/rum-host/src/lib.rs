//! Narrow Rust embedding facade for `rum`.
//!
//! `rum-host` deliberately wraps the broader `rum-runtime` request API in
//! transport-friendly host types. It is the public boundary future Rust and
//! Python adapters should prefer over depending on evaluator internals.
//!
//! # Running untrusted code
//!
//! For hosts that run untrusted code (e.g. editor plugins), use the capability policy:
//! [`HostConfig::sandboxed`] installs a `{Pure}` policy that blocks every non-pure
//! builtin, including filesystem (`Get`/`Save`/`Import`/…) and diagnostic
//! (`RumProfileSnapshot`/`MemoryInUse`) capabilities, at dispatch — including
//! runtime-constructed calls like `Symbol["Get"][…]`. Fine-grained policies are
//! expressible via [`CapabilityPolicy`] / [`Capability`] (e.g. read-only
//! `{Pure, FileRead}` or diagnostic-enabled `{Pure, Diagnostic}`); the default is
//! unrestricted (trusted hosts: CLI / REPL / Jupyter). Combined with the resource
//! limits, cancel, timeout, and poison boundary, an untrusted evaluation can do
//! nothing but return a value from a resource-limited, capability-scoped session.
//! Role/identity mapping is the caller's concern — map your roles to a
//! `CapabilityPolicy` and pass it in.

mod config;
mod cursor;
mod model;
mod session;

pub use config::{HostConfig, HostResourceLimits};
pub use model::{
    HostCompletenessStatus, HostCompletionRequest, HostCompletionResponse, HostDefinition,
    HostDefinitionSummary, HostEvalOutcome, HostEvalRequest, HostEvalSuccess, HostFailure,
    HostFailureCode, HostInputKind, HostInputRequest, HostInputSuspension, HostInspectRequest,
    HostInspectResponse, HostIsCompleteRequest, HostIsCompleteResponse, HostMessage,
    HostMessageTemplate, HostParseRequest, HostParseResponse, HostStructuredOutput,
    HostStructuredOutputKind, HostSymbolInfo, HostTraceEvent, HostTyped, HostValue,
};
pub use rum_runtime::{Capability, CapabilityPolicy, CapabilityPolicyBuilder};
pub use session::{HostCancelHandle, HostSession, HostSessionBuilder};

#[cfg(test)]
mod lifecycle_contract_tests;

#[cfg(test)]
mod tests;
