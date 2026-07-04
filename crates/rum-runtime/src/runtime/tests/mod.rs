use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("rum-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, source: &str) -> PathBuf {
        let path = self.path.join(relative);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        fs::write(&path, source).unwrap();
        path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn rum_path(path: &Path) -> String {
    normalize_found_path(path)
        .display()
        .to_string()
        .replace('\\', "/")
}

macro_rules! test_control_handler {
    ($name:ident, $builtin:ident) => {
        fn $name(session: &mut Session, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
            session.eval_control_data_builtin(ControlDataBuiltin::$builtin, head, args)
        }
    };
}

test_control_handler!(test_control_compound_expression, CompoundExpression);
test_control_handler!(test_control_if, If);
test_control_handler!(test_control_which, Which);
test_control_handler!(test_control_switch, Switch);
test_control_handler!(test_control_do, Do);
test_control_handler!(test_control_product, Product);
test_control_handler!(test_control_sum, Sum);
test_control_handler!(test_control_while, While);
test_control_handler!(test_control_until, Until);
test_control_handler!(test_control_for, For);
test_control_handler!(test_control_table, Table);
test_control_handler!(test_control_abort, Abort);
test_control_handler!(test_control_exit, Exit);
test_control_handler!(test_control_check_abort, CheckAbort);
test_control_handler!(test_control_abort_protect, AbortProtect);
test_control_handler!(test_control_with_cleanup, WithCleanup);
test_control_handler!(test_control_failsafe, Failsafe);
test_control_handler!(test_control_assert, Assert);
test_control_handler!(test_control_enclose, Enclose);
test_control_handler!(test_control_confirm, Confirm);
test_control_handler!(test_control_confirm_by, ConfirmBy);
test_control_handler!(test_control_confirm_match, ConfirmMatch);
test_control_handler!(test_control_confirm_quiet, ConfirmQuiet);
test_control_handler!(test_control_confirm_assert, ConfirmAssert);
test_control_handler!(test_control_return, Return);
test_control_handler!(test_control_break, Break);
test_control_handler!(test_control_continue, Continue);
test_control_handler!(test_control_throw, Throw);
test_control_handler!(test_control_catch, Catch);
test_control_handler!(test_control_failure, Failure);
test_control_handler!(test_control_failure_q, FailureQ);
test_control_handler!(test_control_success, Success);
test_control_handler!(test_control_terminated_evaluation, TerminatedEvaluation);

fn register_test_control_builtins(session: &mut Session) {
    let specs = [
        BuiltinSpec::ordinary_with_hold(
            "CompoundExpression",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_compound_expression,
        ),
        BuiltinSpec::ordinary_with_hold(
            "If",
            "Test-local moved control builtin.",
            HoldAttribute::HoldRest,
            test_control_if,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Which",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_which,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Switch",
            "Test-local moved control builtin.",
            HoldAttribute::HoldRest,
            test_control_switch,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Do",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_do,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Product",
            "Test-local moved control builtin.",
            AttributeSet::from_hold(HoldAttribute::HoldAll)
                .with(Attribute::Protected)
                .with(Attribute::ReadProtected),
            test_control_product,
        ),
        BuiltinSpec::ordinary_with_attributes(
            "Sum",
            "Test-local moved control builtin.",
            AttributeSet::from_hold(HoldAttribute::HoldAll)
                .with(Attribute::Protected)
                .with(Attribute::ReadProtected),
            test_control_sum,
        ),
        BuiltinSpec::ordinary_with_hold(
            "While",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_while,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Until",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_until,
        ),
        BuiltinSpec::ordinary_with_hold(
            "For",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_for,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Table",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_table,
        ),
        BuiltinSpec::ordinary(
            "Abort",
            "Test-local moved control builtin.",
            test_control_abort,
        ),
        BuiltinSpec::ordinary(
            "Exit",
            "Test-local moved control builtin.",
            test_control_exit,
        ),
        BuiltinSpec::ordinary_with_hold(
            "CheckAbort",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_check_abort,
        ),
        BuiltinSpec::ordinary_with_hold(
            "AbortProtect",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_abort_protect,
        ),
        BuiltinSpec::ordinary_with_hold(
            "WithCleanup",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_with_cleanup,
        ),
        BuiltinSpec::ordinary(
            "Failsafe",
            "Test-local moved control builtin.",
            test_control_failsafe,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Assert",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_assert,
        ),
        BuiltinSpec::new(
            "$AssertFunction",
            "Test-local moved control-support symbol.",
        ),
        BuiltinSpec::new(
            "PropagateAborts",
            "Test-local moved control-support symbol.",
        ),
        BuiltinSpec::ordinary_with_hold(
            "Enclose",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_enclose,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Confirm",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_confirm,
        ),
        BuiltinSpec::ordinary_with_hold(
            "ConfirmBy",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_confirm_by,
        ),
        BuiltinSpec::ordinary_with_hold(
            "ConfirmMatch",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_confirm_match,
        ),
        BuiltinSpec::ordinary_with_hold(
            "ConfirmQuiet",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_confirm_quiet,
        ),
        BuiltinSpec::ordinary_with_hold(
            "ConfirmAssert",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_confirm_assert,
        ),
        BuiltinSpec::ordinary(
            "Return",
            "Test-local moved control builtin.",
            test_control_return,
        ),
        BuiltinSpec::ordinary(
            "Break",
            "Test-local moved control builtin.",
            test_control_break,
        ),
        BuiltinSpec::ordinary(
            "Continue",
            "Test-local moved control builtin.",
            test_control_continue,
        ),
        BuiltinSpec::ordinary(
            "Throw",
            "Test-local moved control builtin.",
            test_control_throw,
        ),
        BuiltinSpec::ordinary_with_hold(
            "Catch",
            "Test-local moved control builtin.",
            HoldAttribute::HoldAll,
            test_control_catch,
        ),
        BuiltinSpec::ordinary(
            "Failure",
            "Test-local moved control builtin.",
            test_control_failure,
        ),
        BuiltinSpec::ordinary(
            "FailureQ",
            "Test-local moved control builtin.",
            test_control_failure_q,
        ),
        BuiltinSpec::ordinary(
            "Success",
            "Test-local moved control builtin.",
            test_control_success,
        ),
        BuiltinSpec::ordinary(
            "TerminatedEvaluation",
            "Test-local moved control builtin.",
            test_control_terminated_evaluation,
        ),
    ];

    for spec in specs {
        session.register_builtin(spec);
    }
}

fn session_with_test_control_builtins() -> Session {
    let mut session = Session::new();
    register_test_control_builtins(&mut session);
    session
}

mod arithmetic;
mod context_and_loader;
mod debug_tools;
mod definitions_and_rules;
mod definitions_and_rules_cases;
mod definitions_and_rules_part2;
mod definitions_and_rules_part3;
mod definitions_and_rules_part4;
mod definitions_and_rules_patterns;
mod definitions_and_rules_rewrite_inspect;
mod definitions_and_rules_tags;
mod holding_and_control_io;
mod holding_and_control_part2;
mod lowering_and_context;
mod lowering_golden;
mod messages;
mod messages_part2;
mod messages_part3;
mod namespace_names;
mod nothing;
mod options_and_protection;
mod package_tools;
mod pattern_matching_orderless_optional;
mod registered_builtin_resolution;
mod scoping_gensym_gc;
mod scoping_unique;
mod session_config;
mod session_requests;
mod structure_data;
mod syntax_information_debug_catalog;
mod syntax_information_lifecycle;
mod thread_and_dispatch;
mod thread_and_dispatch_part2;
mod traversal_part2;
