use crate::FormatterOptions;

pub fn parse_format_cli_args(
    args: &[String],
    command_name: &str,
) -> Result<(String, FormatterOptions), String> {
    let mut options = FormatterOptions::default();
    let mut start = 0usize;
    let mut end = args.len();

    while let Some(consumed) = consume_leading_cli_option(&args[start..], &mut options)? {
        start += consumed;
    }

    while let Some(consumed) = consume_trailing_cli_option(&args[start..end], &mut options)? {
        end -= consumed;
    }

    let expr_args = &args[start..end];

    match expr_args {
        [] => Err(format!("{command_name} expects an expression")),
        [expr] => Ok((expr.clone(), options)),
        [_, extra, ..] => Err(format!("unexpected extra argument `{extra}`")),
    }
}

pub fn parse_format_command<'a>(
    input: &'a str,
    command_name: &str,
) -> Result<(&'a str, FormatterOptions), String> {
    let mut options = FormatterOptions::default();
    let mut remainder = input.trim();

    if remainder.is_empty() {
        return Err(format!("{command_name} expects an expression"));
    }

    while let Some(next) = consume_leading_text_option(remainder, &mut options)? {
        remainder = next;
    }

    while let Some(next) = consume_trailing_text_option(remainder, &mut options)? {
        remainder = next;
    }

    let expr = remainder.trim();

    if expr.is_empty() {
        Err(format!("{command_name} expects an expression"))
    } else {
        Ok((expr, options))
    }
}

fn consume_leading_cli_option(
    args: &[String],
    options: &mut FormatterOptions,
) -> Result<Option<usize>, String> {
    let Some(flag) = args.first().map(String::as_str) else {
        return Ok(None);
    };

    parse_option(flag, args.get(1).map(String::as_str), options)
}

fn consume_trailing_cli_option(
    args: &[String],
    options: &mut FormatterOptions,
) -> Result<Option<usize>, String> {
    let Some(last) = args.last().map(String::as_str) else {
        return Ok(None);
    };

    if let Some(raw_width) = last.strip_prefix("--line-width=") {
        options.line_width = parse_line_width(raw_width, "--line-width")?;
        return Ok(Some(1));
    }

    if last.starts_with("--") {
        return Err(unknown_flag(last));
    }

    if let Some(flag) = args.get(args.len().saturating_sub(2)).map(String::as_str) {
        if flag == "--line-width" {
            options.line_width = parse_line_width(last, "--line-width")?;
            return Ok(Some(2));
        }
    }

    Ok(None)
}

fn consume_leading_text_option<'a>(
    input: &'a str,
    options: &mut FormatterOptions,
) -> Result<Option<&'a str>, String> {
    let Some((flag, rest)) = split_first_token(input) else {
        return Ok(None);
    };

    match parse_option_text(flag, rest, options)? {
        Some(next) => Ok(Some(next)),
        None => Ok(None),
    }
}

fn consume_trailing_text_option<'a>(
    input: &'a str,
    options: &mut FormatterOptions,
) -> Result<Option<&'a str>, String> {
    let Some((body, last)) = split_last_token(input) else {
        return Ok(None);
    };

    if let Some(raw_width) = last.strip_prefix("--line-width=") {
        options.line_width = parse_line_width(raw_width, "--line-width")?;
        return Ok(Some(body));
    }

    if last.starts_with("--") {
        return Err(unknown_flag(last));
    }

    let Some((before_flag, flag)) = split_last_token(body) else {
        return Ok(None);
    };

    if flag == "--line-width" {
        options.line_width = parse_line_width(last, "--line-width")?;
        return Ok(Some(before_flag));
    }

    Ok(None)
}

fn parse_option(
    flag: &str,
    next: Option<&str>,
    options: &mut FormatterOptions,
) -> Result<Option<usize>, String> {
    if let Some(raw_width) = flag.strip_prefix("--line-width=") {
        options.line_width = parse_line_width(raw_width, "--line-width")?;
        return Ok(Some(1));
    }

    match flag {
        "--line-width" => {
            let raw_width =
                next.ok_or_else(|| "--line-width expects a positive integer".to_owned())?;
            options.line_width = parse_line_width(raw_width, "--line-width")?;
            Ok(Some(2))
        }
        other if other.starts_with("--") => Err(unknown_flag(other)),
        _ => Ok(None),
    }
}

fn parse_option_text<'a>(
    flag: &str,
    rest: &'a str,
    options: &mut FormatterOptions,
) -> Result<Option<&'a str>, String> {
    if let Some(raw_width) = flag.strip_prefix("--line-width=") {
        options.line_width = parse_line_width(raw_width, "--line-width")?;
        return Ok(Some(rest));
    }

    match flag {
        "--line-width" => {
            let Some((raw_width, next)) = split_first_token(rest) else {
                return Err("--line-width expects a positive integer".to_owned());
            };
            options.line_width = parse_line_width(raw_width, "--line-width")?;
            Ok(Some(next))
        }
        other if other.starts_with("--") => Err(unknown_flag(other)),
        _ => Ok(None),
    }
}

fn parse_line_width(raw: &str, flag: &str) -> Result<usize, String> {
    match raw.parse::<usize>() {
        Ok(0) | Err(_) => Err(format!("{flag} expects a positive integer")),
        Ok(width) => Ok(width),
    }
}

fn split_first_token(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim_start();

    if trimmed.is_empty() {
        return None;
    }

    match trimmed.find(char::is_whitespace) {
        Some(index) => Some((&trimmed[..index], trimmed[index..].trim_start())),
        None => Some((trimmed, "")),
    }
}

fn split_last_token(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim_end();

    if trimmed.is_empty() {
        return None;
    }

    match trimmed.rfind(char::is_whitespace) {
        Some(index) => Some((trimmed[..index].trim_end(), &trimmed[index + 1..])),
        None => Some(("", trimmed)),
    }
}

fn unknown_flag(flag: &str) -> String {
    format!("unknown flag `{flag}`")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_format_cli_args_accepts_prefix_and_suffix_width_flags() {
        let (expr, options) = parse_format_cli_args(
            &[
                "--line-width=50".to_owned(),
                "f[x]".to_owned(),
                "--line-width".to_owned(),
                "60".to_owned(),
            ],
            "--format",
        )
        .unwrap();

        assert_eq!(expr, "f[x]");
        assert_eq!(options.line_width, 60);
    }

    #[test]
    fn parse_format_command_accepts_prefix_and_suffix_width_flags() {
        let (expr, options) =
            parse_format_command("--line-width=50 f[x] --line-width 60", ":format").unwrap();

        assert_eq!(expr, "f[x]");
        assert_eq!(options.line_width, 60);
    }

    #[test]
    fn parse_format_command_rejects_invalid_width_with_shared_message() {
        assert_eq!(
            parse_format_command("--line-width nope f[x]", ":format").unwrap_err(),
            "--line-width expects a positive integer"
        );
        assert_eq!(
            parse_format_cli_args(
                &[
                    "f[x]".to_owned(),
                    "--line-width".to_owned(),
                    "nope".to_owned()
                ],
                "--format",
            )
            .unwrap_err(),
            "--line-width expects a positive integer"
        );
    }

    #[test]
    fn parse_format_command_rejects_unknown_leading_flags() {
        assert_eq!(
            parse_format_command("--typo f[x]", ":format").unwrap_err(),
            "unknown flag `--typo`"
        );
    }
}
