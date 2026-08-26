use rum_host::{HostCompletenessStatus, HostSession};

use super::{CompletenessHint, IncrementalCompleteness};
use crate::resolve_completeness;

fn append_source(completeness: &mut IncrementalCompleteness, source: &str) {
    for (index, line) in source.split('\n').enumerate() {
        if index > 0 {
            completeness.append("\n");
        }
        completeness.append(line);
    }
}

fn tracked_status(source: &str) -> (HostCompletenessStatus, CompletenessHint) {
    let mut completeness = IncrementalCompleteness::default();
    append_source(&mut completeness, source);
    let hint = completeness.hint();
    let mut session = HostSession::new();
    let status = resolve_completeness(&completeness, source.trim(), |pending| {
        session.is_complete(pending).status
    });
    (status, hint)
}

#[test]
fn incremental_completeness_matches_the_host_matrix() {
    let cases = [
        (
            "balanced single line",
            "1 + 1",
            HostCompletenessStatus::Complete,
        ),
        (
            "unbalanced open bracket",
            "f[",
            HostCompletenessStatus::Incomplete,
        ),
        (
            "nested brackets",
            "f[{<|a -> (g[[1",
            HostCompletenessStatus::Incomplete,
        ),
        (
            "bracket inside string",
            "Identity[\"[\"]",
            HostCompletenessStatus::Complete,
        ),
        (
            "bracket inside comment",
            "Identity[(* [ <| *) 1]",
            HostCompletenessStatus::Complete,
        ),
        (
            "nested comment",
            "Identity[(* outer [ (* inner ] *) *) 1]",
            HostCompletenessStatus::Complete,
        ),
        (
            "escaped quote",
            "Identity[\"a \\\" [\"]",
            HostCompletenessStatus::Complete,
        ),
        (
            "unfinished trailing operator",
            "1 +",
            HostCompletenessStatus::Incomplete,
        ),
        (
            "trailing operator across lines",
            "1 +\n2",
            HostCompletenessStatus::Complete,
        ),
        (
            "syntactically invalid form",
            "f[1,]",
            HostCompletenessStatus::Invalid,
        ),
        ("empty line", "", HostCompletenessStatus::Complete),
        ("whitespace only", " \t ", HostCompletenessStatus::Complete),
        (
            "comment only",
            "(* only a comment *)",
            HostCompletenessStatus::Invalid,
        ),
    ];

    for (label, source, expected) in cases {
        let mut baseline = HostSession::new();
        let baseline = baseline.is_complete(source).status;
        let (tracked, hint) = tracked_status(source);

        assert_eq!(baseline, expected, "baseline drift for {label}");
        assert_eq!(tracked, baseline, "incremental verdict drift for {label}");
        println!("{label}: {tracked:?} ({hint:?})");
    }
}

#[test]
fn fast_incomplete_hint_never_hides_lexer_or_parser_errors() {
    let fallback_cases = [
        "f[1 + * [",
        "f[1,] [",
        "[[1",
        "f[{1]",
        "f[[1]",
        "1 ! 2 [",
        "f[x=. [",
        "f[x& [",
        "f[\"\\[NotACharacter]\"",
        "#!/usr/bin/env rum\nf[",
    ];

    for source in fallback_cases {
        let mut completeness = IncrementalCompleteness::default();
        append_source(&mut completeness, source);
        assert_eq!(
            completeness.hint(),
            CompletenessHint::NeedsFullCheck,
            "hard or uncertain prefix must fall back: {source:?}"
        );
        let mut session = HostSession::new();
        assert_eq!(
            tracked_status(source).0,
            session.is_complete(source).status,
            "fallback verdict drift for {source:?}"
        );
    }

    let fast_cases = [
        "f[1",
        "f[[1",
        "{1",
        "<|a -> 1",
        "(1 + 2",
        "f[\"[)]\"",
        "f[(* [ (* ] *) *) 1",
        "f['''[)]",
    ];

    for source in fast_cases {
        let mut completeness = IncrementalCompleteness::default();
        append_source(&mut completeness, source);
        assert_eq!(
            completeness.hint(),
            CompletenessHint::DefinitelyIncomplete,
            "expected certified incomplete prefix: {source:?}"
        );
        let mut session = HostSession::new();
        assert_eq!(
            session.is_complete(source).status,
            HostCompletenessStatus::Incomplete,
            "fast hint must agree with the host parser: {source:?}"
        );
    }

    let too_deep = "f[".repeat(129);
    let mut completeness = IncrementalCompleteness::default();
    completeness.append(&too_deep);
    assert_eq!(completeness.hint(), CompletenessHint::NeedsFullCheck);
    let mut session = HostSession::new();
    assert_eq!(
        tracked_status(&too_deep).0,
        session.is_complete(too_deep.as_str()).status
    );
}

#[test]
fn certified_incomplete_prefix_corpus_agrees_with_the_host_parser() {
    let pieces = [
        "1", "f", "+", "-", "*", "^", "=", "->", "++", "--", "!", "'", "=.", "..", "...", ",", "]",
        "}", ")", "|>", "[", "[[", "{", "(", "<|", "_", "&", " ", "(* c *)", "\"s\"",
    ];
    let mut session = HostSession::new();

    for first in pieces {
        for second in pieces {
            for third in pieces {
                let source = format!("f[{first}{second}{third}[");
                let mut completeness = IncrementalCompleteness::default();
                completeness.append(&source);

                if completeness.hint() == CompletenessHint::DefinitelyIncomplete {
                    assert_eq!(
                        session.is_complete(source.as_str()).status,
                        HostCompletenessStatus::Incomplete,
                        "certified prefix disagreed with host parser: {source:?}"
                    );
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct CompletenessCost {
    lines: usize,
    source_bytes: usize,
    fast_examined_bytes: usize,
    full_check_bytes: usize,
    full_check_calls: usize,
}

impl CompletenessCost {
    fn total_examined_bytes(self) -> usize {
        self.fast_examined_bytes + self.full_check_bytes
    }
}

fn completeness_cost(lines: usize) -> CompletenessCost {
    let mut completeness = IncrementalCompleteness::default();
    let mut source = String::from("f[");
    completeness.append("f[");
    let mut session = HostSession::new();
    let mut full_check_bytes = 0;
    let mut full_check_calls = 0;

    for _ in 0..lines {
        source.push('\n');
        completeness.append("\n");
        source.push_str("1,");
        completeness.append("1,");
        let status = resolve_completeness(&completeness, source.trim(), |pending| {
            full_check_calls += 1;
            full_check_bytes += pending.len();
            session.is_complete(pending).status
        });
        assert_eq!(status, HostCompletenessStatus::Incomplete);
    }

    source.push('\n');
    completeness.append("\n");
    source.push_str("0]");
    completeness.append("0]");
    let status = resolve_completeness(&completeness, source.trim(), |pending| {
        full_check_calls += 1;
        full_check_bytes += pending.len();
        session.is_complete(pending).status
    });
    assert_eq!(status, HostCompletenessStatus::Complete);

    CompletenessCost {
        lines,
        source_bytes: source.len(),
        fast_examined_bytes: completeness.examined_bytes(),
        full_check_bytes,
        full_check_calls,
    }
}

#[test]
fn multiline_completeness_work_is_linear_in_appended_source() {
    let small = completeness_cost(128);
    let large = completeness_cost(256);

    assert_eq!(small.full_check_calls, 1);
    assert_eq!(large.full_check_calls, 1);
    assert!(small.fast_examined_bytes <= small.source_bytes * 2);
    assert!(large.fast_examined_bytes <= large.source_bytes * 2);
    assert!(large.total_examined_bytes() <= small.total_examined_bytes() * 2);

    println!(
        "linear-cost-counter lines={} source_bytes={} fast_examined_bytes={} full_check_bytes={} full_check_calls={} total_examined_bytes={}",
        large.lines,
        large.source_bytes,
        large.fast_examined_bytes,
        large.full_check_bytes,
        large.full_check_calls,
        large.total_examined_bytes(),
    );
}
