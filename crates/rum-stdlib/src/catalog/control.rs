use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec, Capability, HoldAttribute};

use crate::control_flow;

pub(crate) const CONTROL_BUILTINS: &[BuiltinSpec] = &[
    BuiltinSpec::ordinary_with_hold(
        "CompoundExpression",
        "Evaluates expressions sequentially and returns the last result.",
        HoldAttribute::HoldAll,
        control_flow::eval_compound_expression,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "If",
        "Evaluates one of several branches conditionally.",
        HoldAttribute::HoldRest,
        control_flow::eval_if,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Which",
        "Evaluates conditions in order and returns the first matching branch.",
        HoldAttribute::HoldAll,
        control_flow::eval_which,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Switch",
        "Matches one evaluated expression against held form/value pairs.",
        HoldAttribute::HoldRest,
        control_flow::eval_switch,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Do",
        "Evaluates an expression repeatedly in the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_do,
    ).protected(),
    BuiltinSpec::ordinary_with_attributes(
        "Product",
        "Multiplies expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        control_flow::eval_product,
    ),
    BuiltinSpec::ordinary_with_attributes(
        "Sum",
        "Adds expression values over concrete iterator specifications.",
        AttributeSet::from_hold(HoldAttribute::HoldAll)
            .with(Attribute::Protected)
            .with(Attribute::ReadProtected),
        control_flow::eval_sum,
    ),
    BuiltinSpec::ordinary_with_hold(
        "While",
        "Evaluates a body while its test yields True.",
        HoldAttribute::HoldAll,
        control_flow::eval_while,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Until",
        "Evaluates a body until its test yields True.",
        HoldAttribute::HoldAll,
        control_flow::eval_until,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "For",
        "Evaluates start, test, increment, and body in the current first-pass procedural subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_for,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Table",
        "Generates lists using the current first-pass iterator subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_table,
    ).protected(),
    BuiltinSpec::ordinary(
        "Abort",
        "Generates an abort that terminates the current evaluation unless caught.",
        control_flow::eval_abort,
    ).protected(),
    BuiltinSpec::ordinary(
        "Exit",
        "Requests host-level termination with an optional integer exit code.",
        control_flow::eval_exit,
    ).protected()
    .with_capability(Capability::Process),
    BuiltinSpec::ordinary_with_hold(
        "CheckAbort",
        "Evaluates an expression and substitutes a fallback if an Abort occurs.",
        HoldAttribute::HoldAll,
        control_flow::eval_check_abort,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "AbortProtect",
        "Evaluates an expression while delaying any abort until the protected region exits in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_abort_protect,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "WithCleanup",
        "Evaluates cleanup code even when the main expression exits abnormally in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_with_cleanup,
    ).protected(),
    BuiltinSpec::ordinary(
        "Failsafe",
        "Creates a callable wrapper that preserves failure-like inputs in the current first-pass subset.",
        control_flow::eval_failsafe,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Assert",
        "Represents an assertion site and supports the current session flag plus $AssertFunction hook.",
        HoldAttribute::HoldAll,
        control_flow::eval_assert,
    ).protected(),
    BuiltinSpec::new(
        "$AssertFunction",
        "Stores the optional assertion failure callback used by Assert in the current first-pass subset.",
    ),
    BuiltinSpec::new(
        "PropagateAborts",
        "Represents the current first-pass CheckAbort option for rethrowing caught aborts.",
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Enclose",
        "Evaluates an expression and catches confirmation failures in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_enclose,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Confirm",
        "Confirms that an expression does not evaluate to a failure-like result in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmBy",
        "Confirms that a test function returns True for an expression in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_by,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmMatch",
        "Confirms that an expression matches a held pattern in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_match,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmQuiet",
        "Confirms that an evaluation emits no matching messages in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_quiet,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "ConfirmAssert",
        "Confirms that a test evaluates to True in the current first-pass subset.",
        HoldAttribute::HoldAll,
        control_flow::eval_confirm_assert,
    ).protected(),
    BuiltinSpec::ordinary(
        "Return",
        "Exits the current function-style evaluation with an optional result.",
        control_flow::eval_return,
    ).protected(),
    BuiltinSpec::ordinary(
        "Break",
        "Exits the nearest enclosing loop.",
        control_flow::eval_break,
    ).protected(),
    BuiltinSpec::ordinary(
        "Continue",
        "Skips to the next iteration of the nearest enclosing loop.",
        control_flow::eval_continue,
    ).protected(),
    BuiltinSpec::ordinary(
        "Throw",
        "Propagates a nonlocal value, optionally tagged for Catch matching.",
        control_flow::eval_throw,
    ).protected(),
    BuiltinSpec::ordinary_with_hold(
        "Catch",
        "Evaluates an expression and intercepts matching Throw values.",
        HoldAttribute::HoldAll,
        control_flow::eval_catch,
    ).protected(),
    BuiltinSpec::ordinary(
        "Failure",
        "Represents a failure object with associated details in the current first-pass subset.",
        control_flow::eval_failure,
    ).protected(),
    BuiltinSpec::ordinary(
        "FailureQ",
        "Tests whether an expression is a Failure object.",
        control_flow::eval_failure_q,
    ).protected(),
    BuiltinSpec::ordinary(
        "Success",
        "Represents a success object with associated details in the current first-pass subset.",
        control_flow::eval_success,
    ).protected(),
    BuiltinSpec::ordinary(
        "TerminatedEvaluation",
        "Represents a terminated evaluation object in the current first-pass subset.",
        control_flow::eval_terminated_evaluation,
    ).protected(),
];
