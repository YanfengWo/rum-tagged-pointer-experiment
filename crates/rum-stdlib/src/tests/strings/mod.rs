use super::*;
use rum_core::{Atom, BigInt, Expr};
use rum_runtime::kernel_api::system_expr;

fn core_session() -> Session {
    let mut session = Session::new();
    install_core(&mut session);
    session
}

fn eval_to_string(session: &mut Session, input: &str) -> String {
    session.eval_text(input).unwrap().value.to_string()
}

mod advanced_patterns;
mod encoding;
mod incidental_runtime;
mod parts;
mod search_replace;
mod similarity;
mod surface;
