//! Narrow Rust embedding facade for `rum`.
//!
//! `rum-host` deliberately wraps the broader `rum-runtime` request API in
//! transport-friendly host types. It is the public boundary future Rust and
//! Python adapters should prefer over depending on evaluator internals.

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
