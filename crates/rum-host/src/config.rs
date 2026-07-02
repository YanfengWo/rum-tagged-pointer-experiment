use std::path::PathBuf;

use rum_runtime::{Capability, CapabilityPolicy, ResourceLimits, SessionConfig};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostResourceLimits {
    pub recursion_limit: usize,
    pub replace_repeated_limit: usize,
    pub evaluation_pass_limit: usize,
    pub generated_element_limit: usize,
    pub integer_work_limit: usize,
    pub iteration_work_limit: usize,
}

impl Default for HostResourceLimits {
    fn default() -> Self {
        ResourceLimits::default().into()
    }
}

impl From<ResourceLimits> for HostResourceLimits {
    fn from(limits: ResourceLimits) -> Self {
        Self {
            recursion_limit: limits.recursion_limit,
            replace_repeated_limit: limits.replace_repeated_limit,
            evaluation_pass_limit: limits.evaluation_pass_limit,
            generated_element_limit: limits.generated_element_limit,
            integer_work_limit: limits.integer_work_limit,
            iteration_work_limit: limits.iteration_work_limit,
        }
    }
}

impl From<HostResourceLimits> for ResourceLimits {
    fn from(limits: HostResourceLimits) -> Self {
        Self {
            recursion_limit: limits.recursion_limit,
            replace_repeated_limit: limits.replace_repeated_limit,
            evaluation_pass_limit: limits.evaluation_pass_limit,
            generated_element_limit: limits.generated_element_limit,
            integer_work_limit: limits.integer_work_limit,
            iteration_work_limit: limits.iteration_work_limit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostConfig {
    pub resource_limits: HostResourceLimits,
    pub capability_policy: CapabilityPolicy,
    pub current_context: String,
    pub context_path: Vec<String>,
    pub package_search_paths: Vec<PathBuf>,
    pub tracing_enabled: bool,
    pub asserts_enabled: bool,
    pub install_core_builtins: bool,
}

impl Default for HostConfig {
    fn default() -> Self {
        let config = SessionConfig::default();

        Self {
            resource_limits: config.resource_limits.into(),
            capability_policy: config.capability_policy,
            current_context: config.current_context,
            context_path: config.context_path,
            package_search_paths: config.package_search_paths,
            tracing_enabled: config.tracing_enabled,
            asserts_enabled: config.asserts_enabled,
            install_core_builtins: true,
        }
    }
}

impl HostConfig {
    pub fn sandboxed() -> Self {
        Self::default().with_capability_policy(CapabilityPolicy::allow([Capability::Pure]))
    }

    pub fn with_resource_limits(mut self, limits: HostResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    pub fn with_capability_policy(mut self, policy: CapabilityPolicy) -> Self {
        self.capability_policy = policy;
        self
    }

    pub fn with_current_context(mut self, context: impl Into<String>) -> Self {
        self.current_context = context.into();
        self
    }

    pub fn with_context_path<I, S>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.context_path = contexts.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_package_search_paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.package_search_paths = paths.into_iter().map(Into::into).collect();
        self
    }

    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing_enabled = enabled;
        self
    }

    pub fn with_asserts_enabled(mut self, enabled: bool) -> Self {
        self.asserts_enabled = enabled;
        self
    }

    pub fn with_core_builtins(mut self, enabled: bool) -> Self {
        self.install_core_builtins = enabled;
        self
    }

    pub(crate) fn runtime_config(&self) -> SessionConfig {
        SessionConfig::default()
            .with_resource_limits(self.resource_limits.into())
            .with_capability_policy(self.capability_policy)
            .with_current_context(self.current_context.clone())
            .with_context_path(self.context_path.clone())
            .with_package_search_paths(self.package_search_paths.clone())
            .with_tracing(self.tracing_enabled)
            .with_asserts_enabled(self.asserts_enabled)
    }
}
