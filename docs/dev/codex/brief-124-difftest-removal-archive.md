# Brief 124 — Remove `rum-difftest` + archive to `../Archive/rum-history`

Implementation brief. First execution step of
`docs/dev/release-transformation-plan.md` (read its Decision + Policy sections
first — they explain why this removal happens and what the archive is).

## Goal

The workspace no longer contains the differential-testing crate or the
WolframKernel builtin-export script: `crates/rum-difftest/` and
`scripts/export-wl-builtins.wls` are deleted from the tree (after being copied
verbatim into the private archive), the workspace `Cargo.toml` / `Cargo.lock`
no longer reference `rum-difftest`, `.gitignore` no longer carries the
`WLBuiltin/` entry, and the full workspace builds and tests green.

## Context pointers

- `docs/dev/release-transformation-plan.md` — the plan this brief executes.
- Archive root: `/Users/fft/Developments/Archive/rum-history/` (EXISTS; already
  contains `MANIFEST.md` and `WLBuiltin/`). Archived items mirror repo paths.
- `crates/rum-difftest/` — the whole crate to remove (oracle lanes, curated
  cases, inventory baseline, perf baselines, bins).
- `scripts/export-wl-builtins.wls` — tracked script that drives WolframKernel;
  removed under the same policy.

## Scope

- Files you MAY modify (inside the repo):
  - DELETE `crates/rum-difftest/` (entire directory, via `git rm -r`);
  - DELETE `scripts/export-wl-builtins.wls` (via `git rm`);
  - `Cargo.toml` (workspace root): remove the `crates/rum-difftest` member (and
    any other workspace-level reference to it, e.g. default-members, if present);
  - `Cargo.lock`: only the mechanical regeneration produced by cargo after the
    member removal;
  - `.gitignore`: remove the `WLBuiltin/` line only.
- Files you MAY create/modify (outside the repo — EXPLICIT EXCEPTION for this
  brief): under `/Users/fft/Developments/Archive/rum-history/` only —
  `mkdir -p` mirror directories, the two archive copies, and appending entries
  to its `MANIFEST.md`. Touch nothing else outside the repo.
- MUST NOT touch: any other crate's source, `docs/**` (residual `rum-difftest`
  mentions in docs/README/AGENTS are handled by later briefs — REPORT them,
  do not fix), `scripts/` beyond the one listed file, `.git`, git config, any
  `mma-hol` clone.

## Steps

1. Archive copies FIRST (verbatim, before any deletion):
   ```
   mkdir -p /Users/fft/Developments/Archive/rum-history/crates
   mkdir -p /Users/fft/Developments/Archive/rum-history/scripts
   cp -R crates/rum-difftest /Users/fft/Developments/Archive/rum-history/crates/rum-difftest
   cp scripts/export-wl-builtins.wls /Users/fft/Developments/Archive/rum-history/scripts/
   ```
   Verify the copy: `diff -rq crates/rum-difftest /Users/fft/Developments/Archive/rum-history/crates/rum-difftest` → no output.
2. Append two entries to `/Users/fft/Developments/Archive/rum-history/MANIFEST.md`
   following the existing entry format (原路径 / 性质 / 移除 commit / 回收线索).
   For 移除 commit write exactly: `（由 Claude 在提交时回填）`. For 回收线索 on the
   crate entry, note the three recycling candidates: curated
   `cases/wolfram/core/*.txt` vectors (recyclable per-batch iff their expected
   values are promoted into rum's own reference as design commitments), the
   `src/inventory/` catalog-drift baseline tool (rum-local, recyclable renamed),
   and the `lane_d` rum-vs-rum baseline mechanism.
3. `git rm -r crates/rum-difftest` and `git rm scripts/export-wl-builtins.wls`.
4. Edit root `Cargo.toml`: drop the member. Edit `.gitignore`: drop `WLBuiltin/`.
5. Regenerate the lock mechanically: `cargo check --workspace` (or the test run
   below) and confirm `rg -c 'rum-difftest' Cargo.lock` → no hits.
6. Residue sweep (report verbatim): `rg -n 'rum[-_]difftest' Cargo.toml Cargo.lock crates/ scripts/ .gitignore`
   must be EMPTY; `rg -ln 'rum[-_]difftest' README.md AGENTS.md TODO.md docs/`
   is EXPECTED to have hits — list them in the report as handoff to briefs
   126/127 and M1, do NOT edit those files.

## Deliverable

- The deletions + the three small file edits above, left uncommitted in the
  working tree.
- Archive copies verified by `diff -rq`; MANIFEST entries appended.
- Verification output pasted verbatim.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
rg -n 'rum[-_]difftest' Cargo.toml Cargo.lock crates/ scripts/ .gitignore || echo CLEAN
rg -ln 'rum[-_]difftest' README.md AGENTS.md TODO.md docs/
diff -rq crates_removed_check 2>/dev/null; ls crates/ | sort
```

(The last line: just show `ls crates/` proving `rum-difftest` is gone.)
`cargo test --workspace` must be GREEN — if any pre-existing failure appears,
STOP and report it verbatim instead of fixing unrelated code.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- Outside-repo writes are allowed ONLY under
  `/Users/fft/Developments/Archive/rum-history/` as listed. No network.
- Do NOT edit any file outside Scope — residual mentions elsewhere are
  report-only handoffs.
- No new `unsafe` (trivially satisfied — no code is written).
- Minimal diff: no drive-by reformatting or renames anywhere.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per path (deleted / edited / archived), one line each.
2. Verification — each command with verbatim output (test totals, sweep results).
3. Residue handoff list — every remaining `rum-difftest` mention outside Scope.
4. Stop-loss / open questions (empty if none).
