# brief-305 — split scripts into forms in linear time

## Goal

`split_script_into_forms` re-parses the whole accumulated text once per line
while a form stays unterminated, which is Σ O(i) = O(N²) in the line count of
one multi-line form. Loading a 10 000-line data script takes 7.75 s today; the
same data on one line takes 0.01 s. When you are done, the common case is
linear, and the observable form splitting is byte-for-byte what it is now.

## Measured today (release CLI, verified by the director — do not re-derive)

`data = {` + N lines of one element each + `};`

| N lines | time |
|---|---|
| 500 | 0.026 s |
| 1000 | 0.071 s |
| 2000 | 0.273 s |
| 4000 | 1.096 s |
| 8000 | 4.478 s |
| 10000 | **7.75 s** |

Textbook quadratic (~4× per doubling). The same 10 000 elements on a single
line: **0.01 s**. A control of 10 000 independent one-line statements is linear
(0.031 s), because each line parses on its own and never accumulates.

## The mechanism

`crates/rum-runtime/src/runtime/context_and_load_helpers.rs:100-149`. Each line
is appended to `current`, then `parse_expr(&current)` runs on the whole
accumulation, and the result picks one of four branches:

| branch | action |
|---|---|
| `Ok(_)` | push `current` as a form, clear |
| `Err(EmptyInput)` | clear, push nothing |
| `Err(e) if e.is_incomplete()` | keep accumulating |
| `Err(_)` (hard error) | push `current` as a form anyway, clear |

That fourth branch exists so one syntax error cannot poison every later line —
pinned by `runtime/tests/context_and_loader.rs:609`
(`"1 2\n3"` → `["1 2", "3"]`). The tail block after the loop applies the same
four-way logic, except that an incomplete tail is returned as an error.

## The fix, and the property it rests on

Keep the four-way logic exactly as it is, but **skip the `parse_expr` call when
the text cannot possibly be complete yet**. Maintain an incremental balance
state that only consumes the newly appended line: open delimiters (`[ ] { } ( )
<| |>`), string literals in every form the lexer accepts, and comments. While
the balance is open, the form is necessarily incomplete — continue accumulating
without parsing. Only when the balance closes does `parse_expr` run, and then
all four branches behave as today.

This rests on one property, which the director has spot-checked and which your
tests must pin properly:

> balance open ⟹ `parse_expr` returns an error for which `is_incomplete()` is true

Spot-checks already done (release CLI): an unclosed `f[1 2` followed by another
line reports `incomplete input: expected ]`; an unclosed `f[1 2 3` at EOF
reports the same; an unclosed string reports `unterminated string`. All three
are `is_incomplete()` classes (`crates/rum-syntax/src/error.rs:36-48` lists
exactly four: `Incomplete`, `UnclosedDelimiter`, `UnterminatedString`,
`UnterminatedComment`). Note that a hard error inside an unclosed delimiter
still surfaces as `expected ]` — that is why the property holds. If you find any
input where the balance is open but the parse error is NOT incomplete, STOP and
report it: that input would change form splitting, and the design needs revising
rather than patching.

**Where the scanner lives**: `rum-syntax`. Lexical truth belongs in one crate,
and `rum-runtime` should consume it, not re-derive it. Export it alongside the
existing `pub use lexer::lex_with_trivia` / `pub use parser::parse_expr`
(`crates/rum-syntax/src/lib.rs:17-28`).

**The scanner must agree with the real lexer.** This is the main risk: a string
or comment form the scanner does not know about would silently misclassify. Do
not hand-wave the lexical rules — derive them from the lexer
(`crates/rum-syntax/src/lexer/`), covering at minimum `"…"`, `"""…"""`,
`'''…'''`, escapes, and comments including nesting if the lexer nests them.

## Known residue — record it, do not hide it

A form whose delimiters are balanced but whose expression is unfinished — a
trailing operator, e.g.

```
1 +
2 +
3
```

stays quadratic, because balance cannot predict it and `parse_expr` must run per
line. That is acceptable: such forms are short in practice, while the quadratic
case that hurts is the long balanced literal. State this limitation explicitly
in the report and in a code comment; do not pretend the fix is total.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/` — the new scanner, its module wiring, and its export
- `crates/rum-runtime/src/runtime/context_and_load_helpers.rs`
- `crates/rum-runtime/src/runtime/tests/context_and_loader.rs`
- test modules under `crates/rum-syntax/src/` for the scanner's own tests

Out of scope: everything else. Specifically **not** `crates/rum-repl/`. Its
`is_complete` re-parses the buffer the same way
(`crates/rum-repl/src/lib.rs:113-129`), but that buffer is what a human typed at
a prompt — the line count has a practical ceiling, so the quadratic is bounded
in a way the file-loading path's is not. It is not worth the risk in this brief.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The incremental scanner in `rum-syntax`, with its own unit tests.
- `split_script_into_forms` using it, with the four-way branch logic unchanged.
- An **equivalence test** — this is the acceptance evidence, not a nicety.
  Keep a reference copy of today's parse-every-line implementation inside the
  test module, and assert that both produce identical `Vec<String>` output over
  a corpus that includes: multi-line lists/associations/function definitions;
  the trailing-operator residue case; unterminated strings and comments; a hard
  syntax error at top level (the `"1 2\n3"` shape); a hard error inside an
  unclosed delimiter; blank lines in and around forms; a shebang line; nested
  brackets; strings containing `[`, `]`, `;` and newlines; comments containing
  delimiters; and an incomplete tail at EOF.
- The scaling numbers, before and after, for the N in the table above.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli`, regenerate the scaling
table. Feed stdin from `/dev/null` — the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (500, 1000, 2000, 4000, 8000, 10000):
    body = ",\n".join("  %d" % i for i in range(1, n + 1))
    open("/tmp/b305-%d.rum" % n, "w").write(
        "data = {\n%s\n};\nPrint[\"len=\", Length[data]];\n" % body)
PY
for n in 500 1000 2000 4000 8000 10000; do
  echo "--- $n ---"
  /usr/bin/time -p ./target/release/rum-cli /tmp/b305-$n.rum < /dev/null
done
```

Every run must print the correct `len=`, and the growth must be linear rather
than ~4× per doubling. Paste the raw numbers.

Also confirm the one-line control (`data = {1, …, 10000};` on a single line) and
the 10 000-independent-statement control did not regress.

No microbench A/B is required: this is the script-loading path, not the
evaluation hot path. Say so if you believe otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If the balance-implies-incomplete property fails on any
  input, or the equivalence test cannot be made to pass, STOP and report with
  the failing input rather than special-casing it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, with line ranges, and the scanner's lexical rules
   and where you derived each from.
2. Verification — each command with exact verbatim output: test counts, the
   equivalence-test corpus size, and the before/after scaling table.
3. The verdict — the achieved complexity, and the residue case stated plainly.
4. Stop-loss / open questions — any input where the property was doubtful.
