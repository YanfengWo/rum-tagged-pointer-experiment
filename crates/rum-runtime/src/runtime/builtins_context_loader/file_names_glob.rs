use super::*;

pub(super) const FILE_NAMES_ENTRY_LIMIT: usize = 100_000;

pub(super) struct FileNamesConfig {
    pub(super) pattern: String,
    pub(super) dirs: Vec<PathBuf>,
    pub(super) depth: FileNamesDepth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FileNamesDepth {
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

pub(super) fn parse_file_names_args(args: &[Expr]) -> Option<FileNamesConfig> {
    let (pattern, dirs, depth) = match args {
        [] => (
            "*".to_string(),
            vec![PathBuf::from(".")],
            FileNamesDepth::Finite(1),
        ),
        [pattern] => (
            file_names_pattern(pattern)?,
            vec![PathBuf::from(".")],
            FileNamesDepth::Finite(1),
        ),
        [pattern, dirs] => (
            file_names_pattern(pattern)?,
            file_names_directories(dirs)?,
            FileNamesDepth::Finite(1),
        ),
        [pattern, dirs, depth] => (
            file_names_pattern(pattern)?,
            file_names_directories(dirs)?,
            file_names_depth(depth)?,
        ),
        _ => return None,
    };

    Some(FileNamesConfig {
        pattern,
        dirs,
        depth,
    })
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

    list_items(expr)?
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

pub(super) fn collect_file_name_matches(
    session: &Session,
    file_io: &dyn FileIo,
    pattern: &str,
    dirs: Vec<PathBuf>,
    depth: FileNamesDepth,
) -> Vec<String> {
    let mut matches = Vec::new();
    let mut visited_entries = 0usize;
    let mut pending = dirs
        .into_iter()
        .rev()
        .map(|dir| (dir, depth))
        .collect::<Vec<_>>();

    while visited_entries < FILE_NAMES_ENTRY_LIMIT {
        let Some((dir, depth)) = pending.pop() else {
            break;
        };
        let Some(next_depth) = depth.next() else {
            continue;
        };
        let Ok(mut entries) = file_io.list_dir(&dir) else {
            continue;
        };

        entries.sort();
        entries.truncate(FILE_NAMES_ENTRY_LIMIT - visited_entries);
        visited_entries += entries.len();

        let mut child_dirs = Vec::new();
        for path in entries {
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            // FileNames deliberately reuses the full-string wildcard matcher
            // shared by Names and Clear instead of defining a second glob.
            if session.wildcard_name_pattern_matches(pattern, name, false) {
                matches.push(normalize_found_path(&path).display().to_string());
            }

            let probe = file_io.probe(&path);
            if probe.exists && !probe.is_file {
                child_dirs.push(path);
            }
        }

        pending.extend(child_dirs.into_iter().rev().map(|dir| (dir, next_depth)));
    }

    matches.sort();
    matches.dedup();
    matches
}
