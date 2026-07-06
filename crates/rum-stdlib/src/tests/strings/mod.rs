use super::*;
use rum_core::{BigInt, Expr};
use rum_runtime::kernel_api::system_expr;

fn core_session() -> Session {
    let mut session = Session::new();
    install_test_file_io(&mut session);
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
mod scripting;
mod search_replace;
mod similarity;
mod surface;
mod text_odds;
mod tokenize;
