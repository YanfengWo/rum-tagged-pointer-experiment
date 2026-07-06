# Brief 154 — Embedding adoption-cost numbers (measure + document)

Mostly-diagnostic brief: measure the numbers a light host (dun-class) asks
before pulling `rum-host` in, then write them into the embedding guide.
The only repo change is the new guide section (+ scratch measurement code
that is DELETED before finishing).

## Measurements (release profile, the pinned 1.85 toolchain, this machine;
each number reported with its exact command)

1. **Dependency footprint**: `cargo tree -p rum-host -e normal --no-dedupe
   | wc -l` and the deduped direct list (`cargo tree -p rum-host -e normal
   --depth 1`); count of external (non-rum) crates in the full normal
   graph.
2. **Binary-size delta**: build two tiny bins in a THROWAWAY scratch crate
   under `/tmp/rum-154-size/` (path-dep on the repo): (a) `fn main() {
   println!("hi") }`; (b) create a `HostSession`, eval `1 + 1`, print the
   result. Both `--release`, then `strip`ped copies too. Report the four
   sizes and the deltas. (The scratch crate lives OUTSIDE the repo; nothing
   in-repo references it.)
3. **Cold-start time**: in the scratch bin, measure with `std::time::
   Instant`: (a) `HostSession::new()` (which installs core) and (b) the
   first `eval("1 + 1")`, separately; loop 10 iterations in ONE process
   (fresh sessions each) and report min/median; ALSO report the first
   iteration separately (true cold, includes lazy statics). No profiling
   env; release build.
4. **Session memory ballpark**: `MemoryInUse[]` via a Diagnostic-allowed
   session right after install (one number, labeled as the cooperative
   proxy, not RSS).

## Documentation

Add an "Adoption Cost" section to `docs/guide/embedding.md`: the four
numbers with the machine/toolchain context line ("measured on <cpu>,
rustc 1.85.0, release, YYYY-MM-DD"), one sentence on the feature-gating
story (a host that does not construct a session pays only the link cost),
and one sentence noting numbers will be refreshed when the workspace
changes materially. Keep it to ~20 lines; numbers rounded sensibly.

## Scope

- MAY modify: `docs/guide/embedding.md` ONLY.
- Scratch measurement code under `/tmp/rum-154-size/` (outside the repo,
  left behind is fine; nothing may reference the repo checkout path inside
  committed files).
- MUST NOT touch anything else in the repo.

## Verification (MANDATORY — paste verbatim)

```
# every measurement command + raw output (persist to /tmp files first)
cargo fmt --check          # trivially green (docs-only)
git status --short         # exactly one modified file
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network
  (path-dep scratch crate; the cargo cache has everything).
- Measurement hygiene: release only, no profiling env, raw outputs saved
  to /tmp before parsing, report medians not cherry-picks.
- Stop-loss: same failure twice → STOP and report.

## Report format

1. The four numbers with commands, raw values, and the exact guide text
   added.
2. Anything surprising (e.g. an unexpectedly heavy dependency) —
   report-only.
3. Stop-loss / open questions.
