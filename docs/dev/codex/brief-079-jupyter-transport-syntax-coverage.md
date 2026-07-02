# brief-079 — coverage cleanup: jupyter JSON transport + syntax trivia/ast (test-only)

## Goal

Finish the coverage-driven test buildout's lower-priority targets, covering REAL logic and
skipping false vacuums. TWO independent areas, both TEST-ONLY (no product-code change):
- **rum-jupyter JSON transport** — the Jupyter wire-protocol serialization (bytes ↔
  structs), pure logic testable WITHOUT sockets/network.
- **rum-syntax `trivia.rs` + `ast.rs`** — trivia (comment/whitespace) handling and AST
  node methods.
If any test reveals a bug (malformed-input mishandling, trivia mis-attachment), REPORT it
(leave the failing test) — do not weaken the assertion.

## Scope of what to cover (target real logic; SKIP the false vacuums)

### A. rum-jupyter JSON transport (wire protocol, unit-testable)
Cover, via bytes↔struct round-trips + edge/malformed inputs:
- `transport/json/parse/{messages.rs (51%), fields.rs (44%), status.rs (43%)}` — parsing
  Jupyter wire messages / header fields / status; cover the message kinds + field types +
  MALFORMED/missing-field/wrong-type inputs (do they error gracefully, not panic?).
- `json_support.rs (62%)` — the JSON helper accessors (typed getters, defaults, error paths).
- `transport/codec.rs (60%)` + `transport/codec/validation.rs (82%)` — the wire codec
  (framing, HMAC signature validation): valid frame round-trips + tampered/short/bad-sig
  frames rejected.
- `transport/json/encode.rs (72%)` — encoding structs → wire JSON (round-trip with parse).
- `transport/local.rs (77%)` — the in-process transport (testable without sockets).

**SKIP (false vacuums — do NOT try to unit-test):** `launcher.rs` + `bin/rum-jupyter-kernel.rs`
(process entry points, 0%, need e2e), and `zeromq.rs`/`zeromq_native/*`/`socket.rs`/
`connection.rs` (real-socket network I/O — integration, not unit). Note them as skipped.

### B. rum-syntax trivia + ast
- `trivia.rs (28%)` — trivia (comment/whitespace) capture + attachment: parse sources with
  leading/trailing/inline comments + varied whitespace and assert the trivia is captured/
  attached where expected (round-trip through the formatter if that's how trivia surfaces).
- `ast.rs (65%, 91 missed)` — AST node methods (constructors, accessors, `Display`/format,
  the less-common node kinds); cover the untested node kinds + methods directly and/or via
  parse.

## rum / test pitfalls (read twice)

1. **Test real logic, skip integration-only false vacuums** (the SKIP list above). Do not
   write a "test" that needs a live socket/process — that's not a unit test.
2. **Malformed-input paths are the point** for the transport parse/codec: feed bad/short/
   missing/wrong-type/tampered inputs and assert graceful errors (not panics). Those error
   arms are much of the missed coverage.
3. **Real assertions** (parsed value / round-trip equality / specific error), not `is_ok()`.
   For a codec, a valid→encode→parse→equal round-trip + a tampered→rejected pair.
4. **Report bugs as findings** (leave the failing test); do not weaken.
5. Match each crate's existing test style; add test modules under the crate's `src`
   (`#[cfg(test)]`) or `tests/` as that crate already does.

## Scope (files)

- MAY modify: test code in `crates/rum-jupyter/` and `crates/rum-syntax/` only (new
  `#[cfg(test)]` modules / `tests/` files + minimal mod wiring). NO product-code change
  under non-test `src`.
- MUST NOT: edit any `mma-hol` file; `Expr`/serde; no new `unsafe`; no `git commit`.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-jupyter            # green (paste before/after count)
cargo test -p rum-syntax             # green (paste before/after count)
cargo test --workspace               # no regression (paste the totals)
cargo llvm-cov --workspace --summary-only 2>&1 | grep -E "rum-jupyter/src/transport|json_support|rum-syntax/src/(trivia|ast)"
# paste the targeted files' coverage before -> after
```

Paste: both per-crate test counts (before → after), the workspace total, the targeted
files' coverage before→after, and any bug a test revealed (failure + analysis).

## Hard rules

- No `git commit`/branch/push. Never edit any `mma-hol` file. Test-only. No new `unsafe`.
- Skip the integration-only false vacuums (launcher/bin/zeromq/socket/connection). Cover
  the real wire-protocol + trivia/ast logic.
- Real assertions incl. malformed-input error paths; report bugs as findings.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — new test modules per area/file, counts; confirm no product-code change;
   confirm the false vacuums were skipped (list them).
2. Verification — rum-jupyter + rum-syntax + workspace test counts, targeted-file coverage
   before→after — verbatim.
3. Findings — any bug revealed (malformed-input/codec/trivia), or "all covered behavior
   holds."
4. Open questions — any file arm unreachable via unit tests (with why).

---

## Outcome (Claude gate, 2026-07-01) — DONE + KEEP. jupyter transport + syntax trivia/ast covered; no bug.

Committed test-only (verified: all 6 modified `src` files are additions to existing
`#[cfg(test)] mod tests` or new test files — the lone "deletion" in json_support.rs is an
import-line expansion; NO product-logic change). rum-jupyter 80→96, rum-syntax 146→152;
workspace 1251 passed / 0 failed / 3 ignored (Claude reproduced jupyter 96/0 + syntax 152/0).
Coverage jumps: trivia.rs 28→99%, status.rs 43→100%, fields.rs 44→80%, json_support 62→80%,
encode 73→86%, local 77→84%, ast.rs 66→86%, codec/validation 82→89%. No bug (malformed
JSON / tampered codec / trivia all handled gracefully). False vacuums skipped as intended
(launcher/bin/zeromq/socket/connection — covered by the e2e/browser smoke layer instead).

Test-buildout line (unit) DONE: host/lifecycle (076), Orderless+Optional (077, 4 bugs
queued), gensym-GC (078), jupyter transport + syntax trivia/ast (079). Next: run the
jupyter e2e/browser smoke (scripts/rum-jupyter-*smoke*) to verify the live-kernel path that
unit tests can't reach.
