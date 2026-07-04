use super::*;

mod cache;
mod compiler;
mod executor;
mod model;
mod safety;
#[cfg(test)]
mod tests;

use cache::{cached_rhs_template, rhs_template_enabled};
#[cfg(debug_assertions)]
use compiler::debug_assert_template_matches_tree_walk;
use compiler::slot_values_for_code;
use executor::execute_compiled_body;
pub(super) use model::{PreparedCompiledRhsTemplate, PreparedRhsTemplate};

pub(super) fn prepare_rule_bindings_for_rhs_with_template(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    evaluation_epoch: u64,
) -> PreparedRhsTemplate {
    if !rhs_template_enabled() {
        return PreparedRhsTemplate::Substituted(substitute_rule_bindings_for_rhs(
            expr,
            bindings,
            may_splice_without_bindings,
        ));
    }

    prepare_rule_bindings_for_rhs_with_template_slow(
        expr,
        bindings,
        may_splice_without_bindings,
        evaluation_epoch,
    )
}

pub(super) fn trace_rhs_for_prepared_template(
    prepared: &PreparedRhsTemplate,
    original: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    include_payload: bool,
) -> Expr {
    match prepared {
        PreparedRhsTemplate::Substituted(expr) => expr.clone(),
        PreparedRhsTemplate::Compiled(_) if include_payload => {
            substitute_rule_bindings_for_rhs(original, bindings, may_splice_without_bindings)
        }
        PreparedRhsTemplate::Compiled(_) => original.clone(),
    }
}

pub(super) fn evaluate_prepared_rhs_template(
    session: &mut Session,
    prepared: PreparedRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    match prepared {
        PreparedRhsTemplate::Substituted(rhs) => session.evaluate_definition_rhs(option_scope, rhs),
        PreparedRhsTemplate::Compiled(prepared) => {
            let result = execute_compiled_body(session, prepared, option_scope);
            let outcome = session.suspend_with_result_frame_lazy(result, || {
                move |_, outcome| finish_compiled_definition_rhs_result(outcome)
            });

            match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => finish_compiled_definition_rhs_result(outcome),
            }
        }
    }
}

fn prepare_rule_bindings_for_rhs_with_template_slow(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    evaluation_epoch: u64,
) -> PreparedRhsTemplate {
    if let Some(code) = cached_rhs_template(expr, bindings, evaluation_epoch) {
        if let Some(slot_values) = slot_values_for_code(&code, bindings) {
            #[cfg(debug_assertions)]
            debug_assert_template_matches_tree_walk(
                &code,
                &slot_values,
                expr,
                bindings,
                may_splice_without_bindings,
            );
            return PreparedRhsTemplate::Compiled(PreparedCompiledRhsTemplate {
                code,
                slot_values,
            });
        }
    }

    PreparedRhsTemplate::Substituted(substitute_rule_bindings_for_rhs(
        expr,
        bindings,
        may_splice_without_bindings,
    ))
}

fn finish_compiled_definition_rhs_result(
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    match outcome {
        Ok(value) => Ok(value),
        Err(EvalError::Return { value }) => Ok(value),
        Err(error) => Err(error),
    }
}
