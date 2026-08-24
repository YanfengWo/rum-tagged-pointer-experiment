# brief-307 — linear script splitting via boundary-aware prefix parse

## Goal

`split_script_into_forms` splits a script in **one lex plus one parse pass** over
the source — every token parsed exactly once — while its observable output stays
byte-identical to today's line-driven algorithm: the same form strings, the same
tail error, the same downstream behavior. The measured quadratic disappears at
this step.

This is step 1 of 2. Step 2 (forms carrying their already-built `SyntaxExpr` so
evaluation stops re-parsing) is a separate brief; do not start it here.

## Established, do not re-derive

- The quadratic, measured by the director: `data = {` + N element lines + `}`
  takes 0.026 / 0.071 / 0.273 / 1.096 / 4.478 s at N = 500 / 1000 / 2000 / 4000
  / 8000, and **7.75 s at 10 000**; the same data on one line is **0.01 s**.
- The four-way branch logic lives at
  `crates/rum-runtime/src/runtime/context_and_load_helpers.rs:100-149`.
- `parse_root` (`crates/rum-syntax/src/parser/front.rs:87-101`) requires the
  whole token stream to be consumed, otherwise `TrailingInput`.
- The parser is recursive descent **with no backtracking** — the director
  grepped for position save/restore and found none. This is what makes prefix
  determinism true rather than assumed.
- brief-305's lexical-balance approach is dead; do not revive it.

## The rule

Drive the parser over the whole token stream, form by form, stopping on exactly
this condition:

> At delimiter depth 0, when the parser is making an **optional-continuation**
> decision — it has not yet committed to consuming an operator, postfix, or
> juxtaposition — and the next token **starts a new line**, the current form ends
> there.

A token "starts a new line" when the source text between the previous token's
end and this token's start contains `\n`.

The equivalence to today's algorithm rests on two structural facts, which the
report must restate in its own words:

- **(A) Prefix determinism.** With no backtracking, the parser's decision at
  token T depends only on the tokens before T (plus its bounded lookahead).
  So "the whole-file parse first fails hard at T" ⟺ "any line-prefix containing
  T fails hard at T" — which is precisely when today's line loop notices.
- **(B) Optional/required dichotomy.** At depth 0 the parser's decision about
  the next token is either *optional* (stopping yields a complete expression) or
  *required* (stopping yields incomplete). Today's "the form ends at the first
  line whose parse returns `Ok`" is exactly "the form ends before the first
  new-line token met at an optional decision at depth 0".

## Step 1a — enumerate the stop sites, and prove the enumeration

The rule is only as good as the list of optional-continuation sites. **Do not
copy a list from anywhere.** Derive it yourself: walk the parser and enumerate
every place that inspects the current token and may choose *not* to consume it,
at depth 0. Give `path:line` for each, and state how you convinced yourself the
enumeration is complete — e.g. by enumerating all reads of `self.current()` /
`current_binary_op()` and classifying each as optional or required.

Sites known to exist (a starting point to verify and extend, not to trust):
the `parse_function_expr` loop, the `parse_binary_expr_internal` climbing loop
including its trailing-`;` `Null` branch, the `parse_postfix_expr` loop, the
unset (`=.`) loop, `parse_span_sugar`'s end/step decisions, `parse_blank_tail`'s
head absorption, and the form-root finishing decision. Audit `parse_slot`'s
adjacency absorption explicitly — `parser/tests/rules_slots_assignments.rs:122`
suggests it requires adjacency and so may need no site, but confirm.

**A missed site's symptom is a silently different form boundary**, so this
enumeration is the load-bearing work of the brief, not preamble.

## Step 1b — the new entry point

In `rum-syntax`, add a public entry (suggested `parse_script_forms`) that lexes
the source once via `lex_with_trivia` and drives the parser form by form.
`parse_expr` and `parse_tokens` keep their exact signatures and semantics — all
seven non-test callers elsewhere in the workspace are single-complete-expression
users and must be unaffected.

Form text is `source[start of the first token's line ..= end of the last token's
line]`, rebuilt to match today's accumulation, then trimmed.

Three details that are easy to get wrong:

1. **CRLF.** Today's accumulation is built with `str::lines()`, which strips
   `\r`. Build the form string as `slice.lines().join("\n")` (a
   `contains('\r')` fast path is fine) or byte offsets will drift on CRLF files.
2. **Hard errors.** A form containing a hard parse error ends **at the end of
   the line containing the offending token**, and the next form resumes after
   that line. This is what preserves today's poisoning resistance, pinned by
   `runtime/tests/context_and_loader.rs:609` (`"1 2\n3"` → `["1 2", "3"]` — note
   that case goes through the **`Ok`** branch, since `1 2` is `Times[1, 2]`).
3. **The tail.** When the final form ends incomplete, today's error is computed
   on the accumulated string and its span is relative to that string. Reproduce
   it by rebuilding the legacy-shaped accumulation and re-parsing it once to
   obtain the error — that keeps the message and span byte-identical at O(tail)
   cost.

## Step 1c — the lex-failure fallback

`lex_with_trivia` fails on its first error (`rum-syntax/src/lexer/mod.rs:26`),
which loses the tokens after it. Rather than propagate that, **keep today's
line-driven implementation and fall back to it whenever the whole-source lex
fails at all** — unterminated strings/comments included, because today's code
evaluates the earlier forms before reporting the tail error, and that ordering
is pinned by the rollback tests around `runtime/tests/context_and_loader.rs:646`.

State this dual path plainly in a code comment: it is a deliberate design
choice, not an oversight. Clean sources — including every quadratic case that
motivates this work — take the linear path.

Retaining the legacy function has a second payoff: it is the oracle for the
differential test below.

## Scope

Files you MAY modify:

- `crates/rum-syntax/src/` — the new entry point, its module wiring, export, and
  its own tests
- `crates/rum-runtime/src/runtime/context_and_load_helpers.rs`
- `crates/rum-runtime/src/runtime/tests/context_and_loader.rs`

Out of scope: everything else. Specifically **not** `crates/rum-repl/` (its
buffer is what a human typed, so its N is bounded), **not**
`session/session_eval.rs` (that is step 2), and **not** the span-offset defect
described next.

**Known defect, deliberately NOT fixed here**: error spans are computed relative
to the form but rendered against the whole file, so they point at the wrong
place — a probe on `q = 1\nw = 1 ! 2` reports `2:1` when the offending `!` is at
`2:7`. That is a real, pre-existing bug, and fixing it would change user-visible
output and void this brief's byte-identical acceptance criterion. Leave it
exactly as it is; the director will schedule it separately.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The stop-site enumeration with its completeness argument.
- The new entry point plus the rewired `split_script_into_forms`.
- **A differential test** — the acceptance evidence. Keep today's implementation
  in the tree (it is the fallback anyway) and assert both produce identical
  `Vec<String>` over a corpus covering at minimum: multi-line lists,
  associations and function definitions; a trailing-operator form (`1 +\n2 +\n3`);
  unterminated strings and comments; a hard error at top level; a hard error
  inside an unclosed delimiter; blank lines inside and around forms; a shebang;
  nested brackets; strings containing `[`, `]`, `;` and newlines; comments
  containing delimiters; an incomplete tail at EOF; and a CRLF variant of
  several of the above. State the corpus size.
- The before/after scaling table.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (500, 1000, 2000, 4000, 8000, 10000):
    body = ",\n".join("  %d" % i for i in range(1, n + 1))
    open("/tmp/b307-%d.rum" % n, "w").write(
        "data = {\n%s\n};\nPrint[\"len=\", Length[data]];\n" % body)
PY
for n in 500 1000 2000 4000 8000 10000; do
  echo "--- $n ---"; /usr/bin/time -p ./target/release/rum-cli /tmp/b307-$n.rum < /dev/null
done
```

Every run must print the correct `len=`, and growth must be linear rather than
~4× per doubling. Also confirm the one-line control and the
10 000-independent-statement control did not regress.

No microbench A/B: this is the script-loading path, not the evaluation hot path.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses. The differential test
  compares rum against rum and is fine.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If the differential test finds a corpus input where the
  two implementations disagree and the cause is not a missing stop site you can
  name, STOP and report the input rather than special-casing it.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file, with line ranges.
2. The stop-site enumeration and why it is complete.
3. Verification — each command with verbatim output: test counts, the
   differential corpus size, the before/after scaling table.
4. Stop-loss / open questions — any disagreement found, any site you were unsure
   about.
