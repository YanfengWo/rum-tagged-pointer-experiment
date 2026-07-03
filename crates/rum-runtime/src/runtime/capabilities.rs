#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Capability {
    Pure,
    FileRead,
    FileWrite,
}

impl Capability {
    const fn bit(self) -> u8 {
        match self {
            Self::Pure => 1 << 0,
            Self::FileRead => 1 << 1,
            Self::FileWrite => 1 << 2,
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
    }

    #[test]
    fn capability_policy_allowlist_is_default_deny() {
        let policy = CapabilityPolicy::allow([Capability::Pure, Capability::FileRead]);

        assert!(!policy.is_unrestricted());
        assert!(policy.allows(Capability::Pure));
        assert!(policy.allows(Capability::FileRead));
        assert!(!policy.allows(Capability::FileWrite));
    }

    #[test]
    fn catalog_capabilities_classify_filesystem_builtins() {
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
        assert_eq!(capability("Import"), Capability::FileRead);
        assert_eq!(capability("Save"), Capability::FileWrite);
        assert_eq!(capability("DumpSave"), Capability::FileWrite);
        assert_eq!(capability("FileNameJoin"), Capability::Pure);
        assert_eq!(capability("DirectoryName"), Capability::Pure);
        assert_eq!(capability("ParentDirectory"), Capability::Pure);
        assert_eq!(capability("FileNameTake"), Capability::Pure);
    }
}
