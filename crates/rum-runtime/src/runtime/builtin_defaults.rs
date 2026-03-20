use super::*;

pub(super) fn default_builtin_dispatch_spec(name: &str) -> Option<BuiltinSpec> {
    Some(match name {
        "Identity" => BuiltinSpec::ordinary("", "", BuiltinHandler::Identity),
        "Set" => BuiltinSpec::special("", "", BuiltinHandler::Set),
        "SetDelayed" => BuiltinSpec::special("", "", BuiltinHandler::SetDelayed),
        "TagSet" => BuiltinSpec::special("", "", BuiltinHandler::TagSet),
        "TagSetDelayed" => BuiltinSpec::special("", "", BuiltinHandler::TagSetDelayed),
        "TagUnset" => BuiltinSpec::special("", "", BuiltinHandler::TagUnset),
        "Rule" => BuiltinSpec::special("", "", BuiltinHandler::Rule),
        "RuleDelayed" => BuiltinSpec::special("", "", BuiltinHandler::RuleDelayed),
        "MatchQ" => BuiltinSpec::special("", "", BuiltinHandler::MatchQ),
        "Cases" => BuiltinSpec::special("", "", BuiltinHandler::Cases),
        "Module" => BuiltinSpec::special("", "", BuiltinHandler::Module),
        "Block" => BuiltinSpec::special("", "", BuiltinHandler::Block),
        "ReplaceAll" => BuiltinSpec::special("", "", BuiltinHandler::ReplaceAll),
        "ReplaceRepeated" => BuiltinSpec::special("", "", BuiltinHandler::ReplaceRepeated),
        "FindFile" => BuiltinSpec::ordinary("", "", BuiltinHandler::FindFile),
        "Get" => BuiltinSpec::ordinary("", "", BuiltinHandler::Get),
        "Needs" => BuiltinSpec::ordinary("", "", BuiltinHandler::Needs),
        "Context" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::Context,
        ),
        "Begin" => BuiltinSpec::ordinary("", "", BuiltinHandler::Begin),
        "End" => BuiltinSpec::ordinary("", "", BuiltinHandler::End),
        "BeginPackage" => BuiltinSpec::ordinary("", "", BuiltinHandler::BeginPackage),
        "EndPackage" => BuiltinSpec::ordinary("", "", BuiltinHandler::EndPackage),
        "Message" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::Message,
        ),
        "MessageName" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::MessageName,
        ),
        "Messages" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::Messages,
        ),
        "MessageDefinitions" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::MessageDefinitions,
        ),
        "MessageTags" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::MessageTags,
        ),
        "OwnValues" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::OwnValues,
        ),
        "UpValues" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::UpValues,
        ),
        "DownValues" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::DownValues,
        ),
        "SubValues" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::SubValues,
        ),
        "Attributes" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::Attributes,
        ),
        "SetAttributes" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::SetAttributes,
        ),
        "ClearAttributes" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::ClearAttributes,
        ),
        "Unset" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldFirst, BuiltinHandler::Unset)
        }
        "Clear" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldAll, BuiltinHandler::Clear)
        }
        "ClearAll" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::ClearAll,
        ),
        "NormalizeRules" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::NormalizeRules,
        ),
        "Check" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldAll, BuiltinHandler::Check)
        }
        "Quiet" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldAll, BuiltinHandler::Quiet)
        }
        "If" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldRest, BuiltinHandler::If)
        }
        "Which" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldAll, BuiltinHandler::Which)
        }
        "Function" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::HoldLiteral,
        ),
        "With" => {
            BuiltinSpec::ordinary_with_hold("", "", HoldAttribute::HoldAll, BuiltinHandler::With)
        }
        "Hold" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::HoldLiteral,
        ),
        "HoldFirst" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldFirst,
            BuiltinHandler::HoldLiteral,
        ),
        "HoldRest" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldRest,
            BuiltinHandler::HoldLiteral,
        ),
        "HoldAll" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::HoldLiteral,
        ),
        "HoldAllComplete" => BuiltinSpec::ordinary_with_attributes(
            "",
            "",
            AttributeSet::from_attribute(Attribute::HoldAllComplete),
            BuiltinHandler::HoldLiteral,
        ),
        "Unevaluated" => BuiltinSpec::ordinary_with_attributes(
            "",
            "",
            AttributeSet::from_attribute(Attribute::HoldAllComplete),
            BuiltinHandler::Unevaluated,
        ),
        "Evaluate" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::Evaluate,
        ),
        "HoldPattern" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::HoldLiteral,
        ),
        "ReleaseHold" => BuiltinSpec::ordinary_with_hold(
            "",
            "",
            HoldAttribute::HoldAll,
            BuiltinHandler::ReleaseHold,
        ),
        "Sequence" => BuiltinSpec::ordinary("", "", BuiltinHandler::Sequence),
        "Thread" => BuiltinSpec::ordinary("", "", BuiltinHandler::Thread),
        "InputString" => BuiltinSpec::ordinary("", "", BuiltinHandler::InputString),
        _ => return None,
    })
}

pub(super) fn default_system_attribute_set(symbol: &Symbol) -> AttributeSet {
    if symbol.context() != SYSTEM_CONTEXT {
        return AttributeSet::empty();
    }

    match symbol.name() {
        "Attributes" | "Context" | "Message" | "OwnValues" | "UpValues" | "DownValues"
        | "SubValues" | "SetAttributes" | "ClearAttributes" | "Unset" => {
            AttributeSet::from_attribute(Attribute::HoldFirst)
        }
        "HoldAllComplete" | "Unevaluated" => {
            AttributeSet::from_attribute(Attribute::HoldAllComplete)
        }
        "ReleaseHold" | "HoldPattern" => AttributeSet::from_attribute(Attribute::HoldAll),
        "Hold" | "HoldAll" | "Evaluate" | "Function" | "Which" | "Module" | "Block" | "Check"
        | "Quiet" | "MessageName" | "Messages" | "MessageDefinitions" | "MessageTags"
        | "NormalizeRules" | "With" | "Clear" | "ClearAll" => {
            AttributeSet::from_attribute(Attribute::HoldAll)
        }
        "If" => AttributeSet::from_attribute(Attribute::HoldRest),
        "HoldFirst" => AttributeSet::from_attribute(Attribute::HoldFirst),
        "HoldRest" => AttributeSet::from_attribute(Attribute::HoldRest),
        "Plus" | "Times" => AttributeSet::empty()
            .with(Attribute::Listable)
            .with(Attribute::Orderless)
            .with(Attribute::Flat)
            .with(Attribute::OneIdentity),
        _ => AttributeSet::empty(),
    }
}

pub(super) fn parse_attribute_set_expr(expr: &Expr) -> Option<AttributeSet> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut attributes = AttributeSet::empty();

            for item in args {
                attributes.insert(Attribute::from_expr(item)?);
            }

            Some(attributes)
        }
        _ => Some(AttributeSet::from_attribute(Attribute::from_expr(expr)?)),
    }
}
