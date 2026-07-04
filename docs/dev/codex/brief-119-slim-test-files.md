# Brief 119 — file-slimming batch 6 (FINAL): the two >35k test files

## Goal

Last debt-file batch. Split
`crates/rum-stdlib/src/tests/strings/incidental_runtime.rs` (67,110 chars —
the strings-move migration artifact holding measured-failure test fns from
many source files) and `crates/rum-syntax/src/parser/tests.rs` (37,920
chars) into module directories per brief-114's SPLITTING PRINCIPLES, adapted
for tests: split by BEHAVIOR FAMILY (what is being tested), never by size
chunks; shared setup/factories go in a small local support module; every
test function moves VERBATIM (same names — the test count is the identity
check). Zero test deleted/weakened/renamed; workspace green with the same
totals (1301/0 my count; doc-test-inclusive counts may read higher). Every
new file ≤20k.

## Context pointers

- Read `AGENTS.md`, `docs/code-organization-guidelines.md` (incl. the Test
  File Guidance section: test files may run somewhat larger when related
  scenarios belong together — a cohesive 22k scenario family is acceptable
  WITH justification), brief-114 principles.
- `incidental_runtime.rs`: accumulated during the strings move (brief-105)
  from measured-failure migrations — its content GROUPS NATURALLY BY SOURCE
  origin (which runtime area the tests came from: messages/formatting,
  loader/context, definitions, session-requests, …). Read the file's
  section comments (`// Moved from …` markers) — they are the split map's
  raw material. Name the modules by the BEHAVIOR under test, not by the
  historical source file.
- `parser/tests.rs`: split by grammar area (expressions/operators,
  precedence, patterns, strings/escapes, diagnostics/errors, spans, …) —
  verify against the actual test names.
- Identity check REQUIRED: `cargo test -p rum-stdlib -- --list` and
  `cargo test -p rum-syntax -- --list` test-name sets must be IDENTICAL
  before vs after (capture both to /tmp files and diff; paste the diff —
  which must be empty except module-path prefixes; account for prefix
  changes explicitly).

## Scope

- MAY modify: the two files → module directories (for `parser/tests.rs`:
  `parser/tests/` with `mod.rs`); the `mod` declaration lines that include
  them; `use` lines inside.
- MUST NOT touch: any non-test code, any test body/name, other test files,
  `Cargo.toml`/`Cargo.lock`, any `mma-hol` clone, `docs/**` except your
  report, `.git`.

## Verification (MANDATORY)

```
cargo test -p rum-stdlib -- --list > /tmp/cdx_119_stdlib_before.txt   # BEFORE any edit
cargo test -p rum-syntax -- --list > /tmp/cdx_119_syntax_before.txt   # BEFORE any edit
# ... split ...
cargo fmt --check
cargo test --workspace
cargo test -p rum-stdlib -- --list > /tmp/cdx_119_stdlib_after.txt
cargo test -p rum-syntax -- --list > /tmp/cdx_119_syntax_after.txt
# paste the name-set comparison (normalize module prefixes, then diff)
ls crates/rum-stdlib/src/tests/strings/incidental_runtime.rs crates/rum-syntax/src/parser/tests.rs 2>&1   # both gone
find crates/rum-stdlib/src/tests/strings/incidental_runtime crates/rum-syntax/src/parser/tests -name "*.rs" 2>/dev/null | while read f; do wc -c "$f"; done | sort -rn
git status --short -- crates/
```

Every new file ≤20k (test-cohesion exceptions allowed WITH justification).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, `grep`,
  `python3`. Never edit any `mma-hol` file. **No new `unsafe`.**
- Stop-loss: any test-name delta beyond module prefixes, or the same compile
  error twice → STOP and report.

## Report format

1. BOTH SPLIT MAPS (module → behavior family → chars → test-fn count).
2. What changed — inventory; the test-name identity evidence.
3. Verification — verbatim outputs incl. size tables.
4. Verdict — the line's END STATE: paste the final whole-workspace >35k
   audit (`find crates -name "*.rs" -path "*/src/*" | while read f; do c=$(wc -c < "$f"); if [ $c -gt 35000 ]; then echo "$c $f"; fi; done`)
   — expected EMPTY; plus the 20-35k band count for the record.
5. Stop-loss / open questions.
