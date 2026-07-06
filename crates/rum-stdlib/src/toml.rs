use rum_core::{Atom, Expr};
use rum_runtime::kernel_api::{EvalError, Session, system_expr};

mod decode;
mod encode;
mod model;
mod number;
mod temporal;

#[derive(Clone, Debug, PartialEq, Eq)]
struct TomlParseError {
    offset: usize,
    message: String,
}

impl TomlParseError {
    fn new(offset: usize, message: impl Into<String>) -> Self {
        Self {
            offset,
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        format!(
            "Invalid TOML at byte offset {}: {}.",
            self.offset, self.message
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TomlEncodeError {
    message: String,
}

impl TomlEncodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

pub(crate) fn eval_from_toml(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [source] = args else {
        return Ok(fail_toml(
            session,
            "FromToml",
            "FromToml expects exactly one string argument.",
        ));
    };

    let Expr::Atom(Atom::String(source)) = source else {
        return Ok(fail_toml(
            session,
            "FromToml",
            "FromToml expects a TOML text string.",
        ));
    };

    match decode::from_toml(source) {
        Ok(expr) => Ok(expr),
        Err(error) => Ok(fail_toml(session, "FromToml", error.message())),
    }
}

pub(crate) fn eval_to_toml(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr] = args else {
        return Ok(fail_toml(
            session,
            "ToToml",
            "ToToml expects exactly one Association argument.",
        ));
    };

    match encode::to_toml(expr) {
        Ok(output) => Ok(Expr::string(output)),
        Err(error) => Ok(fail_toml(session, "ToToml", error.message())),
    }
}

fn fail_toml(session: &mut Session, symbol: &str, message: impl Into<String>) -> Expr {
    session.push_message(symbol, "toml", message);
    system_expr("$Failed")
}
