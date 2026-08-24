# brief-309 — the remaining native-stack crash sites in rum-runtime

## Goal

Four more places still abort the process with a native stack overflow on wide
input. Make each of them iterate. When you are done, every probe in the table
below exits 0, and `Reap` no longer copies its accumulated state once per
bucket.

This continues brief-304, which converted six tree walkers the same way and
whose `pending: Option<Outcome>` pattern is the template here.

## Measured today (release CLI, verified by the director — do not re-derive)

| input | result |
|---|---|
| `ReplaceList[1, Table[i -> 0, {i, 1, 10000}]]` | `exit=-6`, stack overflow |
| `Reap[Do[Sow[i, i], {i, 1, 12000}]]` | `exit=-6` |
| 10 000 `UpValue` definitions on one symbol, then a call | `exit=-6` |
| 10 000 `SubValue` definitions, then a call | `exit=-6` |

**Also measured, and deliberately NOT in scope: `DownValue` dispatch does not
crash.** 10 000 `f[n] = v` definitions followed by `f[5000]` returns correctly.
An earlier survey listed `down_values.rs` alongside the others on a structural
reading; the probe says otherwise. Do not touch it.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/definitions/application/up_values.rs`
- `crates/rum-runtime/src/runtime/definitions/application/sub_values.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs`
- `crates/rum-runtime/src/runtime/debug_tools/reap.rs`
- test modules under `crates/rum-runtime/src/runtime/tests/` for the new tests

Out of scope: everything else. Specifically **not**
`definitions/application/down_values.rs` (measured fine, see above), and **not**
`rum-stdlib`'s string walks. `StringReplace`'s list form aborts at 50 000
strings, but it also clones the remaining input and the accumulated output per
element — a separate O(m²) that must be fixed in the same change as its
recursion, so it gets its own brief.

Do not add unconditional cost to the default evaluation path.

## The three shapes, and what each needs

They are not all the same recursion, so read each before converting it.

**1. Continuation self-call** — `up_values.rs:152`, `sub_values.rs:187`. A
`drive_internal_value` continuation calls the enclosing function again for the
next candidate rule/symbol. This is exactly brief-304's shape: convert to a loop
carrying the index, and add a trailing `pending: Option<…>` parameter so a
resume frame re-enters the loop at the stored index with the child outcome. Copy
that structure rather than inventing a variant.

**2. Per-rule recursion** — `list.rs:102`, `:128`. `ReplaceList` recurses once
per *rule* rather than per element, so the depth is the rule count. Same
conversion, but the loop variable is the rule index; make sure the accumulated
match list keeps its order (rule order is observable).

**3. Direct tail recursion** — `reap.rs:158` (`resume_reap_buckets_from_index`)
and the group-level `resume_reap_scope_groups_from_index` just above it. These
call themselves directly with `index + 1`; Rust does not eliminate the tail
call, so each bucket costs a frame. Convert both to loops.

`Reap` also needs a second fix in the same change: `reap.rs:172-174` does
`state.clone()` and `rendered.clone()` **per bucket**, which is O(n²) in the
bucket count. Once the frames are gone that copying becomes the binding cost, so
removing the crash without removing the copies would just trade one wide-input
failure for another. Carry the state by value through the loop and clone only
what a suspension actually needs to store.

## Deliverable

- Each of the four sites iterating, with suspension and resume preserved.
- `Reap` no longer cloning its state and accumulated output per bucket.
- Per site, a test at a width well past the measured abort threshold (30 000 is
  a reasonable target; `ReplaceList` and the value dispatchers are measured to
  die at 10 000, `Reap` at 12 000). Report each new test's runtime, and say so
  rather than trimming if one exceeds ~10 s in debug.
- For `Reap`, a scaling check showing the bucket count is no longer quadratic.

## rum / Rust pitfalls (read twice)

1. **Suspension safety is the hard part.** These are `eval_*`/`resume_*` paths.
   Preserve the suspend/resume continuation shape; a suspended traversal must
   resume at the same index with the same accumulated state. If a site's
   suspension shape resists the conversion, STOP and report that site.
2. **Do not trade a crash for an O(n²).** Removing frames must not introduce a
   per-element copy of the accumulator. This is not hypothetical — it is exactly
   what `Reap` already does.
3. **Order is observable.** Rule order in `ReplaceList`, bucket order in `Reap`
   (first-appearance order), and candidate order in the value dispatchers all
   show up in results. Keep them.
4. **`Reap`'s handler runs user code.** The per-bucket handler application can
   suspend and can fail; keep both paths behaving as they do now.
5. **One variable per measurement.** Convert, test, then measure.
6. **Stop-loss is real.** If the same site fails twice for the same reason, stop
   and report with the partial work left in the tree.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli`, run each probe and paste
its verbatim output **and exit code**. Feed stdin from `/dev/null` — the CLI
waits for input otherwise:

```
cat > /tmp/b309-1.rum <<'EOF'
r = Table[i -> 0, {i, 1, 30000}];
Print["replacelist: ", Head[ReplaceList[1, r]]];
EOF
cat > /tmp/b309-2.rum <<'EOF'
Print["reap: ", Length[Last[Reap[Do[Sow[i, i], {i, 1, 30000}]]]]];
EOF
python3 - <<'PY'
u = ["g /: h%d[g] := %d;" % (i, i) for i in range(1, 30001)]
open("/tmp/b309-3.rum", "w").write("\n".join(u) + '\nPrint["upvalue: ", h15000[g]];\n')
s = ["s[%d][x_] := %d;" % (i, i) for i in range(1, 30001)]
open("/tmp/b309-4.rum", "w").write("Clear[s];\n" + "\n".join(s) + '\nPrint["subvalue: ", s[15000][1]];\n')
PY
for i in 1 2 3 4; do
  echo "--- $i ---"; ./target/release/rum-cli /tmp/b309-$i.rum < /dev/null; echo "exit=$?"
done
```

Every one must exit 0 with the correct value. **An exit code of -6 / 134
anywhere is a failure of this brief.**

For the `Reap` scaling check, use the kernel's own `AbsoluteTiming` (not
`/usr/bin/time`, whose resolution is coarser than these probes) over bucket
counts 2000 / 4000 / 8000 and show the growth is linear rather than ~4× per
doubling.

Whether a microbench A/B is needed is your call to argue: the value dispatchers
sit on the evaluation path, `Reap` and `ReplaceList` do not. If you touch the
dispatch path in a way that could cost the common single-rule case, run the
gate and paste the affected labels plus same-tier neighbors, with a plain
statement of whether the machine was quiet.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts if you argue for them,
  and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file and per site, with line ranges, which of the three
   shapes it was, and how the suspension state is now carried.
2. Verification — each command with verbatim output and exit codes, the four
   30 000-width probes, the new tests' runtimes, and the `Reap` scaling rows.
3. The verdict — plus, for `Reap`, what the per-bucket copying cost before.
4. Stop-loss / open questions.
