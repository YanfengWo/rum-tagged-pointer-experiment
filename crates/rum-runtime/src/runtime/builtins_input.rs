use std::thread;
use std::time::Duration;

use super::*;

impl Session {
    pub(super) fn eval_input_string(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let prompt = match args {
            [] => String::new(),
            [Expr::Atom(Atom::String(prompt))] => prompt.clone(),
            [_] => {
                return Ok(self.invalid_call_with_arg_form(
                    "InputString",
                    args,
                    "expected zero arguments or one prompt string",
                ));
            }
            _ => return Ok(self.invalid_call_with_arg_range("InputString", args, 0, 1)),
        };

        if let Some(value) = self.queued_input_responses.pop_front() {
            self.clear_suspension_state();
            return Ok(Expr::string(value));
        }

        let request = RuntimeInputRequest::new(prompt);
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
