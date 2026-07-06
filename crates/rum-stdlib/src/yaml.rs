use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{EvalError, Session, expr_as_string, system_expr};

mod decode;
mod encode;
mod number;

#[derive(Clone, Debug, PartialEq, Eq)]
struct YamlParseError {
    offset: usize,
    message: String,
}

impl YamlParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        format!(
            "Invalid YAML at byte offset {}: {}.",
            self.offset, self.message
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct YamlEncodeError {
    message: String,
}

impl YamlEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

pub(crate) fn eval_from_yaml(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [source] = args else {
        return Ok(fail_yaml(
            session,
            "FromYaml",
            "FromYaml expects exactly one string argument.",
        ));
    };

    let Some(source) = expr_as_string(source) else {
        return Ok(fail_yaml(
            session,
            "FromYaml",
            "FromYaml expects a YAML text string.",
        ));
    };

    match decode::from_yaml(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_yaml(session, "FromYaml", error.message())),
    }
}

pub(crate) fn eval_to_yaml(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(fail_yaml(
            session,
            "ToYaml",
            "ToYaml expects exactly one expression argument.",
        ));
    };

    match encode::to_yaml(expr) {
        Ok(output) => Ok(Expr::string(output)),
        Err(error) => Ok(fail_yaml(session, "ToYaml", error.message())),
    }
}

fn fail_yaml(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "yaml", message);
    system_expr("$Failed")
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

fn association_expr(entries: Vec<(String, Expr)>) -> Expr {
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![Expr::string(key), value]))
            .collect(),
    )
}

fn symbol_value(name: &str) -> Expr {
    system_expr(name)
}

fn string_atom(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::String(value)) => Some(value),
        _ => None,
    }
}
