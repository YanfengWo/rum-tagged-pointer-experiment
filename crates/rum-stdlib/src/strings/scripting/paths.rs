use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_file_extension(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [path] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FileExtension", args, 1));
    };
    let Some(path) = expr_as_string(path) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(file_extension(path)))
}

pub(crate) fn eval_file_base_name(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [path] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FileBaseName", args, 1));
    };
    let Some(path) = expr_as_string(path) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(file_base_name(path)))
}

pub(crate) fn eval_file_name_split(
    session: &mut Session,
    head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let [path] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("FileNameSplit", args, 1));
    };
    let Some(path) = expr_as_string(path) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(string_list_expr(file_name_split(path)))
}

fn file_extension(path: &str) -> String {
    let leaf = file_leaf(path);
    let Some(index) = extension_dot_index(&leaf) else {
        return String::new();
    };
    leaf[index + 1..].to_string()
}

fn file_base_name(path: &str) -> String {
    let leaf = file_leaf(path);
    let Some(index) = extension_dot_index(&leaf) else {
        return leaf;
    };
    leaf[..index].to_string()
}

fn file_leaf(path: &str) -> String {
    path.trim_end_matches('/')
        .rsplit('/')
        .find(|component| !component.is_empty())
        .unwrap_or("")
        .to_string()
}

fn extension_dot_index(leaf: &str) -> Option<usize> {
    let index = leaf.rfind('.')?;
    (index > 0 && index + 1 < leaf.len()).then_some(index)
}

fn file_name_split(path: &str) -> Vec<String> {
    if path.is_empty() {
        return Vec::new();
    }

    let mut parts = Vec::new();
    if path.starts_with('/') {
        parts.push("/".to_string());
    }
    parts.extend(
        path.split('/')
            .filter(|component| !component.is_empty())
            .map(str::to_string),
    );
    parts
}
