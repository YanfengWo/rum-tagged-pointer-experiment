use rum_host::{HostEvalOutcome, HostSession};

fn main() {
    let mut session = HostSession::new();

    match session.eval("rule[x_Integer] := x + 1; rule[41]") {
        HostEvalOutcome::Success(success) => {
            println!("value: {}", success.value.text);
        }
        HostEvalOutcome::InputRequired(suspension) => {
            println!("input required: {}", suspension.request.prompt);
        }
        HostEvalOutcome::Failure(failure) => {
            eprintln!("error: {}", failure.message);
        }
    }
}
