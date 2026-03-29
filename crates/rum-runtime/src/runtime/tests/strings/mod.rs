use super::*;

fn eval_to_string(session: &mut Session, input: &str) -> String {
    session.eval_text(input).unwrap().value.to_string()
}

mod advanced_patterns;
mod encoding;
mod parts;
mod search_replace;
mod similarity;
mod surface;
