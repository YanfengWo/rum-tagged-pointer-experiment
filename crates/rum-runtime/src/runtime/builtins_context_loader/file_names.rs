use super::*;

impl Session {
    pub(in crate::runtime) fn eval_file_name_join(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [parts_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FileNameJoin", args, 1));
        };

        let Some(parts) = expr_as_string_list(parts_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "FileNameJoin",
                args,
                "expected a list of string path components",
            ));
        };

        let path = parts.into_iter().fold(PathBuf::new(), |mut path, part| {
            path.push(part);
            path
        });
        Ok(Expr::string(path.display().to_string()))
    }

    pub(in crate::runtime) fn eval_directory_name(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (path, levels) = match args {
            [path] => (path, 1),
            [path, levels] => {
                let Some(levels) = expr_as_non_negative_usize(levels) else {
                    return Ok(Expr::call(system_expr("DirectoryName"), args.to_vec()));
                };
                (path, levels)
            }
            _ => return Ok(self.invalid_call_with_arg_range("DirectoryName", args, 1, 2)),
        };

        let Some(path) = expr_as_string(path) else {
            return Ok(self.invalid_call_with_arg_form(
                "DirectoryName",
                args,
                "expected a string file name",
            ));
        };

        Ok(Expr::string(render_directory_name(parent_path_n(
            Path::new(path),
            levels,
        ))))
    }

    pub(in crate::runtime) fn eval_parent_directory(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [path] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ParentDirectory", args, 1));
        };

        let Some(path) = expr_as_string(path) else {
            return Ok(self.invalid_call_with_arg_form(
                "ParentDirectory",
                args,
                "expected a string directory name",
            ));
        };

        let path = Path::new(path);
        if !path.is_dir() {
            return Ok(self.invalid_call_with_message(
                "ParentDirectory",
                args,
                "dirnex",
                format!("Directory {} does not exist.", path.display()),
            ));
        }

        Ok(Expr::string(render_directory_name(parent_path_n(path, 1))))
    }

    pub(in crate::runtime) fn eval_file_name_take(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (path, spec) = match args {
            [path] => (path, None),
            [path, spec] => (path, Some(spec)),
            _ => return Ok(self.invalid_call_with_arg_range("FileNameTake", args, 1, 2)),
        };

        let Some(path) = expr_as_string(path) else {
            return Ok(self.invalid_call_with_arg_form(
                "FileNameTake",
                args,
                "expected a string file name",
            ));
        };

        let taken = match spec {
            None => file_name_leaf(path),
            Some(spec) => {
                let Some(taken) = file_name_take_by_spec(path, spec) else {
                    return Ok(Expr::call(system_expr("FileNameTake"), args.to_vec()));
                };
                taken
            }
        };
        Ok(Expr::string(taken))
    }

    pub(in crate::runtime) fn eval_file_names(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (pattern, dirs, depth) = match args {
            [] => (
                "*".to_string(),
                vec![PathBuf::from(".")],
                FileNamesDepth::Finite(1),
            ),
            [pattern] => {
                let Some(pattern) = file_names_pattern(pattern) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                (pattern, vec![PathBuf::from(".")], FileNamesDepth::Finite(1))
            }
            [pattern, dirs] => {
                let Some(pattern) = file_names_pattern(pattern) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                let Some(dirs) = file_names_directories(dirs) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                (pattern, dirs, FileNamesDepth::Finite(1))
            }
            [pattern, dirs, depth] => {
                let Some(pattern) = file_names_pattern(pattern) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                let Some(dirs) = file_names_directories(dirs) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                let Some(depth) = file_names_depth(depth) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                (pattern, dirs, depth)
            }
            _ => return Ok(self.invalid_call_with_arg_range("FileNames", args, 0, 3)),
        };

        let mut matches = Vec::new();
        for dir in dirs {
            collect_file_name_matches(&pattern, &dir, depth, &mut matches);
        }
        matches.sort();
        matches.dedup();

        Ok(Expr::call(
            system_expr("List"),
            matches.into_iter().map(Expr::string).collect(),
        ))
    }

    pub(in crate::runtime) fn eval_file_exists_q(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [path_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FileExistsQ", args, 1));
        };
        let Some(path) = load_spec_from_expr(path_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "FileExistsQ",
                args,
                "expected a string or File[\"...\"] path",
            ));
        };

        Ok(bool_expr(Path::new(&path).exists()))
    }

    pub(in crate::runtime) fn eval_file_byte_count(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [path_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FileByteCount", args, 1));
        };
        let Some(path) = load_spec_from_expr(path_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "FileByteCount",
                args,
                "expected a string or File[\"...\"] path",
            ));
        };

        Ok(fs::metadata(path)
            .map(|metadata| Expr::integer(BigInt::from(metadata.len())))
            .unwrap_or_else(|_| system_expr("$Failed")))
    }

    pub(in crate::runtime) fn eval_file_date(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let path_expr = match args {
            [path] => path,
            [path, property] if expr_as_string(property) == Some("Modification") => path,
            [_, _] => return Ok(Expr::call(system_expr("FileDate"), args.to_vec())),
            _ => return Ok(self.invalid_call_with_arg_range("FileDate", args, 1, 2)),
        };
        let Some(path) = load_spec_from_expr(path_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "FileDate",
                args,
                "expected a string or File[\"...\"] path",
            ));
        };

        Ok(fs::metadata(path)
            .and_then(|metadata| metadata.modified())
            .map(|modified| Expr::call(system_expr("DateObject"), vec![unix_time_expr(modified)]))
            .unwrap_or_else(|_| system_expr("$Failed")))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileNamesDepth {
    Finite(usize),
    Infinity,
}

impl FileNamesDepth {
    fn next(self) -> Option<Self> {
        match self {
            Self::Finite(0) => None,
            Self::Finite(depth) => Some(Self::Finite(depth - 1)),
            Self::Infinity => Some(Self::Infinity),
        }
    }
}

fn parent_path_n(path: &Path, levels: usize) -> PathBuf {
    let mut current = path.to_path_buf();
    for _ in 0..levels {
        current = current
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(PathBuf::new);
    }
    current
}

fn render_directory_name(path: PathBuf) -> String {
    let mut rendered = path.display().to_string();
    if rendered.is_empty() {
        return rendered;
    }
    let separator = std::path::MAIN_SEPARATOR;
    if !rendered.ends_with(separator) {
        rendered.push(separator);
    }
    rendered
}

fn file_name_leaf(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_string()
}

fn file_name_take_by_spec(path: &str, spec: &Expr) -> Option<String> {
    if let Some(count) = expr_as_signed_isize(spec) {
        return Some(file_name_take_by_count(path, count));
    }

    let items = list_items(spec)?;
    let [start, end] = items else {
        return None;
    };
    file_name_take_by_range(
        path,
        expr_as_signed_isize(start)?,
        expr_as_signed_isize(end)?,
    )
}

fn file_name_take_by_count(path: &str, count: isize) -> String {
    if count == 0 {
        return String::new();
    }

    let components = file_name_components(path);
    let taken = if count > 0 {
        components
            .into_iter()
            .take(count.unsigned_abs())
            .collect::<Vec<_>>()
    } else {
        let count = count.unsigned_abs();
        components
            .into_iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
    };
    join_file_name_components(&taken)
}

fn file_name_take_by_range(path: &str, start: isize, end: isize) -> Option<String> {
    let components = file_name_components(path)
        .into_iter()
        .filter(|component| component != std::path::MAIN_SEPARATOR_STR)
        .collect::<Vec<_>>();
    if components.is_empty() {
        return Some(String::new());
    }

    let start = normalize_file_component_index(start, components.len())?;
    let end = normalize_file_component_index(end, components.len())?;
    if start > end {
        return Some(String::new());
    }

    Some(join_file_name_components(&components[start..=end]))
}

fn normalize_file_component_index(index: isize, len: usize) -> Option<usize> {
    if index > 0 {
        usize::try_from(index - 1).ok().filter(|index| *index < len)
    } else if index < 0 {
        let len = isize::try_from(len).ok()?;
        usize::try_from(len + index).ok()
    } else {
        None
    }
}

fn file_name_components(path: &str) -> Vec<String> {
    Path::new(path)
        .components()
        .filter_map(|component| match component {
            std::path::Component::RootDir => Some(std::path::MAIN_SEPARATOR.to_string()),
            std::path::Component::Normal(value) => value.to_str().map(str::to_string),
            std::path::Component::CurDir => Some(".".to_string()),
            std::path::Component::ParentDir => Some("..".to_string()),
            std::path::Component::Prefix(prefix) => {
                Some(prefix.as_os_str().to_string_lossy().into_owned())
            }
        })
        .collect()
}

fn join_file_name_components(components: &[String]) -> String {
    let mut path = PathBuf::new();
    for component in components {
        if component == std::path::MAIN_SEPARATOR_STR {
            path.push(std::path::MAIN_SEPARATOR_STR);
        } else {
            path.push(component);
        }
    }
    path.display().to_string()
}

fn file_names_pattern(expr: &Expr) -> Option<String> {
    if is_system_symbol_name(expr, "All") {
        return Some("*".to_string());
    }
    expr_as_string(expr).map(str::to_string)
}

fn file_names_directories(expr: &Expr) -> Option<Vec<PathBuf>> {
    if let Some(dir) = expr_as_string(expr) {
        return Some(vec![PathBuf::from(dir)]);
    }
    let items = list_items(expr)?;
    items
        .iter()
        .map(|item| expr_as_string(item).map(PathBuf::from))
        .collect()
}

fn file_names_depth(expr: &Expr) -> Option<FileNamesDepth> {
    if is_system_symbol_name(expr, "Infinity") {
        return Some(FileNamesDepth::Infinity);
    }
    expr_as_non_negative_usize(expr).map(FileNamesDepth::Finite)
}

fn collect_file_name_matches(
    pattern: &str,
    dir: &Path,
    depth: FileNamesDepth,
    matches: &mut Vec<String>,
) {
    let Some(next_depth) = depth.next() else {
        return;
    };
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if wildcard_matches(pattern, name) {
            matches.push(normalize_found_path(&path).display().to_string());
        }
        if entry.file_type().is_ok_and(|file_type| file_type.is_dir()) {
            collect_file_name_matches(pattern, &path, next_depth, matches);
        }
    }
}

fn bool_expr(value: bool) -> Expr {
    system_expr(if value { "True" } else { "False" })
}

fn wildcard_matches(pattern: &str, text: &str) -> bool {
    wildcard_matches_chars(
        &pattern.chars().collect::<Vec<_>>(),
        &text.chars().collect::<Vec<_>>(),
    )
}

fn wildcard_matches_chars(pattern: &[char], text: &[char]) -> bool {
    match pattern {
        [] => text.is_empty(),
        ['*', rest @ ..] => {
            wildcard_matches_chars(rest, text)
                || (!text.is_empty() && wildcard_matches_chars(pattern, &text[1..]))
        }
        ['?', rest @ ..] => !text.is_empty() && wildcard_matches_chars(rest, &text[1..]),
        [expected, rest @ ..] => {
            text.first() == Some(expected) && wildcard_matches_chars(rest, &text[1..])
        }
    }
}
