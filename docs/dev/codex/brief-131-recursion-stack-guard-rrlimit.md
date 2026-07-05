# Brief 131 — Native-stack-safe recursion guard + ReplaceRepeated limit-as-value

Implementation brief with a mandatory diagnostic phase A. Fixes the two
pre-release bugs recorded in `TODO.md` §Pre-Release Fixes (found by
brief-129). Both fixes must land without regressing the evaluation hot path
(microbench A/B gate applies).

## Goal

1. `f[n_] := f[n + 1]; f[0]` (and any unbounded user recursion through
   definitions) terminates as a RECOVERABLE outcome — a
   `TerminatedEvaluation[...]` value plus message — instead of overflowing
   the native stack and killing the process, on a default 8 MiB thread.
2. `1 //. x_ :> x + 1` returns the recoverable limits-as-values outcome
   (`TerminatedEvaluation[...]` + `General::iterlim`-family message) through
   the CLI and host paths, consistent with the other budgets, instead of a
   hard `error:` exit.
3. Guide/reference stay in sync (drift test green; update the
   resource-limits chapter wording if the observed transcript changes).

## Phase A — diagnose before changing anything (paste evidence)

1. Reproduce the crash under the default build; then instrument (temporary,
   env-gated or debug print — removed before finishing) to answer:
   - at the moment of overflow, what is the WL `recursion_limit` counter
     value vs its configured limit (default 1024)?
   - approximate native stack bytes consumed per WL recursion level
     (e.g. record the address of a stack local at two depths and divide).
2. State the mechanism in one sentence: does the counter simply never trip
   on this path, or does it trip too late because per-level native frames ×
   1024 exceeds the thread stack?

## Phase B — implement per findings

Preferred design (adjust only if Phase A contradicts it; if so, STOP and
report before large changes):

- **Native stack headroom guard at the existing amortized poll point**
  (the brief-059 cancel/timeout poll in the eval loop): at evaluation start
  record a stack-base marker (address of a local as `usize`); at the poll
  point compute consumed = base - current local address; if consumed exceeds
  a conservative budget (default: leave ≥ 2 MiB headroom on an assumed
  8 MiB stack, i.e. trip around ~6 MiB consumed; make the budget a
  `HostResourceLimits` field with a documented default), terminate with the
  recoverable reason `TerminatedEvaluation["StackLimit"]` + a
  `General::reslim`-family message. Pointer-to-usize casts are safe Rust —
  **no new `unsafe`** (hard rule). No per-call cost beyond the existing
  amortized poll (default-path rule).
- Keep `recursion_limit` semantics unchanged (it still trips first when its
  count is reached at sane frame sizes).
- **ReplaceRepeated**: route the `replace_repeated_limit` overflow through
  the same recoverable terminated-evaluation mapping the other budgets use
  (pick the tag consistently with the existing family — if an established
  `IterationLimit`-style reason fits, use it; introduce a dedicated reason
  only if the existing tags would be misleading). CLI must print the value +
  message, exit code as for other recoverable outcomes; `rum-host` returns
  Success with the terminated value.

## Scope

- MAY modify: `crates/rum-runtime/src/**` (the eval-loop poll point, limit
  plumbing, `HostResourceLimits`, ReplaceRepeated limit mapping),
  `crates/rum-host/src/**` (only if the new limit field needs surfacing),
  `crates/rum-cli/src/**` (only the outcome mapping if needed), tests in the
  owning crates, `docs/guide/resource-limits-and-sandbox.md` +
  `docs/guide/evaluation.md` (transcript/wording updates for the new
  behavior), `docs/reference/` ONLY via the generator if catalog text
  changes (unlikely).
- MUST NOT touch: matcher internals, dispatch stages beyond the poll/limit
  plumbing, scripts, other docs, `.git`.

## Tests (new, in the owning crates)

- unbounded definition recursion terminates recoverably (the exact
  `f[n_] := f[n + 1]; f[0]` shape) and the session stays usable afterward;
- a legitimately deep but bounded recursion below the guard still succeeds
  (no false trip — pick a depth that passes today);
- `1 //. x_ :> x + 1` returns the terminated value + message through the
  session API; CLI-level check if a CLI test harness exists;
- existing suites stay green with UNCHANGED totals apart from added tests.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-cli -- --eval 'f[n_] := f[n + 1]; f[0]'
cargo run -q -p rum-cli -- --eval '1 //. x_ :> x + 1'
cargo test -p rum-stdlib builtin_reference_is_in_sync
```

Hot-path microbench A/B (the poll point is on the eval loop):

```
bash scripts/core-language-microbench.sh --output-dir target/brief-131-base   # on BASE (stash/worktree the change away)
bash scripts/core-language-microbench.sh --output-dir target/brief-131-new    # on the change
# paste per-label rum_seconds for both runs; any hot-label regression is a defect
```

(Probe budget: one full run per side is enough; timings.tsv is per-label.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- **No new `unsafe`** anywhere.
- No default-path cost outside the existing amortized poll point.
- Remove all Phase-A instrumentation before finishing.
- Stop-loss: same failure twice, or Phase A contradicts the design → STOP
  and report with the evidence.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Phase A findings — counter value at overflow, bytes/level, the one-line
   mechanism.
2. What changed — per file, with the chosen reason tags and default budget.
3. Verification — all commands verbatim, including the A/B per-label table.
4. Stop-loss / open questions (empty if none).
