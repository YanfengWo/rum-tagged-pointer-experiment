use std::collections::HashMap;
use std::sync::OnceLock;

use super::*;

mod associations;
mod control;
mod debug;
mod functional;
mod logic;
mod namespace;
mod numeric;
mod rules;
mod scoping;
mod strings;
mod structure;

pub fn core_builtin_specs() -> &'static [BuiltinSpec] {
    static CORE_BUILTINS: OnceLock<Vec<BuiltinSpec>> = OnceLock::new();

    CORE_BUILTINS
        .get_or_init(|| {
            let mut specs = Vec::with_capacity(
                numeric::NUMERIC_BUILTINS.len()
                    + strings::STRING_BUILTINS.len()
                    + structure::STRUCTURE_BUILTINS.len()
                    + associations::ASSOCIATION_BUILTINS.len()
                    + debug::DEBUG_BUILTINS.len()
                    + control::CONTROL_BUILTINS.len()
                    + functional::FUNCTIONAL_BUILTINS.len()
                    + scoping::SCOPING_BUILTINS.len()
                    + logic::LOGIC_BUILTINS.len()
                    + rules::RULE_BUILTINS.len()
                    + namespace::NAMESPACE_BUILTINS.len(),
            );
            specs.extend_from_slice(numeric::NUMERIC_BUILTINS);
            specs.extend_from_slice(strings::STRING_BUILTINS);
            specs.extend_from_slice(structure::STRUCTURE_BUILTINS);
            specs.extend_from_slice(associations::ASSOCIATION_BUILTINS);
            specs.extend_from_slice(debug::DEBUG_BUILTINS);
            specs.extend_from_slice(control::CONTROL_BUILTINS);
            specs.extend_from_slice(functional::FUNCTIONAL_BUILTINS);
            specs.extend_from_slice(scoping::SCOPING_BUILTINS);
            specs.extend_from_slice(logic::LOGIC_BUILTINS);
            specs.extend_from_slice(rules::RULE_BUILTINS);
            specs.extend_from_slice(namespace::NAMESPACE_BUILTINS);
            specs
        })
        .as_slice()
}

pub(super) fn default_builtin_dispatch_spec(name: &str) -> Option<BuiltinSpec> {
    static CORE_BUILTINS_BY_NAME: OnceLock<HashMap<&'static str, BuiltinSpec>> = OnceLock::new();

    CORE_BUILTINS_BY_NAME
        .get_or_init(|| {
            core_builtin_specs()
                .iter()
                .map(|spec| (spec.name, *spec))
                .collect()
        })
        .get(name)
        .copied()
}

pub(super) fn default_system_attribute_set(symbol: &Symbol) -> AttributeSet {
    // Wolfram's private Information display wrapper holds symbols that would
    // otherwise evaluate while an InformationData expression reaches a fixed point.
    if symbol.context() == "Information`" && symbol.name() == "InformationValueForm" {
        return AttributeSet::from_hold(HoldAttribute::HoldAll);
    }

    if symbol.context() != SYSTEM_CONTEXT {
        return AttributeSet::empty();
    }

    default_builtin_dispatch_spec(symbol.name())
        .map(|spec| spec.attributes)
        .unwrap_or_else(AttributeSet::empty)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_catalog_drives_both_dispatch_and_default_attributes() {
        let module = Symbol::new(SYSTEM_CONTEXT, "Module");
        let spec = default_builtin_dispatch_spec("Module").expect("Module should be cataloged");

        assert_eq!(spec.dispatch, BuiltinDispatch::SpecialForm);
        assert_eq!(
            spec.attributes,
            AttributeSet::from_hold(HoldAttribute::HoldAll)
        );
        assert_eq!(default_system_attribute_set(&module), spec.attributes);
    }

    #[test]
    fn non_catalog_system_attributes_can_still_use_manual_fallbacks() {
        let plus = Symbol::new(SYSTEM_CONTEXT, "Plus");
        let attributes = default_system_attribute_set(&plus);

        assert!(attributes.contains(Attribute::Listable));
        assert!(attributes.contains(Attribute::Orderless));
        assert!(attributes.contains(Attribute::Flat));
        assert!(attributes.contains(Attribute::OneIdentity));
    }
}
