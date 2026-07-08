# dun General Editor Plugin Requirements

Status: proposal from the downstream `dun` editor project.

This document complements `docs/dev/dun-embedding-plugin-requirements.md`.
That earlier note focuses on syntax highlighting and custom log filtering.
This one records other plugin roles that would make `rum` especially useful as
the embedded language for a general SSH-oriented terminal editor.

The common rule is unchanged: `dun` owns authority. A plugin receives bounded
snapshots from the editor and returns structured data or command intents.
`dun` validates the result and performs all file, editor, terminal, and process
actions itself.

## Common Host Requirements

These roles all benefit from the same host-facing improvements:

- typed host calls that pass arguments without source-string interpolation;
- richer `HostTyped` values, especially `Association`, `Rule`, booleans,
  null-like values, and a bounded generic call tree;
- pure-only non-interactive plugin sandbox profile;
- request-level timeout, cancellation, resource limits, output limits, and
  panic isolation;
- request isolation or automatic rollback for state created during a plugin
  call;
- stable result schemas for spans, diagnostics, text edits, completions,
  symbols, and structured matches;
- fixture-oriented host test support for plugin contracts;
- clear classification of timeout, cancellation, generated-element limits, and
  other recoverable terminations in host outcomes.

## Required Core Plugin Roles

### Diagnostics And Lint

Diagnostics are the most important general plugin role after highlighting.
They cover configuration mistakes, script errors, invalid data, deprecated
fields, suspicious values, and style warnings.

Suggested result shape:

```text
Diagnostic[
  severity,
  span,
  message,
  code,
  related,
  fixes
]
```

`severity` should support at least `error`, `warning`, `info`, and `hint`.
`related` should be a bounded list of secondary spans with messages. `fixes`
should be optional edit intents, not direct mutation.

`rum` needs:

- convenient construction of structured diagnostic data;
- span-preserving parser helpers for common data/config formats;
- fail-closed behavior for malformed input;
- bounded message and result size controls;
- a way for hosts to distinguish plugin failure from ordinary diagnostics.

### Formatters

Formatters normalize JSON, TOML, YAML, INI, XML, shell snippets, service files,
and domain-specific configs.

The formatter role should return either a full replacement or a list of text
edits. `dun` applies the edits only after validating that they are sorted,
non-overlapping, in range, and within a configured edit-size budget.

`rum` needs:

- a stable text-edit schema: `Insert`, `Delete`, `Replace`, or a compact tuple
  equivalent;
- pure format/render helpers for common codecs;
- options for preserving comments, indentation, key order, and newline style
  where the source format supports it;
- deterministic output with no ambient filesystem or clock dependency;
- good errors when formatting cannot preserve a source feature.

### Completion And Snippets

Completion should be useful even without a full LSP. For configuration files,
it can provide known field names, section names, enum values, path templates,
or safe snippets.

Suggested result fields:

- `label`;
- `kind`;
- `insert_text`;
- `replacement_span`;
- `detail`;
- `documentation`;
- `sort_key`;
- optional `additional_text_edits`.

`rum` needs:

- low-latency typed calls with cancellation;
- compact structured return values;
- optional host-provided context such as language id, path metadata, cursor
  offset, nearby text, and active schema name;
- result-count limits and deterministic sorting guidance.

### Outline, Symbols, And Jump Targets

An SSH TUI benefits heavily from fast navigation. Plugins should be able to
return sections, headings, functions, services, blocks, tasks, keys, and other
symbols.

Suggested shape:

```text
List[List[name, kind, span, selection_span, depth], ...]
```

`rum` needs:

- parser/token helpers that preserve offsets;
- simple tree/list construction patterns;
- a standard symbol-kind vocabulary that hosts can map to UI labels;
- bounded output and depth limits.

### Structural Selection, Matching, And Folding

Structural plugins support matching brackets, expanding selection, jumping to a
matching delimiter, selecting an enclosing section, and folding blocks.

Suggested outputs:

- `PairSpan` for matching delimiters;
- `NodeSpan` for syntax/config blocks;
- `FoldRange` for collapsible regions;
- parent/child relationships where cheap to compute.

`rum` needs:

- span-preserving parse helpers or tokenizer capture spans;
- robust behavior for malformed partial files;
- a convention for partial results with diagnostics when parsing is incomplete.

## High-Value Operational Roles

### Config Validators

For `dun`'s operations use case, config validation may be more important than
highlighting. Operators need to know whether a config has obvious mistakes
before saving or restarting a service.

Validators can be general, schema-driven, or site-specific. They should return
diagnostics and optional safe fixes. If an include file is needed, `dun` must
provide a bounded include snapshot explicitly; the plugin must not read it from
the filesystem.

`rum` needs:

- ergonomic schema and association validation helpers;
- source-span aware diagnostics;
- readable rule definitions for site-specific policies;
- a way to report missing host-provided context as data, not as side effects.

### Quick Fix And Safe Rewrite

Quick fixes turn diagnostics into proposed changes: add a missing field, remove
a duplicate key, replace a deprecated option, sort keys, normalize booleans, or
rewrite a legacy config shape.

The plugin should return edit intents with a reason. `dun` should preview the
diff and apply only validated edits.

`rum` needs:

- stable edit-intent data shapes;
- helpers to compose and sort edits;
- conflict detection or at least conventions that make conflicts easy for the
  host to detect;
- dry-run friendly output.

### Secrets And Risk Scanner

Risk scanning is a strong fit for an editor that opens production files and
logs. Plugins can detect private keys, access tokens, passwords, suspicious
escape sequences, dangerous shell commands, world-writable paths, or accidental
credential leaks.

The plugin should return security diagnostics and optional redaction or replace
intents. It must not exfiltrate or persist detected secrets.

`rum` needs:

- efficient pure string and pattern matching;
- capture spans for regex/string patterns;
- result redaction helpers for diagnostics;
- strict output limits so a scanner cannot dump large secrets into messages;
- examples and tests for adversarial text containing control characters.

### Semantic Search And Extraction

Semantic search is a structured form of find: search by JSON path, TOML key,
INI section, systemd unit field, function name, severity, status code, or a
custom predicate.

Suggested result shape should match the log-filter match schema where possible:

```text
List[List["match", span, fields], ...]
```

`rum` needs:

- path/query helpers for associations and nested lists;
- span-preserving structured parsers;
- predicate composition that is short enough for incident-time scripts;
- bounded results with a structured truncation marker.

### Diff And Patch Assistance

`dun` should eventually show useful diffs for files edited over SSH. Plugins
can provide semantic diffs that ignore insignificant formatting, key order, or
comments where appropriate.

The plugin should return hunks, semantic labels, and risk notes. `dun` owns the
display and any patch application.

`rum` needs:

- stable hunk and edit-intent schemas;
- pure comparison helpers for lists, associations, and parsed config trees;
- size limits and graceful fallback to textual diff hints for large inputs.

### File-Type Detection

Servers often contain files with weak or no extensions: `config`, `hosts`,
`.env`, unit overrides, shell fragments, generated files, or vendor-specific
names.

The detector role should receive path metadata plus a bounded text prefix and
return language candidates with confidence.

Suggested shape:

```text
List[List[language_id, confidence, reason], ...]
```

`rum` needs:

- fast pattern matching over file names and text samples;
- deterministic confidence scoring conventions;
- a stable language-id naming convention or guidance for host/plugin authors.

## Optional But Useful Roles

### Documentation Hover

For config keys or commands, a plugin can return short documentation for the
symbol under the cursor. In a TUI this should be concise and bounded.

`rum` needs structured lookup results with size caps and no implicit file or
network access.

### Test Or Probe Generators

Some configs can be checked by generating commands or test cases, but untrusted
plugins must not run them. A plugin may return a command intent or checklist;
`dun` or the user decides what to execute outside the sandbox.

`rum` needs an intent schema that clearly separates command text from authority.

### Redaction And View Filters

For production logs, plugins can create redacted display views without changing
the buffer. This overlaps with risk scanning but is display-oriented.

`rum` needs span-based replacement intents and careful size limits.

## Suggested Shared Schemas

First adoption can use list tuples because current host typed values expose
lists well. Richer associations should become the preferred shape once
`HostTyped` supports them structurally.

Span:

```text
List[start_Integer, end_Integer]
```

Diagnostic:

```text
List["diagnostic", severity_String, span, code_String, message_String]
```

Text edit:

```text
List["replace", span, text_String]
List["insert", span, text_String]
List["delete", span]
```

Completion:

```text
List["completion", label_String, kind_String, replacement_span, insert_text_String]
```

Symbol:

```text
List["symbol", name_String, kind_String, span, selection_span, depth_Integer]
```

Structured match:

```text
List["match", span, List[List[field_String, value_String], ...]]
```

Truncation marker:

```text
List["truncated", reason_String]
```

## Priority For rum

The most useful upstream work for these roles is:

1. typed host function calls;
2. richer `HostTyped` and stable structured result guidance;
3. plugin sandbox profile for non-interactive pure evaluation;
4. isolated request calls or low-cost snapshot/restore helpers;
5. standard span, diagnostic, edit, completion, symbol, and match schemas;
6. capture spans for string patterns and regular expressions;
7. span-preserving parsers or token helpers for common config/data formats;
8. output-size and result-count limits with structured truncation reporting;
9. host fixture test harness for plugin roles;
10. examples aimed at real operations files: systemd units, nginx snippets,
    `.env`, SSH config, hosts files, JSON/TOML/YAML/INI, shell fragments, and
    custom service configs.

These requirements should not force `rum` to become an editor framework. The
goal is a small, safe, pleasant embedded language boundary that lets host
applications like `dun` build editor-specific behavior around pure structured
computations.
