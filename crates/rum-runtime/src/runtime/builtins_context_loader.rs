use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct NeedsLoadConfig {
    context: String,
    load_spec: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BeginPackageConfig {
    target_context: String,
    dependencies: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LoaderSpecNormalizationError {
    InvalidLoadTarget,
    InvalidNeedsContext,
    InvalidNeedsLoadTarget,
    InvalidBeginPackageContext,
    InvalidBeginPackageDependencies,
    WrongArgCount {
        expected: &'static str,
        actual: usize,
    },
}

impl LoaderSpecNormalizationError {
    fn message(self) -> Option<&'static str> {
        match self {
            Self::InvalidLoadTarget => Some("expected a string or File[\"...\"] load target"),
            Self::InvalidNeedsContext => Some(
                "expected a string context name, optionally followed by a string or File[\"...\"] load target",
            ),
            Self::InvalidNeedsLoadTarget => {
                Some("expected a string or File[\"...\"] as the explicit load target")
            }
            Self::InvalidBeginPackageContext => Some(
                "expected a string context name, optionally followed by a list of string dependencies",
            ),
            Self::InvalidBeginPackageDependencies => {
                Some("expected the dependency list to be a list of strings")
            }
            Self::WrongArgCount { .. } => None,
        }
    }
}

impl Session {
    pub(super) fn eval_context(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(Expr::string(self.current_context.clone())),
            [Expr::Atom(Atom::Symbol(symbol))] => Ok(Expr::string(symbol.context().to_string())),
            [Expr::Atom(Atom::String(raw_name))] => self
                .resolve_symbol_name(raw_name, false)
                .map(|symbol| Expr::string(symbol.context().to_string()))
                .map_or_else(|| Ok(Expr::call(system_expr("Context"), args.to_vec())), Ok),
            [_] => Ok(self.invalid_call_with_arg_form(
                "Context",
                args,
                "expected a symbol or string argument",
            )),
            _ => Ok(self.invalid_call_with_message(
                "Context",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(super) fn eval_find_file(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let spec = match self.compile_load_target_from_resolved_args(args) {
            Ok(spec) => spec,
            Err(LoaderSpecNormalizationError::WrongArgCount { .. }) => {
                return Ok(self.invalid_call_with_exact_arg_count("FindFile", args, 1));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "FindFile",
                    args,
                    error
                        .message()
                        .expect("loader target normalization errors should have messages"),
                ));
            }
        };

        Ok(match self.find_load_target(&spec) {
            Some((path, _)) => Expr::string(path.display().to_string()),
            None => {
                self.emit_loader_message(
                    "FindFile",
                    &EvalError::LoadPathNotFound {
                        target: spec.clone(),
                        searched: self.load_target_candidates(&spec).1,
                    },
                );
                system_expr("$Failed")
            }
        })
    }

    pub(super) fn eval_file_name_join(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_directory_name(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_parent_directory(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_file_name_take(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
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

    pub(super) fn eval_file_names(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (pattern, dirs) = match args {
            [] => ("*".to_string(), vec![PathBuf::from(".")]),
            [pattern] => {
                let Some(pattern) = file_names_pattern(pattern) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                (pattern, vec![PathBuf::from(".")])
            }
            [pattern, dirs] => {
                let Some(pattern) = file_names_pattern(pattern) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                let Some(dirs) = file_names_directories(dirs) else {
                    return Ok(Expr::call(system_expr("FileNames"), args.to_vec()));
                };
                (pattern, dirs)
            }
            _ => return Ok(self.invalid_call_with_arg_range("FileNames", args, 0, 2)),
        };

        let mut matches = Vec::new();
        for dir in dirs {
            let Ok(entries) = fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                    continue;
                };
                if wildcard_matches(&pattern, name) {
                    matches.push(normalize_found_path(&path).display().to_string());
                }
            }
        }
        matches.sort();
        matches.dedup();

        Ok(Expr::call(
            system_expr("List"),
            matches.into_iter().map(Expr::string).collect(),
        ))
    }

    pub(super) fn eval_get(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let spec = match self.compile_load_target_from_resolved_args(args) {
            Ok(spec) => spec,
            Err(LoaderSpecNormalizationError::WrongArgCount { .. }) => {
                return Ok(self.invalid_call_with_exact_arg_count("Get", args, 1));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Get",
                    args,
                    error
                        .message()
                        .expect("loader target normalization errors should have messages"),
                ));
            }
        };

        let path = match self.resolve_load_target(&spec) {
            Ok(path) => path,
            Err(err) => {
                self.emit_loader_message("Get", &err);
                return Ok(system_expr("$Failed"));
            }
        };

        match self.eval_file(path) {
            Ok(outcome) => {
                self.messages.extend(outcome.messages);
                Ok(outcome.value)
            }
            Err(err) => {
                if matches!(err, EvalError::Exit { .. }) {
                    return Err(err);
                }
                self.emit_loader_message("Get", &err);
                Ok(system_expr("$Failed"))
            }
        }
    }

    pub(super) fn eval_needs(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let config = match self.compile_needs_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(LoaderSpecNormalizationError::WrongArgCount { expected, actual }) => {
                return Ok(self.invalid_call_with_message(
                    "Needs",
                    args,
                    "argx",
                    format!("expected {expected} argument(s), got {actual}"),
                ));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "Needs",
                    args,
                    error
                        .message()
                        .expect("loader spec normalization errors should have messages"),
                ));
            }
        };

        if self
            .loaded_packages
            .iter()
            .any(|loaded| loaded == &config.context)
        {
            return Ok(system_expr("Null"));
        }

        let result = self.eval_get(&[Expr::string(config.load_spec.clone())])?;

        if is_system_symbol_name(&result, "$Failed") {
            self.push_message(
                "Needs",
                "load",
                format!("context `{}` could not be loaded", config.context),
            );
            return Ok(system_expr("$Failed"));
        }

        self.mark_package_loaded(&config.context);
        self.context_path = prepend_context(&config.context, &self.context_path);
        Ok(system_expr("Null"))
    }

    pub(super) fn eval_begin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [context] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Begin", args, 1));
        };

        let Expr::Atom(Atom::String(context)) = context else {
            return Ok(self.invalid_call_with_arg_form(
                "Begin",
                args,
                "expected a string context name",
            ));
        };

        let target = expand_context_spec(context, &self.current_context);
        Ok(self.begin_context(target))
    }

    pub(super) fn eval_end(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("End", args, 0));
        }

        Ok(self.end_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "End",
                &[],
                "context",
                "no matching Begin[] frame is active",
            )
        }))
    }

    pub(super) fn eval_begin_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let config = match self.compile_begin_package_call_from_resolved_args(args) {
            Ok(config) => config,
            Err(LoaderSpecNormalizationError::WrongArgCount { expected, actual }) => {
                return Ok(self.invalid_call_with_message(
                    "BeginPackage",
                    args,
                    "argx",
                    format!("expected {expected} argument(s), got {actual}"),
                ));
            }
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "BeginPackage",
                    args,
                    error
                        .message()
                        .expect("loader spec normalization errors should have messages"),
                ));
            }
        };

        let result = self.begin_package_context(config.target_context.clone());

        for context in &config.dependencies {
            self.eval_needs(&[Expr::string(context.clone())])?;
        }

        self.context_path = prepend_context(&config.target_context, &self.context_path);

        Ok(result)
    }

    pub(super) fn eval_end_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("EndPackage", args, 0));
        }

        Ok(self.end_package_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "EndPackage",
                &[],
                "context",
                "no matching BeginPackage[] frame is active",
            )
        }))
    }

    fn compile_load_target_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<String, LoaderSpecNormalizationError> {
        match args {
            [spec] => {
                load_spec_from_expr(spec).ok_or(LoaderSpecNormalizationError::InvalidLoadTarget)
            }
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1",
                actual: args.len(),
            }),
        }
    }

    fn compile_needs_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<NeedsLoadConfig, LoaderSpecNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context))] => {
                let context = normalize_context_name(context);
                Ok(NeedsLoadConfig {
                    load_spec: context.clone(),
                    context,
                })
            }
            [Expr::Atom(Atom::String(context)), spec] => Ok(NeedsLoadConfig {
                context: normalize_context_name(context),
                load_spec: load_spec_from_expr(spec)
                    .ok_or(LoaderSpecNormalizationError::InvalidNeedsLoadTarget)?,
            }),
            [_, ..] if args.len() <= 2 => Err(LoaderSpecNormalizationError::InvalidNeedsContext),
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1 or 2",
                actual: args.len(),
            }),
        }
    }

    fn compile_begin_package_call_from_resolved_args(
        &self,
        args: &[Expr],
    ) -> Result<BeginPackageConfig, LoaderSpecNormalizationError> {
        match args {
            [Expr::Atom(Atom::String(context))] => Ok(BeginPackageConfig {
                target_context: expand_context_spec(context, &self.current_context),
                dependencies: Vec::new(),
            }),
            [Expr::Atom(Atom::String(context)), dependencies] => Ok(BeginPackageConfig {
                target_context: expand_context_spec(context, &self.current_context),
                dependencies: expr_as_string_list(dependencies)
                    .ok_or(LoaderSpecNormalizationError::InvalidBeginPackageDependencies)?
                    .into_iter()
                    .map(normalize_context_name)
                    .collect(),
            }),
            [_, ..] if args.len() <= 2 => {
                Err(LoaderSpecNormalizationError::InvalidBeginPackageContext)
            }
            _ => Err(LoaderSpecNormalizationError::WrongArgCount {
                expected: "1 or 2",
                actual: args.len(),
            }),
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
