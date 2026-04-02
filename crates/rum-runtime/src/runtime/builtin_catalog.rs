use std::sync::OnceLock;

use super::*;

mod associations;
mod control;
mod functional;
mod scoping;
mod strings;
mod structure;

const CORE_BUILTINS_PREFIX: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary(
        "Identity",
        "Returns its argument unchanged.",
        BuiltinHandler::Identity,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Plus",
        "Adds exact numbers and canonically combines additive expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Plus,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Times",
        "Multiplies exact numbers and canonically combines multiplicative expressions.",
        AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        BuiltinHandler::Times,
    ),
    BuiltinSpec::ordinary(
        "Power",
        "Raises an exact number to an integer power when possible.",
        BuiltinHandler::Power,
    ),
    BuiltinSpec::ordinary(
        "Subtract",
        "Subtracts one exact number from another.",
        BuiltinHandler::Subtract,
    ),
    BuiltinSpec::ordinary(
        "Divide",
        "Divides exact numbers, producing exact rationals when possible.",
        BuiltinHandler::Divide,
    ),
    BuiltinSpec::ordinary(
        "Mod",
        "Returns the modulus of one integer with respect to another.",
        BuiltinHandler::Mod,
    ),
    BuiltinSpec::ordinary(
        "Quotient",
        "Returns the floor-division quotient of two integers.",
        BuiltinHandler::Quotient,
    ),
    BuiltinSpec::ordinary(
        "PowerMod",
        "Computes modular exponentiation for exact integers.",
        BuiltinHandler::PowerMod,
    ),
    BuiltinSpec::ordinary(
        "GCD",
        "Computes the greatest common divisor of exact integers.",
        BuiltinHandler::GCD,
    ),
    BuiltinSpec::ordinary(
        "LCM",
        "Computes the least common multiple of exact integers.",
        BuiltinHandler::LCM,
    ),
    BuiltinSpec::ordinary(
        "Expand",
        "Expands rational-coefficient polynomial expressions over pattern-free indeterminates.",
        BuiltinHandler::Expand,
    ),
    BuiltinSpec::ordinary(
        "Coefficient",
        "Extracts coefficients of exact monomial powers from rational-coefficient polynomial expressions.",
        BuiltinHandler::Coefficient,
    ),
    BuiltinSpec::ordinary(
        "CoefficientList",
        "Builds coefficient arrays for rational-coefficient polynomial expressions in specified indeterminates.",
        BuiltinHandler::CoefficientList,
    ),
    BuiltinSpec::ordinary(
        "Together",
        "Combines rational-function terms over a common polynomial denominator.",
        BuiltinHandler::Together,
    ),
    BuiltinSpec::ordinary(
        "Factor",
        "Factors rational-coefficient polynomials over the rationals using the current polynomial kernel.",
        BuiltinHandler::Factor,
    ),
    BuiltinSpec::ordinary_with_hold(
        "D",
        "Differentiates exact arithmetic, rational-function, and abstract-function expressions with held differentiation variables.",
        HoldAttribute::HoldRest,
        BuiltinHandler::D,
    ),
];

const CORE_BUILTINS_SUFFIX: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "And",
        "Evaluates a conjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        BuiltinHandler::And,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Or",
        "Evaluates a disjunction with current short-circuit boolean semantics.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Or,
    ),
    BuiltinSpec::ordinary(
        "Not",
        "Negates a boolean value in the current exact boolean subset.",
        BuiltinHandler::Not,
    ),
    BuiltinSpec::ordinary(
        "TrueQ",
        "Tests whether an expression evaluates exactly to True.",
        BuiltinHandler::TrueQ,
    ),
    BuiltinSpec::ordinary(
        "BooleanQ",
        "Tests whether an expression evaluates exactly to True or False.",
        BuiltinHandler::BooleanQ,
    ),
    BuiltinSpec::ordinary(
        "AtomQ",
        "Tests whether an expression is atomic rather than a normal expression.",
        BuiltinHandler::AtomQ,
    ),
    BuiltinSpec::ordinary(
        "ListQ",
        "Tests whether an expression has head List.",
        BuiltinHandler::ListQ,
    ),
    BuiltinSpec::ordinary(
        "StringQ",
        "Tests whether an expression is a string atom.",
        BuiltinHandler::StringQ,
    ),
    BuiltinSpec::ordinary(
        "IntegerQ",
        "Tests whether an expression is an exact integer atom.",
        BuiltinHandler::IntegerQ,
    ),
    BuiltinSpec::ordinary(
        "NumberQ",
        "Tests whether an expression is an exact numeric atom in the current number subset.",
        BuiltinHandler::NumberQ,
    ),
    BuiltinSpec::ordinary(
        "NumericQ",
        "Tests whether an expression is currently numeric in the exact-number subset.",
        BuiltinHandler::NumericQ,
    ),
    BuiltinSpec::ordinary(
        "EvenQ",
        "Tests whether an expression is currently an even exact integer.",
        BuiltinHandler::EvenQ,
    ),
    BuiltinSpec::ordinary(
        "OddQ",
        "Tests whether an expression is currently an odd exact integer.",
        BuiltinHandler::OddQ,
    ),
    BuiltinSpec::ordinary(
        "Positive",
        "Tests positivity in the current exact integer/rational subset.",
        BuiltinHandler::Positive,
    ),
    BuiltinSpec::ordinary(
        "Negative",
        "Tests negativity in the current exact integer/rational subset.",
        BuiltinHandler::Negative,
    ),
    BuiltinSpec::ordinary(
        "NonPositive",
        "Tests non-positivity in the current exact integer/rational subset.",
        BuiltinHandler::NonPositive,
    ),
    BuiltinSpec::ordinary(
        "NonNegative",
        "Tests non-negativity in the current exact integer/rational subset.",
        BuiltinHandler::NonNegative,
    ),
    BuiltinSpec::ordinary(
        "PossibleZeroQ",
        "Tests exact zero in the current exact integer/rational subset.",
        BuiltinHandler::PossibleZeroQ,
    ),
    BuiltinSpec::ordinary(
        "MachineNumberQ",
        "Tests whether an expression is a machine number; currently always False because only exact numbers exist.",
        BuiltinHandler::MachineNumberQ,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumberQ",
        "Tests whether an expression is a real-valued number in the current exact-number subset.",
        BuiltinHandler::RealValuedNumberQ,
    ),
    BuiltinSpec::ordinary(
        "RealValuedNumericQ",
        "Tests whether an expression is real-valued and numeric in the current exact-number subset.",
        BuiltinHandler::RealValuedNumericQ,
    ),
    BuiltinSpec::ordinary(
        "PrimeQ",
        "Tests primality in the current exact-integer subset.",
        BuiltinHandler::PrimeQ,
    ),
    BuiltinSpec::ordinary(
        "AlgebraicIntegerQ",
        "Tests whether an expression is an exact integer in the current algebraic-integer subset.",
        BuiltinHandler::AlgebraicIntegerQ,
    ),
    BuiltinSpec::ordinary(
        "DuplicateFreeQ",
        "Tests whether a List has no repeated elements in the current exact structural subset.",
        BuiltinHandler::DuplicateFreeQ,
    ),
    BuiltinSpec::ordinary(
        "IntersectingQ",
        "Tests whether two Lists share any element in the current exact structural subset.",
        BuiltinHandler::IntersectingQ,
    ),
    BuiltinSpec::ordinary(
        "DisjointQ",
        "Tests whether two Lists share no element in the current exact structural subset.",
        BuiltinHandler::DisjointQ,
    ),
    BuiltinSpec::ordinary(
        "SubsetQ",
        "Tests subset inclusion for Lists in the current exact structural subset.",
        BuiltinHandler::SubsetQ,
    ),
    BuiltinSpec::ordinary(
        "Order",
        "Orders two expressions using the current canonical textual ordering subset.",
        BuiltinHandler::Order,
    ),
    BuiltinSpec::ordinary(
        "OrderedQ",
        "Tests whether a List is nondecreasing under the current canonical textual ordering subset.",
        BuiltinHandler::OrderedQ,
    ),
    BuiltinSpec::ordinary(
        "ContainsAll",
        "Tests whether a List contains every requested element in the current exact structural subset.",
        BuiltinHandler::ContainsAll,
    ),
    BuiltinSpec::ordinary(
        "ContainsAny",
        "Tests whether a List contains any requested element in the current exact structural subset.",
        BuiltinHandler::ContainsAny,
    ),
    BuiltinSpec::ordinary(
        "ContainsExactly",
        "Tests List multiset equality in the current exact structural subset.",
        BuiltinHandler::ContainsExactly,
    ),
    BuiltinSpec::ordinary(
        "ContainsNone",
        "Tests whether a List contains none of the requested elements in the current exact structural subset.",
        BuiltinHandler::ContainsNone,
    ),
    BuiltinSpec::ordinary(
        "ContainsOnly",
        "Tests whether every element of a List belongs to the allowed-element List in the current exact structural subset.",
        BuiltinHandler::ContainsOnly,
    ),
    BuiltinSpec::ordinary(
        "Between",
        "Tests interval inclusion in the current exact numeric subset.",
        BuiltinHandler::Between,
    ),
    BuiltinSpec::ordinary(
        "VectorQ",
        "Tests whether an expression is a one-dimensional List array in the current exact structural subset.",
        BuiltinHandler::VectorQ,
    ),
    BuiltinSpec::ordinary(
        "MatrixQ",
        "Tests whether an expression is a rectangular two-dimensional List array in the current exact structural subset.",
        BuiltinHandler::MatrixQ,
    ),
    BuiltinSpec::ordinary(
        "ArrayQ",
        "Tests whether an expression is a rectangular List array in the current exact structural subset.",
        BuiltinHandler::ArrayQ,
    ),
    BuiltinSpec::ordinary(
        "AllTrue",
        "Tests whether all selected structural parts satisfy a criterion in the current boolean subset.",
        BuiltinHandler::AllTrue,
    ),
    BuiltinSpec::ordinary(
        "AnyTrue",
        "Tests whether any selected structural part satisfies a criterion in the current boolean subset.",
        BuiltinHandler::AnyTrue,
    ),
    BuiltinSpec::ordinary(
        "NoneTrue",
        "Tests whether no selected structural parts satisfy a criterion in the current boolean subset.",
        BuiltinHandler::NoneTrue,
    ),
    BuiltinSpec::ordinary(
        "CoprimeQ",
        "Tests whether exact integers are pairwise relatively prime in the current integer subset.",
        BuiltinHandler::CoprimeQ,
    ),
    BuiltinSpec::ordinary(
        "SameQ",
        "Tests whether all arguments are exactly identical after evaluation.",
        BuiltinHandler::SameQ,
    ),
    BuiltinSpec::ordinary(
        "UnsameQ",
        "Tests whether no two arguments are exactly identical after evaluation.",
        BuiltinHandler::UnsameQ,
    ),
    BuiltinSpec::ordinary(
        "Equal",
        "Tests equality in the current exact subset for numbers, strings, and obviously identical expressions.",
        BuiltinHandler::Equal,
    ),
    BuiltinSpec::ordinary(
        "Unequal",
        "Tests inequality in the current exact subset for numbers, strings, and obviously identical expressions.",
        BuiltinHandler::Unequal,
    ),
    BuiltinSpec::ordinary(
        "Less",
        "Tests strict increasing order in the current exact comparable subset.",
        BuiltinHandler::Less,
    ),
    BuiltinSpec::ordinary(
        "Greater",
        "Tests strict decreasing order in the current exact comparable subset.",
        BuiltinHandler::Greater,
    ),
    BuiltinSpec::ordinary(
        "LessEqual",
        "Tests nondecreasing order in the current exact comparable subset.",
        BuiltinHandler::LessEqual,
    ),
    BuiltinSpec::ordinary(
        "GreaterEqual",
        "Tests nonincreasing order in the current exact comparable subset.",
        BuiltinHandler::GreaterEqual,
    ),
    BuiltinSpec::ordinary(
        "Part",
        "Extracts structural parts using integer positions, All, Span, and exact nested part specifications.",
        BuiltinHandler::Part,
    ),
    BuiltinSpec::ordinary(
        "Extract",
        "Extracts parts at structural positions, optionally wrapping each result before evaluation.",
        BuiltinHandler::Extract,
    ),
    BuiltinSpec::ordinary(
        "ReplacePart",
        "Replaces structural parts via immediate or delayed rules over exact, Span, and pattern-based position specifications, with Heads-aware matching.",
        BuiltinHandler::ReplacePart,
    ),
    BuiltinSpec::special(
        "Set",
        "Assigns an immediate value to a symbol or pattern.",
        BuiltinHandler::Set,
    ),
    BuiltinSpec::special(
        "SetDelayed",
        "Assigns a delayed definition evaluated on use.",
        BuiltinHandler::SetDelayed,
    ),
    BuiltinSpec::special(
        "TagSet",
        "Stores an immediate tagged definition, including upvalues when the tag appears in an argument position.",
        BuiltinHandler::TagSet,
    ),
    BuiltinSpec::special(
        "TagSetDelayed",
        "Stores a delayed tagged definition, including upvalues when the tag appears in an argument position.",
        BuiltinHandler::TagSetDelayed,
    ),
    BuiltinSpec::special(
        "TagUnset",
        "Removes a tagged definition using an explicit tag and lhs.",
        BuiltinHandler::TagUnset,
    ),
    BuiltinSpec::special(
        "Rule",
        "Represents an immediate rewrite rule.",
        BuiltinHandler::Rule,
    ),
    BuiltinSpec::special(
        "RuleDelayed",
        "Represents a delayed rewrite rule.",
        BuiltinHandler::RuleDelayed,
    ),
    BuiltinSpec::special(
        "MatchQ",
        "Tests whether an expression matches a pattern.",
        BuiltinHandler::MatchQ,
    ),
    BuiltinSpec::special(
        "Cases",
        "Collects subexpressions matching a pattern or rule.",
        BuiltinHandler::Cases,
    ),
    BuiltinSpec::special(
        "Count",
        "Counts subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::Count,
    ),
    BuiltinSpec::special(
        "MemberQ",
        "Tests whether any subexpression matches a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::MemberQ,
    ),
    BuiltinSpec::special(
        "FreeQ",
        "Tests whether no subexpression matches a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::FreeQ,
    ),
    BuiltinSpec::ordinary(
        "Level",
        "Collects subexpressions found on levels specified by a standard levelspec.",
        BuiltinHandler::Level,
    ),
    BuiltinSpec::special(
        "Position",
        "Finds positions of subexpressions matching a pattern on levels specified by a standard levelspec.",
        BuiltinHandler::Position,
    ),
    BuiltinSpec::special(
        "ReplaceAll",
        "Applies rules everywhere in an expression.",
        BuiltinHandler::ReplaceAll,
    ),
    BuiltinSpec::special(
        "ReplaceRepeated",
        "Repeatedly applies rules until an expression reaches a fixed point.",
        BuiltinHandler::ReplaceRepeated,
    ),
    BuiltinSpec::ordinary_with_hold(
        "NormalizeRules",
        "Normalizes a rule or rule list into an explicit canonical list form for debugging.",
        HoldAttribute::HoldAll,
        BuiltinHandler::NormalizeRules,
    ),
    BuiltinSpec::new(
        "$Context",
        "Evaluates to the current default context name as a string.",
    ),
    BuiltinSpec::new(
        "$ContextPath",
        "Evaluates to the active context search path as a list of strings.",
    ),
    BuiltinSpec::new(
        "$Aborted",
        "Represents a computation aborted by Abort or an external interrupt.",
    ),
    BuiltinSpec::new("$Failed", "Represents a failed operation."),
    BuiltinSpec::new(
        "$MessageList",
        "Lists the message names emitted by the most recent top-level evaluation.",
    ),
    BuiltinSpec::new(
        "$Packages",
        "Lists package contexts loaded into the current session.",
    ),
    BuiltinSpec::new("$Path", "Lists search roots used by the package loader."),
    BuiltinSpec::ordinary_with_hold(
        "Context",
        "Returns the context of the current session or of a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Context,
    ),
    BuiltinSpec::ordinary(
        "Names",
        "Lists symbol names matching a current first-pass name pattern subset.",
        BuiltinHandler::Names,
    ),
    BuiltinSpec::new("File", "Represents an explicit file specification."),
    BuiltinSpec::ordinary(
        "FindFile",
        "Finds the full path of a file or package using the current search path.",
        BuiltinHandler::FindFile,
    ),
    BuiltinSpec::ordinary(
        "Get",
        "Reads a file or package and evaluates the expressions it contains.",
        BuiltinHandler::Get,
    ),
    BuiltinSpec::ordinary(
        "Needs",
        "Loads a package only if its context has not already been loaded.",
        BuiltinHandler::Needs,
    ),
    BuiltinSpec::ordinary(
        "Begin",
        "Switches into a context and prepends it to the context path.",
        BuiltinHandler::Begin,
    ),
    BuiltinSpec::ordinary(
        "End",
        "Restores the previous context after a matching Begin.",
        BuiltinHandler::End,
    ),
    BuiltinSpec::ordinary(
        "BeginPackage",
        "Starts a package context with a package-oriented context path.",
        BuiltinHandler::BeginPackage,
    ),
    BuiltinSpec::ordinary(
        "EndPackage",
        "Ends a package context and leaves it on the restored context path.",
        BuiltinHandler::EndPackage,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Message",
        "Emits a message described by a MessageName expression.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Message,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageName",
        "Represents a symbolic message name such as f::tag.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageName,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Messages",
        "Lists message names defined for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Messages,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageDefinitions",
        "Lists stored message template definitions for the whole session, a symbol, or one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageDefinitions,
    ),
    BuiltinSpec::ordinary_with_hold(
        "MessageTags",
        "Lists message tags defined globally, for a symbol, or for one message query.",
        HoldAttribute::HoldAll,
        BuiltinHandler::MessageTags,
    ),
    BuiltinSpec::ordinary_with_hold(
        "OwnValues",
        "Lists stored immediate or delayed ownvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::OwnValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "UpValues",
        "Lists stored tagged upvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::UpValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "DownValues",
        "Lists stored downvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::DownValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SubValues",
        "Lists stored subvalue definitions for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SubValues,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Attributes",
        "Lists the currently active attributes for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Attributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SyntaxInformation",
        "Lists the current syntax-information metadata rules for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SyntaxInformation,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Options",
        "Lists the current default option rules for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Options,
    ),
    BuiltinSpec::ordinary_with_hold(
        "OptionValue",
        "Looks up an option value from explicit rules or a symbol's defaults.",
        HoldAttribute::HoldAll,
        BuiltinHandler::OptionValue,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SetAttributes",
        "Adds attributes to one or more symbols.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SetAttributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ClearAttributes",
        "Removes attributes from one or more symbols.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::ClearAttributes,
    ),
    BuiltinSpec::ordinary_with_hold(
        "SetOptions",
        "Updates known default options for a symbol.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::SetOptions,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Protect",
        "Adds the Protected attribute to one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Protect,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Unprotect",
        "Removes the Protected attribute from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Unprotect,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Unset",
        "Removes a specific stored definition or message template.",
        HoldAttribute::HoldFirst,
        BuiltinHandler::Unset,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Clear",
        "Clears values and rules from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Clear,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ClearAll",
        "Clears values, rules, messages, and user-defined attributes from one or more symbols.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ClearAll,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Check",
        "Evaluates an expression and substitutes a fallback if matching messages are emitted.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Check,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Quiet",
        "Evaluates an expression while suppressing matching runtime messages it emits.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Quiet,
    ),
];

pub fn core_builtin_specs() -> &'static [BuiltinSpec] {
    static CORE_BUILTINS: OnceLock<Vec<BuiltinSpec>> = OnceLock::new();

    CORE_BUILTINS
        .get_or_init(|| {
            let mut specs = Vec::with_capacity(
                CORE_BUILTINS_PREFIX.len()
                    + strings::STRING_BUILTINS.len()
                    + structure::STRUCTURE_BUILTINS.len()
                    + associations::ASSOCIATION_BUILTINS.len()
                    + control::CONTROL_BUILTINS.len()
                    + functional::FUNCTIONAL_BUILTINS.len()
                    + scoping::SCOPING_BUILTINS.len()
                    + CORE_BUILTINS_SUFFIX.len(),
            );
            specs.extend_from_slice(CORE_BUILTINS_PREFIX);
            specs.extend_from_slice(strings::STRING_BUILTINS);
            specs.extend_from_slice(structure::STRUCTURE_BUILTINS);
            specs.extend_from_slice(associations::ASSOCIATION_BUILTINS);
            specs.extend_from_slice(control::CONTROL_BUILTINS);
            specs.extend_from_slice(functional::FUNCTIONAL_BUILTINS);
            specs.extend_from_slice(scoping::SCOPING_BUILTINS);
            specs.extend_from_slice(CORE_BUILTINS_SUFFIX);
            specs
        })
        .as_slice()
}

pub(super) fn default_builtin_dispatch_spec(name: &str) -> Option<BuiltinSpec> {
    core_builtin_specs()
        .iter()
        .find(|spec| spec.name == name)
        .copied()
}

pub(super) fn default_system_attribute_set(symbol: &Symbol) -> AttributeSet {
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
