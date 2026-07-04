use super::*;

pub(super) fn profile_stats_average(total: u64, count: u64) -> u64 {
    if count == 0 { 0 } else { total / count }
}

pub(super) fn profile_stats_per_mille(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        0
    } else {
        numerator.saturating_mul(1000) / denominator
    }
}

pub(super) fn profile_symbol_name(symbol: Symbol) -> String {
    match symbol.identity() {
        Some(identity) => format!("{symbol}#{}", identity),
        None => symbol.to_string(),
    }
}

pub(super) fn profile_optional_symbol_name(symbol: Option<Symbol>) -> String {
    symbol.map_or_else(|| "-".to_string(), |symbol| symbol.to_string())
}

pub(super) fn expression_rebuild_source_name(
    source: &'static str,
    symbol: Option<Symbol>,
) -> String {
    symbol.map_or_else(|| source.to_string(), |symbol| format!("{source}:{symbol}"))
}

pub(super) fn presence_target_origin_name(origin: PresenceTargetOrigin) -> &'static str {
    match origin {
        PresenceTargetOrigin::None => "none",
        PresenceTargetOrigin::PlainSymbol => "plain",
        PresenceTargetOrigin::GeneratedDollarName => "generated-dollar",
        PresenceTargetOrigin::RuleRhsAlphaIdentity => "rule-rhs-alpha",
        PresenceTargetOrigin::OtherIdentity => "other-identity",
    }
}

pub(super) fn presence_alpha_relation_name(relation: PresenceAlphaRelation) -> &'static str {
    match relation {
        PresenceAlphaRelation::NoTarget => "no-target",
        PresenceAlphaRelation::TargetNotRuleRhsAlpha => "target-not-rule-rhs-alpha",
        PresenceAlphaRelation::ExprHasNoRuleRhsAlpha => "expr-has-no-rule-rhs-alpha",
        PresenceAlphaRelation::ExprOlderThanTarget => "expr-older-than-target",
        PresenceAlphaRelation::ExprSameOrNewerThanTarget => "expr-same-or-newer-than-target",
        PresenceAlphaRelation::Unknown => "unknown",
    }
}

pub(super) fn expression_provenance_kind_name(kind: ExpressionProvenanceKind) -> &'static str {
    match kind {
        ExpressionProvenanceKind::RuleBindingValue => "rule-binding-value",
        ExpressionProvenanceKind::RuleRhsRebuild => "rule-rhs-rebuild",
        ExpressionProvenanceKind::OtherRebuild => "other-rebuild",
        ExpressionProvenanceKind::Unknown => "unknown",
    }
}

pub(super) fn presence_scan_kind_name(kind: PresenceScanKind) -> &'static str {
    match kind {
        PresenceScanKind::Single => "single",
        PresenceScanKind::Multi => "multi",
    }
}

pub(super) fn presence_source_fingerprint_kind_name(
    kind: PresenceSourceFingerprintKind,
) -> &'static str {
    match kind {
        PresenceSourceFingerprintKind::Text => "text",
        PresenceSourceFingerprintKind::Exact => "exact",
    }
}

pub(super) fn presence_root_shape_name(shape: PresenceRootShape) -> String {
    match shape {
        PresenceRootShape::AtomSymbol(symbol) => format!("atom:{symbol}"),
        PresenceRootShape::AtomOther => "atom:<other>".to_string(),
        PresenceRootShape::CallHeadSymbol(symbol) => format!("call-head:{symbol}"),
        PresenceRootShape::CallHeadAtomOther => "call-head:<atom-other>".to_string(),
        PresenceRootShape::CallHeadCallSymbol(symbol) => format!("call-head-call:{symbol}"),
        PresenceRootShape::CallHeadCallOther => "call-head-call:<other>".to_string(),
    }
}
