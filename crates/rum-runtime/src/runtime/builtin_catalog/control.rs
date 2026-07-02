use super::*;

pub(super) const CONTROL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Evaluates expressions sequentially and returns the last result.",
        HoldAttribute::HoldAll,
        BuiltinHandler::CompoundExpression,
    ),
    BuiltinSpec::ordinary_with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
        BuiltinHandler::If,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Which,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Switch",
        "Matches one evaluated expression against held form/value pairs.",
        HoldAttribute::HoldRest,
        BuiltinHandler::Switch,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Do",
        "Evaluates an expression repeatedly in the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Do,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Product",
        "Multiplies expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        BuiltinHandler::Product,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sum",
        "Adds expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        BuiltinHandler::Sum,
    ),
    BuiltinSpec::ordinary_with_hold(
        "While",
        "Evaluates a body while its test yields True.",
        HoldAttribute::HoldAll,
        BuiltinHandler::While,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Until",
        "Evaluates a body until its test yields True.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Until,
    ),
    BuiltinSpec::ordinary_with_hold(
        "For",
        "Evaluates start, test, increment, and body in the current first-pass procedural subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::For,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Table",
        "Generates lists using the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Table,
    ),
    BuiltinSpec::ordinary(
        "Abort",
        "Generates an abort that terminates the current evaluation unless caught.",
        BuiltinHandler::Abort,
    ),
    BuiltinSpec::ordinary(
        "Exit",
        "Requests host-level termination with an optional integer exit code.",
        BuiltinHandler::Exit,
    ),
    BuiltinSpec::ordinary_with_hold(
        "CheckAbort",
        "Evaluates an expression and substitutes a fallback if an Abort occurs.",
        HoldAttribute::HoldAll,
        BuiltinHandler::CheckAbort,
    ),
    BuiltinSpec::ordinary_with_hold(
        "AbortProtect",
        "Evaluates an expression while delaying any abort until the protected region exits in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::AbortProtect,
    ),
    BuiltinSpec::ordinary_with_hold(
        "WithCleanup",
        "Evaluates cleanup code even when the main expression exits abnormally in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::WithCleanup,
    ),
    BuiltinSpec::ordinary(
        "Failsafe",
        "Creates a callable wrapper that preserves failure-like inputs in the current first-pass subset.",
        BuiltinHandler::Failsafe,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Assert",
        "Represents an assertion site and supports the current session flag plus $AssertFunction hook.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Assert,
    ),
    BuiltinSpec::new(
        "$AssertFunction",
        "Stores the optional assertion failure callback used by Assert in the current first-pass subset.",
    ),
    BuiltinSpec::new(
        "PropagateAborts",
        "Represents the current first-pass CheckAbort option for rethrowing caught aborts.",
    ),
    BuiltinSpec::ordinary_with_hold(
        "Enclose",
        "Evaluates an expression and catches confirmation failures in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Enclose,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Confirm",
        "Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Confirm,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmBy",
        "Confirms that a test function returns True for an expression in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmBy,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmMatch",
        "Confirms that an expression matches a held pattern in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmMatch,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmQuiet",
        "Confirms that an evaluation emits no matching messages in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmQuiet,
    ),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmAssert",
        "Confirms that a test evaluates to True in the current first-pass subset.",
        HoldAttribute::HoldAll,
        BuiltinHandler::ConfirmAssert,
    ),
    BuiltinSpec::ordinary(
        "Return",
        "Exits the current function-style evaluation with an optional result.",
        BuiltinHandler::Return,
    ),
    BuiltinSpec::ordinary(
        "Break",
        "Exits the nearest enclosing loop.",
        BuiltinHandler::Break,
    ),
    BuiltinSpec::ordinary(
        "Continue",
        "Skips to the next iteration of the nearest enclosing loop.",
        BuiltinHandler::Continue,
    ),
    BuiltinSpec::ordinary(
        "Throw",
        "Propagates a nonlocal value, optionally tagged for Catch matching.",
        BuiltinHandler::Throw,
    ),
    BuiltinSpec::ordinary_with_hold(
        "Catch",
        "Evaluates an expression and intercepts matching Throw values.",
        HoldAttribute::HoldAll,
        BuiltinHandler::Catch,
    ),
    BuiltinSpec::ordinary(
        "Failure",
        "Represents a failure object with associated details in the current first-pass subset.",
        BuiltinHandler::Failure,
    ),
    BuiltinSpec::ordinary(
        "FailureQ",
        "Tests whether an expression is a Failure object.",
        BuiltinHandler::FailureQ,
    ),
    BuiltinSpec::ordinary(
        "Success",
        "Represents a success object with associated details in the current first-pass subset.",
        BuiltinHandler::Success,
    ),
    BuiltinSpec::ordinary(
        "TerminatedEvaluation",
        "Represents a terminated evaluation object in the current first-pass subset.",
        BuiltinHandler::TerminatedEvaluation,
    ),
];
