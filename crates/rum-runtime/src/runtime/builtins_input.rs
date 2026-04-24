use std::thread;
use std::time::Duration;

use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct DialogConfig {
    body: Option<Expr>,
    prolog: Expr,
    symbols: Vec<(Symbol, Option<Expr>)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogNormalizationError {
    InvalidOptions,
    InvalidSymbols,
}

impl DialogNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidOptions => {
                "expected only DialogProlog and DialogSymbols option rules in the current first-pass subset"
            }
            Self::InvalidSymbols => {
                "expected DialogSymbols to be a list of symbols or symbol initializers in the current first-pass subset"
            }
        }
    }
}

impl Session {
    pub(super) fn eval_dialog(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let config = match self.compile_dialog_call(args) {
            Ok(config) => config,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Dialog", args, error.message()));
            }
        };

        let body = self.dialog_body_expr(config.body);
        let body = if is_system_symbol_name(&config.prolog, "Null") {
            body
        } else {
            Expr::call(system_expr("CompoundExpression"), vec![config.prolog, body])
        };
        let body = if config.symbols.is_empty() {
            body
        } else {
            Expr::call(
                system_expr("Block"),
                vec![render_dialog_symbols_bindings(&config.symbols), body],
            )
        };

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame(body_result, |session, outcome| {
            session.finish_dialog_body(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_dialog_body(outcome),
        }
    }

    pub(super) fn eval_input_string(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let request = match self.compile_input_string_request(args) {
            Some(request) => request,
            None => {
                return Ok(self.invalid_call_with_arg_form(
                    "InputString",
                    args,
                    "expected an optional prompt followed by FieldMasked option rules",
                ));
            }
        };

        if let Some(value) = self.queued_input_responses.pop_front() {
            self.clear_suspension_state();
            return Ok(Expr::string(value));
        }

        self.begin_input_suspension(request.clone());

        Err(EvalError::InputRequired { request })
    }

    pub(super) fn eval_input(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let prompt = match args {
            [] => "?".to_string(),
            [Expr::Atom(Atom::String(prompt))] => prompt.clone(),
            [prompt] => prompt.to_string(),
            _ => return Ok(self.invalid_call_with_arg_range("Input", args, 0, 1)),
        };

        let text_result = self.eval_input_string(&[Expr::string(prompt)]);
        let text = self.suspend_with_frame(text_result, |session, text| {
            session.resume_input_after_text(text)
        })?;

        self.resume_input_after_text(text)
    }

    pub(super) fn eval_print(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let rendered = args
            .iter()
            .map(render_print_fragment)
            .collect::<Vec<_>>()
            .join("");
        self.push_print(rendered);
        Ok(system_expr("Null"))
    }

    pub(super) fn eval_pause(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [seconds] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Pause", args, 1));
        };

        let Some(duration) = expr_as_nonnegative_duration(seconds) else {
            return Ok(self.invalid_call_with_arg_form(
                "Pause",
                args,
                "expected one nonnegative exact numeric duration",
            ));
        };

        if !duration.is_zero() {
            thread::sleep(duration);
        }

        Ok(system_expr("Null"))
    }

    fn compile_dialog_call(
        &mut self,
        args: &[Expr],
    ) -> Result<DialogConfig, DialogNormalizationError> {
        let (body, option_exprs) = split_dialog_body_and_options(args);
        let explicit_options = if option_exprs.is_empty() {
            None
        } else {
            let options = self
                .compile_assigned_options_from_resolved(&Expr::call(
                    system_expr("List"),
                    option_exprs.to_vec(),
                ))
                .map_err(|_| DialogNormalizationError::InvalidOptions)?;
            Some(options)
        };

        if let Some(options) = explicit_options.as_ref() {
            for (name, _, _) in options.iter() {
                if !matches!(name.name(), "DialogProlog" | "DialogSymbols") {
                    return Err(DialogNormalizationError::InvalidOptions);
                }
            }
        }

        let prolog = self.dialog_option_raw_expr(explicit_options.as_ref(), "DialogProlog")?;
        let symbols_expr =
            self.dialog_option_raw_expr(explicit_options.as_ref(), "DialogSymbols")?;
        let symbols = match symbols_expr {
            Some((expr, _delayed)) => {
                parse_module_binding_list(&expr).ok_or(DialogNormalizationError::InvalidSymbols)?
            }
            None => Vec::new(),
        };

        Ok(DialogConfig {
            body: body.cloned(),
            prolog: prolog
                .map(|(expr, _delayed)| expr)
                .unwrap_or_else(|| system_expr("Null")),
            symbols,
        })
    }

    fn compile_input_string_request(&self, args: &[Expr]) -> Option<RuntimeInputRequest> {
        let (prompt, option_exprs) = split_input_string_prompt_and_options(args);
        let prompt = prompt.map_or_else(String::new, render_print_fragment);
        let explicit_options = if option_exprs.is_empty() {
            None
        } else {
            let options = self
                .compile_assigned_options_from_resolved(&Expr::call(
                    system_expr("List"),
                    option_exprs.to_vec(),
                ))
                .ok()?;
            for (name, _, _) in options.iter() {
                if name.name() != "FieldMasked" {
                    return None;
                }
            }
            Some(options)
        };

        let mut request = RuntimeInputRequest::new(prompt);
        if explicit_options
            .as_ref()
            .and_then(|options| options.get_by_name("FieldMasked"))
            .is_some_and(|value| is_system_symbol_name(value, "True"))
        {
            request = request.password();
        }

        Some(request)
    }

    fn dialog_body_expr(&self, body: Option<Expr>) -> Expr {
        body.unwrap_or_else(|| Expr::call(system_expr("Input"), vec![Expr::string("")]))
    }

    fn dialog_option_raw_expr(
        &mut self,
        explicit: Option<&SymbolOptions>,
        option_name: &str,
    ) -> Result<Option<(Expr, bool)>, DialogNormalizationError> {
        Ok(self.raw_option_value_for_symbol(explicit, &Symbol::system("Dialog"), option_name))
    }

    fn finish_dialog_body(&mut self, outcome: Result<Expr, EvalError>) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        }
    }

    fn resume_input_after_text(&mut self, text: Expr) -> Result<Expr, EvalError> {
        let Expr::Atom(Atom::String(source)) = text else {
            return Ok(text);
        };

        let syntax = parse_expr(&source)?;
        let expr = self.lower_syntax_expr(&syntax)?;
        let evaluated = self.evaluate(expr);
        self.suspend_with_frame(evaluated, |_, value| Ok(value))
    }
}

fn split_input_string_prompt_and_options(args: &[Expr]) -> (Option<&Expr>, &[Expr]) {
    match args {
        [] => (None, &[]),
        [first, ..] if is_input_string_option_expr(first) => (None, args),
        [first, rest @ ..] => (Some(first), rest),
    }
}

fn is_input_string_option_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            matches!(
                &args[0],
                Expr::Atom(Atom::Symbol(name)) if name.name() == "FieldMasked"
            ) || matches!(
                &args[0],
                Expr::Atom(Atom::String(name)) if name == "FieldMasked"
            )
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            !args.is_empty() && args.iter().all(is_input_string_option_expr)
        }
        _ => false,
    }
}

fn split_dialog_body_and_options(args: &[Expr]) -> (Option<&Expr>, &[Expr]) {
    match args {
        [] => (None, &[]),
        [first, ..] if is_dialog_option_expr(first) => (None, args),
        [first, rest @ ..] => (Some(first), rest),
    }
}

fn is_dialog_option_expr(expr: &Expr) -> bool {
    let Expr::Call { head, args } = expr else {
        return false;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return false;
    }

    matches!(
        &args[0],
        Expr::Atom(Atom::Symbol(name)) if matches!(name.name(), "DialogProlog" | "DialogSymbols")
    ) || matches!(
        &args[0],
        Expr::Atom(Atom::String(name)) if matches!(name.as_str(), "DialogProlog" | "DialogSymbols")
    )
}

fn render_dialog_symbols_bindings(items: &[(Symbol, Option<Expr>)]) -> Expr {
    Expr::call(
        system_expr("List"),
        items
            .iter()
            .map(|(symbol, init)| match init {
                Some(init) => Expr::call(
                    system_expr("Rule"),
                    vec![Expr::Atom(Atom::Symbol(symbol.clone())), init.clone()],
                ),
                None => Expr::Atom(Atom::Symbol(symbol.clone())),
            })
            .collect(),
    )
}

fn render_print_fragment(expr: &Expr) -> String {
    match expr {
        Expr::Atom(Atom::String(value)) => value.clone(),
        other => other.to_string(),
    }
}

fn expr_as_nonnegative_duration(expr: &Expr) -> Option<Duration> {
    let seconds = match expr {
        Expr::Atom(Atom::Integer(value)) => value.to_string().parse::<f64>().ok()?,
        Expr::Atom(Atom::Rational(value)) => {
            let numerator = value.numerator().to_string().parse::<f64>().ok()?;
            let denominator = value.denominator().to_string().parse::<f64>().ok()?;
            numerator / denominator
        }
        _ => return None,
    };

    if !seconds.is_finite() || seconds < 0.0 {
        return None;
    }

    Some(Duration::from_secs_f64(seconds))
}
