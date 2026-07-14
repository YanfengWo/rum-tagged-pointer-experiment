#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    Pure,
    FileRead,
    FileWrite,
    Clock,
    Random,
    Diagnostic,
    /// Reserved for future downstream process-control integrations.
    Process,
    /// May request interactive input from the host or suspend for host input.
    Interactive,
}

impl Capability {
    const fn bit(self) -> u8 {
        match self {
            Self::Pure => 1 << 0,
            Self::FileRead => 1 << 1,
            Self::FileWrite => 1 << 2,
            Self::Clock => 1 << 3,
            Self::Random => 1 << 4,
            Self::Diagnostic => 1 << 5,
            Self::Process => 1 << 6,
            Self::Interactive => 1 << 7,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityPolicy {
    unrestricted: bool,
    allowed_bits: u8,
}

impl Default for CapabilityPolicy {
    fn default() -> Self {
        Self::unrestricted()
    }
}

impl CapabilityPolicy {
    pub const fn unrestricted() -> Self {
        Self {
            unrestricted: true,
            allowed_bits: 0,
        }
    }

    pub fn allow<I>(capabilities: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        let mut policy = Self {
            unrestricted: false,
            allowed_bits: 0,
        };

        for capability in capabilities {
            policy.allowed_bits |= capability.bit();
        }

        policy
    }

    pub const fn builder() -> CapabilityPolicyBuilder {
        CapabilityPolicyBuilder {
            policy: Self {
                unrestricted: false,
                allowed_bits: 0,
            },
        }
    }

    #[inline]
    pub const fn is_unrestricted(self) -> bool {
        self.unrestricted
    }

    #[inline]
    pub const fn allows(self, capability: Capability) -> bool {
        self.unrestricted || self.allowed_bits & capability.bit() != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CapabilityPolicyBuilder {
    policy: CapabilityPolicy,
}

impl CapabilityPolicyBuilder {
    pub const fn new() -> Self {
        CapabilityPolicy::builder()
    }

    pub const fn allow(mut self, capability: Capability) -> Self {
        self.policy.allowed_bits |= capability.bit();
        self
    }

    pub const fn deny(mut self, capability: Capability) -> Self {
        self.policy.allowed_bits &= !capability.bit();
        self
    }

    pub const fn build(self) -> CapabilityPolicy {
        self.policy
    }
}

impl Default for CapabilityPolicyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::super::core_builtin_specs;
    use super::*;

    #[test]
    fn capability_policy_defaults_to_unrestricted() {
        let policy = CapabilityPolicy::default();

        assert!(policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(policy.allows(Capability::FileRead));
        assert!(policy.allows(Capability::FileWrite));
        assert!(policy.allows(Capability::Clock));
        assert!(policy.allows(Capability::Random));
        assert!(policy.allows(Capability::Diagnostic));
        assert!(policy.allows(Capability::Process));
        assert!(policy.allows(Capability::Interactive));
        assert_eq!(Capability::Interactive.bit(), 1 << 7);
    }

    #[test]
    fn capability_policy_allowlist_is_default_deny() {
        let policy = CapabilityPolicy::allow([Capability::Pure, Capability::FileRead]);

        assert!(!policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(policy.allows(Capability::FileRead));
        assert!(!policy.allows(Capability::FileWrite));
        assert!(!policy.allows(Capability::Clock));
        assert!(!policy.allows(Capability::Random));
        assert!(!policy.allows(Capability::Diagnostic));
        assert!(!policy.allows(Capability::Process));
        assert!(!policy.allows(Capability::Interactive));
    }

    #[test]
    fn capability_policy_builder_can_allow_and_deny_individual_capabilities() {
        let policy = CapabilityPolicy::builder()
            .allow(Capability::Pure)
            .allow(Capability::FileRead)
            .deny(Capability::FileRead)
            .allow(Capability::Random)
            .build();

        assert!(!policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(!policy.allows(Capability::FileRead));
        assert!(!policy.allows(Capability::FileWrite));
        assert!(!policy.allows(Capability::Clock));
        assert!(policy.allows(Capability::Random));
        assert!(!policy.allows(Capability::Diagnostic));
        assert!(!policy.allows(Capability::Process));
        assert!(!policy.allows(Capability::Interactive));
        assert_eq!(
            CapabilityPolicyBuilder::default().build(),
            CapabilityPolicy::allow([])
        );
    }

    #[test]
    fn capability_policy_can_explicitly_allow_diagnostics() {
        let policy = CapabilityPolicy::allow([Capability::Pure, Capability::Diagnostic]);

        assert!(!policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(policy.allows(Capability::Diagnostic));
        assert!(!policy.allows(Capability::FileRead));
        assert!(!policy.allows(Capability::FileWrite));
        assert!(!policy.allows(Capability::Clock));
        assert!(!policy.allows(Capability::Random));
        assert!(!policy.allows(Capability::Process));
        assert!(!policy.allows(Capability::Interactive));
    }

    #[test]
    fn catalog_capabilities_classify_io_builtins() {
        fn capability(name: &str) -> Capability {
            core_builtin_specs()
                .iter()
                .find(|spec| spec.name == name)
                .expect("builtin should be cataloged")
                .capability
        }

        assert_eq!(capability("Get"), Capability::FileRead);
        assert_eq!(capability("Needs"), Capability::FileRead);
        assert_eq!(capability("FindFile"), Capability::FileRead);
        assert_eq!(capability("FileNames"), Capability::FileRead);
        assert_eq!(capability("Save"), Capability::FileWrite);
        assert_eq!(capability("DumpSave"), Capability::FileWrite);
        assert_eq!(capability("FileNameJoin"), Capability::Pure);
        assert_eq!(capability("DirectoryName"), Capability::Pure);
        assert_eq!(capability("ParentDirectory"), Capability::Pure);
        assert_eq!(capability("FileNameTake"), Capability::Pure);
        assert_eq!(capability("Dialog"), Capability::Interactive);
        assert_eq!(capability("InputString"), Capability::Interactive);
        assert_eq!(capability("Input"), Capability::Interactive);
    }

    #[test]
    fn catalog_capabilities_classify_diagnostic_builtins() {
        fn capability(name: &str) -> Capability {
            core_builtin_specs()
                .iter()
                .find(|spec| spec.name == name)
                .expect("builtin should be cataloged")
                .capability
        }

        assert_eq!(capability("RumProfileReset"), Capability::Diagnostic);
        assert_eq!(capability("RumProfileSnapshot"), Capability::Diagnostic);
        assert_eq!(capability("MemoryInUse"), Capability::Diagnostic);
        assert_eq!(capability("MaxMemoryUsed"), Capability::Diagnostic);
        assert_eq!(capability("Trace"), Capability::Pure);
        assert_eq!(capability("TracePrint"), Capability::Pure);
        assert_eq!(capability("ByteCount"), Capability::Pure);
        assert_eq!(capability("TestCreate"), Capability::Pure);
    }

    #[test]
    fn catalog_capabilities_classify_clock_builtins() {
        fn capability(name: &str) -> Capability {
            core_builtin_specs()
                .iter()
                .find(|spec| spec.name == name)
                .expect("builtin should be cataloged")
                .capability
        }

        assert_eq!(capability("AbsoluteTime"), Capability::Clock);
        assert_eq!(capability("Timing"), Capability::Clock);
        assert_eq!(capability("AbsoluteTiming"), Capability::Clock);
        assert_eq!(capability("RepeatedTiming"), Capability::Clock);
        assert_eq!(capability("TimeConstrained"), Capability::Clock);
        assert_eq!(capability("Pause"), Capability::Clock);
        assert_eq!(capability("EvaluationData"), Capability::Clock);
        assert_eq!(capability("TestReport"), Capability::Clock);
    }
}
