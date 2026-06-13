use super::*;
mod encoding;
mod patterns;
mod similarity;
mod template;
mod text;

pub(in crate::runtime::strings) use template::string_form_value;

pub(in crate::runtime) fn render_input_like_expr(expr: &Expr) -> String {
    text::render_input_like_expr(expr)
}

fn bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

fn string_slot_expr() -> Expr {
    Expr::call(system_expr("Slot"), vec![Expr::integer(BigInt::from(1_u8))])
}

fn string_unary_operator_form(symbol: &str, operator: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![operator.clone(), string_slot_expr()],
        )],
    )
}

fn string_unary_operator_form_slot_first(symbol: &str, tail: &Expr) -> Expr {
    Expr::call(
        system_expr("Function"),
        vec![Expr::call(
            system_expr(symbol),
            vec![string_slot_expr(), tail.clone()],
        )],
    )
}

#[derive(Clone)]
struct StringApplyEvalState {
    head: Expr,
    operator: Expr,
    values: Vec<String>,
    list_output: bool,
}

fn string_apply_value(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    value: &str,
) -> Result<Expr, EvalError> {
    session.resume_string_apply_from_index(
        StringApplyEvalState {
            head: head.clone(),
            operator: operator.clone(),
            values: vec![value.to_string()],
            list_output: false,
        },
        0,
        Vec::with_capacity(1),
    )
}

impl Session {
    fn resume_string_apply_from_index(
        &mut self,
        state: StringApplyEvalState,
        index: usize,
        outputs: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let Some(value) = state.values.get(index) else {
            return Ok(if state.list_output {
                list_expr(outputs)
            } else {
                outputs
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| Expr::call(state.head, Vec::new()))
            });
        };

        let applied = Expr::call(
            state.operator.clone(),
            vec![list_expr(
                value
                    .chars()
                    .map(|ch| Expr::string(ch.to_string()))
                    .collect(),
            )],
        );
        let suspended_state = state.clone();
        let suspended_outputs = outputs.clone();
        let evaluated_result = self.evaluate(applied);
        let evaluated = self.suspend_with_frame(evaluated_result, move |session, evaluated| {
            session.resume_string_apply_with_value(
                suspended_state,
                index,
                suspended_outputs,
                evaluated,
            )
        })?;

        self.resume_string_apply_with_value(state, index, outputs, evaluated)
    }

    fn resume_string_apply_with_value(
        &mut self,
        state: StringApplyEvalState,
        index: usize,
        mut outputs: Vec<Expr>,
        evaluated: Expr,
    ) -> Result<Expr, EvalError> {
        let fallback = state
            .values
            .get(index)
            .map(|value| {
                Expr::call(
                    state.head.clone(),
                    vec![state.operator.clone(), Expr::string(value.clone())],
                )
            })
            .unwrap_or_else(|| Expr::call(state.head.clone(), Vec::new()));
        outputs.push(string_apply_output_expr(evaluated).unwrap_or(fallback));
        self.resume_string_apply_from_index(state, index + 1, outputs)
    }

    fn resume_string_apply_values(
        &mut self,
        head: &Expr,
        operator: &Expr,
        values: Vec<String>,
    ) -> Result<Expr, EvalError> {
        let capacity = values.len();
        self.resume_string_apply_from_index(
            StringApplyEvalState {
                head: head.clone(),
                operator: operator.clone(),
                values,
                list_output: true,
            },
            0,
            Vec::with_capacity(capacity),
        )
    }
}

fn string_apply_output_expr(expr: Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::String(_)) => Some(expr),
        Expr::Call { head, args } if is_system_symbol_name(&head, "List") => {
            if args
                .iter()
                .all(|item| matches!(item, Expr::Atom(Atom::String(_))))
            {
                let joined = args
                    .into_iter()
                    .filter_map(|item| match item {
                        Expr::Atom(Atom::String(value)) => Some(value),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .concat();
                return Some(Expr::string(joined));
            }

            let converted = args
                .into_iter()
                .map(string_apply_output_expr)
                .collect::<Option<Vec<_>>>()?;
            Some(list_expr(converted))
        }
        _ => None,
    }
}

fn is_string_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '\r')
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn string_list_expr(items: Vec<String>) -> Expr {
    list_expr(items.into_iter().map(Expr::string).collect())
}
