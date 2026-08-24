# brief-311 — linear string list walks

## Goal

`StringReplace` and `StringCases` over a list of strings clone their entire
remaining input and their entire accumulated output **once per element**, and
recurse once per element on top of that. So they are quadratic and they abort
the process on a wide list. Make both iterate and stop copying; a 50 000-string
call must succeed, and the time must grow linearly.

## Measured today (release CLI, verified by the director — do not re-derive)

`StringReplace[Table["ab" <> ToString[i], {i, n}], "b" -> "x"]`:

| n | time |
|---|---|
| 2000 | 0.173 s |
| 4000 | 0.674 s |
| 8000 | **2.732 s** |
| 50000 | `exit=-6`, stack overflow |

`StringCases[…, "b" -> "x"]` — the **rule** form — behaves identically:

| n | time |
|---|---|
| 2000 | 0.224 s |
| 4000 | 0.827 s |
| 8000 | **3.392 s** |
| 50000 | `exit=-6`, stack overflow |

~4× per doubling in both.

**Do not be misled by a literal-pattern comparison.** `StringCases[…, "b"]`
with a literal pattern is linear (0.006 s at 2000, 0.022 s at 8000) because
`search.rs:268` only routes to the per-element walk when the spec is
`Rules(_)`; a literal pattern takes a different path. An earlier survey cited
that literal measurement as proof that "a correct implementation already exists
in-tree" — it does not. The rule path of `StringCases` has the same defect as
`StringReplace`, which is why both are in scope here.

## The mechanism

`crates/rum-stdlib/src/strings/patterns/search/rules.rs`:

- `resume_replace_string_values_from_index` (`:325-364`) — `:341-342` does
  `state.clone()` and `rendered.clone()` **unconditionally, before knowing
  whether the child suspends**, then the continuation calls back into the walk
  for the next index (`:363`).
- `resume_collect_string_cases_values_from_index` (`:517-545`) — the same shape
  with `outputs.clone()`.

`state` holds the whole remaining `values` list, so each element copies O(m)
and the walk copies O(m²) in total; the per-element continuation call is what
exhausts the native stack.

This is exactly the shape brief-309 fixed in `Reap`'s bucket rendering
(`e0efd95`): convert the walk to a loop, and move the state into a suspension
frame **only on the suspending path**, so the non-suspending common case copies
nothing. Use that commit as the template.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/strings/patterns/search/rules.rs`
- `crates/rum-stdlib/src/strings.rs` — **only** if you confirm the third site
  below is reachable
- the `rum-stdlib` test modules for these builtins

Out of scope: everything else, including the literal-pattern paths (they are
already linear — do not "unify" them into the same code), the TOML codecs, and
`rum-runtime`.

**A third site, unverified.** `strings.rs` around `:192` has the same
`state.clone()` / `outputs.clone()` per element plus a
`resume_string_apply_with_value` continuation, for `StringApply`. The director
could not construct a call that reaches it — `StringApply["abc", Length]`
returns unevaluated — so there is no measurement for it. **First determine
whether it is reachable at all.** If it is, fix it the same way and report your
probe. If it is not reachable from any documented call form, say so and leave
the file alone; do not fix code you cannot demonstrate is live.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- Both list walks iterating, with no per-element clone of state or output.
- Suspension and resume preserved: a suspended walk must resume at the same
  index with the same accumulated output. Add a test that suspends mid-walk and
  checks the accumulated result, as brief-304's `cases_resumes_mid_walk…` does.
- A 50 000-element test per builtin, asserting the result — these also guard the
  crash. Report each new test's runtime; say so rather than trimming if one
  exceeds ~10 s in debug.
- Before/after scaling for n = 2000 / 4000 / 8000 on both builtins.
- The `StringApply` reachability verdict.

## rum / Rust pitfalls (read twice)

1. **Suspension safety is the hard part.** These are `eval_*`/`resume_*` paths;
   rules can be `RuleDelayed` and run user code, which can suspend or fail.
   Both paths must behave as they do now.
2. **Do not trade the copies for a different copy.** Removing the clone must not
   introduce a per-element rebuild of the output vector.
3. **Order and count are observable.** Result order, and the number of times a
   delayed rule's RHS is evaluated, both show up. Keep them.
4. **`max_replacements` / `max_matches` semantics are per element** — check this
   in the code before restructuring the loop, and keep whatever it is.
5. **Stop-loss is real.** If a site's suspension shape resists the conversion,
   stop and report that site.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
cat > /tmp/b311-1.rum <<'EOF'
Print["replace: ", Length[StringReplace[Table["ab" <> ToString[i], {i, 50000}], "b" -> "x"]]];
EOF
cat > /tmp/b311-2.rum <<'EOF'
Print["cases: ", Length[StringCases[Table["ab" <> ToString[i], {i, 50000}], "b" -> "x"]]];
EOF
for i in 1 2; do echo "--- $i ---"; ./target/release/rum-cli /tmp/b311-$i.rum < /dev/null; echo "exit=$?"; done
```

Both must exit 0 with 50000. **An exit code of -6 / 134 is a failure of this
brief.** Then report the scaling table using the kernel's own `AbsoluteTiming`,
not `/usr/bin/time`, whose resolution is coarser than the smaller probes.

Also confirm the literal-pattern path did not regress:
`StringCases[Table[…], "b"]` at 8000 should stay around 0.022 s.

No microbench A/B: these are string builtins, not the evaluation hot path. Say
so if you believe otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.**
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges, and how suspension state is
   now carried.
2. Verification — each command with verbatim output and exit codes: the two
   50 000 probes, the new tests' runtimes, the before/after scaling for both
   builtins, and the literal-path control.
3. The `StringApply` reachability verdict, with the probe you used.
4. Stop-loss / open questions.
