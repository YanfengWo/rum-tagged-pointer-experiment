use super::parsing::parse_test_create_options;
use super::*;

impl Session {
    pub(in crate::runtime) fn eval_test_create(
        &mut self,
        head: &Expr,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let (input, expected, options_expr) = match args.as_ref() {
            [input, expected] => (input.clone(), expected.clone(), None),
            [input, expected, third] => (input.clone(), expected.clone(), Some(third.clone())),
            _ => return Ok(Expr::call(head.clone(), args.to_vec())),
        };

        let mut test_id = None;
        let mut expected_messages = None;
        let mut same_test = None;

        if let Some(options_expr) = options_expr {
            if let Some(options) = parse_test_create_options(&options_expr) {
                for (name, value) in options {
                    match name.as_str() {
                        "TestID" => test_id = Some(value),
                        "ExpectedMessages" => expected_messages = Some(value),
                        "SameTest" => same_test = Some(value),
                        _ => return Ok(Expr::call(head.clone(), args.to_vec())),
                    }
                }
            } else {
                test_id = Some(options_expr);
            }
        }

        let mut entries = vec![
            (
                Expr::string("Input"),
                Expr::call(system_expr("Hold"), vec![input]),
            ),
            (
                Expr::string("ExpectedOutput"),
                Expr::call(system_expr("Hold"), vec![expected]),
            ),
        ];

        if let Some(test_id) = test_id {
            entries.push((Expr::string("TestID"), test_id));
        }
        if let Some(expected_messages) = expected_messages {
            entries.push((
                Expr::string("ExpectedMessages"),
                Expr::call(system_expr("Hold"), vec![expected_messages]),
            ));
        }
        if let Some(same_test) = same_test {
            entries.push((Expr::string("SameTest"), same_test));
        }

        Ok(render_association(entries))
    }
}
