# Brief 126 — Archive closed-line docs and historical dev briefs

Implementation brief. Third step of `docs/dev/release-transformation-plan.md`
(read its Decision + Policy + docs inventory sections first). Docs-only: no
crate source, no scripts.

## Goal

`docs/` at HEAD contains only current-truth documents. The closed-line plans/
audits listed below and the historical dev briefs 001-123 are copied verbatim
into the private archive and deleted from the tree. Nothing else changes.

## Context pointers

- `docs/dev/release-transformation-plan.md` — the authoritative inventory.
- Archive root: `/Users/fft/Developments/Archive/rum-history/` (exists; has
  `MANIFEST.md`, `WLBuiltin/`, `crates/`, `scripts/`).

## Scope

- DELETE from the repo (after archiving) — 20 files under `docs/`:
  `bytecode-vm-closure-plan.md`, `bytecode-vm-design.md`,
  `bytecode-vm-general-plan.md`, `bytecode-vm-plan.md`,
  `bytecode-vm-research-notes.md`, `core-language-support-matrix.md`,
  `crate-rebalance-plan.md`, `evaluator-performance-audit.md`,
  `jupyter-host-migration-audit.md`, `layer-b-node-rewrite-plan.md`,
  `long-tail-gap-audit.md`, `memory-management-plan.md`,
  `mma-hol-smoke-and-snapshot.md`, `next-session-handoff.md`,
  `performance-kernel-rewrite-plan.md`, `performance-stage0-baseline.md`,
  `PROGRESS-archive-2026-07-03.md`, `proptaut-performance-audit.md`,
  `representation-rewrite-plan.md`, `wolfram-builtin-inventory.md`.
- DELETE (after archiving): `docs/dev/kernel-api-audit-2026-07-03.md`.
- DELETE (after archiving): every `docs/dev/codex/brief-*.md` with number 001
  through 123 (123 files). KEEP: `TEMPLATE.md`, `brief-124-*`, `brief-125-*`,
  `brief-126-*` (this brief), and `docs/dev/release-transformation-plan.md`.
- MAY create/modify outside the repo (EXPLICIT EXCEPTION): only under
  `/Users/fft/Developments/Archive/rum-history/` — mirror copies under `docs/`
  (preserving subpaths, e.g. `docs/dev/codex/brief-001-….md`), and appending
  to `MANIFEST.md`.
- MUST NOT touch: every other file. In particular do NOT edit the KEEP-list
  docs even where they link to the archived files — dangling links are handled
  by M1/brief-127. REPORT them only.

## Steps

1. Archive copies FIRST, mirroring paths:
   ```
   mkdir -p /Users/fft/Developments/Archive/rum-history/docs/dev/codex
   cp <each of the 20 docs files> /Users/fft/Developments/Archive/rum-history/docs/
   cp docs/dev/kernel-api-audit-2026-07-03.md /Users/fft/Developments/Archive/rum-history/docs/dev/
   cp docs/dev/codex/brief-0*.md docs/dev/codex/brief-1*.md (001-123 only) \
      /Users/fft/Developments/Archive/rum-history/docs/dev/codex/
   ```
   Select the brief range robustly (e.g. a shell loop over the numeric prefix,
   001..123) — do NOT copy 124/125/126 or TEMPLATE.md.
2. Verify: `diff -q` per copied file (or `diff -rq` on the mirrored dirs after
   excluding the kept files) → no output; then count checks:
   archived docs files = 20, dev = 1, codex briefs = 123.
3. Append TWO grouped entries to the archive `MANIFEST.md` (existing format):
   - "docs/ closed-line plans & audits"（20 files + kernel-api-audit）: 性质 =
     closed/paused-line plan and audit documents (perf lines, memory line,
     bytecode-VM notes, WK-comparison audits, support matrix, progress
     archive); 移除 commit = exactly `（由 Claude 在提交时回填）`; 回收线索 =
     design-history reference; the bytecode-VM notes are the most likely
     future recycle (the line is paused, not dead).
   - "docs/dev/codex briefs 001-123": 性质 = historical Codex task briefs of
     all pre-release lines; 移除 commit = same placeholder; 回收线索 = none
     (process record only).
4. `git rm` all archived files.
5. Residue check (report-only): `rg -ln 'core-language-support-matrix|next-session-handoff|PROGRESS-archive|mma-hol-smoke' README.md ARCHITECTURE.md AGENTS.md TODO.md docs/` — list hits as handoff to M1/127.

## Deliverable

- The deletions, uncommitted in the working tree.
- Archive copies verified; two MANIFEST entries appended.
- Verbatim verification output.

## Verification (MANDATORY — run all, paste verbatim)

```
find /Users/fft/Developments/Archive/rum-history/docs -type f | wc -l          # expect 144
ls /Users/fft/Developments/Archive/rum-history/docs | head -30
ls docs/ | sort
ls docs/dev/ ; ls docs/dev/codex/ | sort
git status --short | head -20 ; git status --short | wc -l
<the diff -q / count checks from step 2>
<the residue rg from step 5>
```

Expected `docs/` after this brief: `code-organization-guidelines.md`,
`formatter-style.md`, `host-adapter-plan.md`, `host-embedding-and-hardening.md`,
`jupyter-console-manual.md`, `lowering-canonical-forms.md`,
`python-adapter-plan.md`, `runtime-evaluation-audit.md`, `testing-strategy.md`,
`v0.1-acceptance-boundary.md`, `dev/`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Outside-repo writes ONLY under `/Users/fft/Developments/Archive/rum-history/`.
  No network. Never touch any `mma-hol` clone.
- Docs moves only — zero edits to file contents, zero edits to kept files.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — the three archived groups with counts.
2. Verification — each command with verbatim output.
3. Residue handoff — dangling references in kept files (list only).
4. Stop-loss / open questions (empty if none).
