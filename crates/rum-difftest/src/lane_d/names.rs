use super::{LaneDProfile, LaneDSuite, LaneDThresholdProfile};

pub fn suite_name(suite: LaneDSuite) -> &'static str {
    match suite {
        LaneDSuite::All => "all",
        LaneDSuite::Timing => "timing",
        LaneDSuite::Memory => "memory",
    }
}

pub fn profile_name(profile: LaneDProfile) -> &'static str {
    match profile {
        LaneDProfile::Smoke => "smoke",
        LaneDProfile::Full => "full",
    }
}

pub fn threshold_profile_name(profile: LaneDThresholdProfile) -> &'static str {
    match profile {
        LaneDThresholdProfile::LocalDebug => "local-debug",
        LaneDThresholdProfile::CiDebug => "ci-debug",
        LaneDThresholdProfile::CiRelease => "ci-release",
    }
}

pub fn parse_suite_name(name: &str) -> Option<LaneDSuite> {
    match name {
        "all" => Some(LaneDSuite::All),
        "timing" => Some(LaneDSuite::Timing),
        "memory" => Some(LaneDSuite::Memory),
        _ => None,
    }
}

pub fn parse_profile_name(name: &str) -> Option<LaneDProfile> {
    match name {
        "smoke" => Some(LaneDProfile::Smoke),
        "full" => Some(LaneDProfile::Full),
        _ => None,
    }
}

pub fn parse_threshold_profile_name(name: &str) -> Option<LaneDThresholdProfile> {
    match name {
        "local-debug" => Some(LaneDThresholdProfile::LocalDebug),
        "ci-debug" => Some(LaneDThresholdProfile::CiDebug),
        "ci-release" => Some(LaneDThresholdProfile::CiRelease),
        _ => None,
    }
}
