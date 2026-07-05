# Brief 132 — Strip the six general file builtins (archive for future rum-fs)

Implementation brief. First execution step of `docs/dev/fs-purity-plan.md`
(read it first). Removal + archive, mirroring the brief-124/125 pattern.

## Goal

`FileNames`, `FileExistsQ`, `FileDate`, `FileByteCount`, `Import`, and
`FileTemplateApply` no longer exist in the tree: implementations,
registrations, capability-match arms, catalog/attribute entries, and their
tests are removed (verbatim copies archived first); the generated builtin
reference is regenerated (six fewer entries, drift test green); the listed
guide lines are updated; the workspace is green. The module system
(`Get`/`Needs`/`FindFile`) and persistence (`Save`/`DumpSave`) are untouched
and still fully covered by tests.

## Context pointers

- `docs/dev/fs-purity-plan.md` — disposition table and end state.
- Implementations: the five kernel-resident ones in
  `crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs`;
  `FileTemplateApply` in `crates/rum-stdlib/src/strings/template.rs`
  (`eval_file_template_apply`). VERIFY the pure template machinery
  (`TemplateApply`/`StringTemplate`-style evaluation, if registered) stays.
- Registrations/attributes: find via
  `rg -n 'FileNames|FileExistsQ|FileDate|FileByteCount|"Import"|FileTemplateApply' crates/`
  — includes the exhaustive no-wildcard capability match (brief-095), catalog
  specs/attribute tables, and adapter glue (`builtin_adapters.rs`,
  `stdlib_attributes.rs`).
- Archive root: `/Users/fft/Developments/Archive/rum-history/` (MANIFEST
  format as before; 移除 commit placeholder exactly `（由 Claude 在提交时回填）`).

## Scope

- MAY modify/delete (inside the repo): the six builtins' implementation
  code, registrations, capability arms, catalog/attribute entries, and the
  test functions/files that cover them (in `rum-runtime` and `rum-stdlib`
  test trees); `docs/reference/builtins.md` via the generator only; the two
  guide files ONLY on the lines listed below.
- MAY write outside the repo: ONLY under
  `/Users/fft/Developments/Archive/rum-history/` — verbatim copies of every
  file you delete AND of every file you edit whose removed content is
  substantial (for partial edits, archive the pre-edit file under its
  mirrored path with suffix `.pre-brief-132`), plus ONE MANIFEST entry:
  性质 = general fs builtins stripped for kernel purity; 回收线索 = **the
  seed implementation for the future downstream `rum-fs` project
  (post-launch)** — flag prominently.
- MUST NOT touch: `Get`/`Needs`/`FindFile`/`Save`/`DumpSave` semantics or
  tests, any evaluator/dispatch path beyond deleting the six's plumbing,
  `Cargo.toml`, scripts, other docs.

## Guide edits (exactly these)

- `docs/guide/resource-limits-and-sandbox.md` §Capability Classes: the
  FileRead example list becomes `Get`, `Needs`, `FindFile`; the FileWrite
  list becomes `Save`, `DumpSave`. Add one sentence: general file utilities
  are deliberately outside the kernel; the filesystem surface is the module
  system and session persistence only.
- `docs/guide/design-boundaries.md` §Filesystem Is A Capability: same
  adjustment — state that the kernel ships no general file utilities and
  that a broader filesystem surface is future downstream work.
- If any other guide line names a removed builtin
  (`rg -n 'FileNames|FileExistsQ|FileDate|FileByteCount|Import|FileTemplateApply' docs/guide/`),
  update it minimally and list it in the report.

## Steps

1. Archive copies FIRST (mirrored paths + `.pre-brief-132` for partial
   edits); verify each with `diff -q`.
2. Remove implementations, registrations, capability arms, attribute/catalog
   rows, and covering tests. The exhaustive capability match must still
   compile (its default-deny structure forces you to remove the arms —
   that is the intended safety net).
3. Regenerate the reference:
   `cargo run -q -p rum-stdlib --bin generate-builtin-reference`.
4. Guide edits per the list above.
5. Sweeps and gates below.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
rg -in 'FileNames|FileExistsQ|FileDate|FileByteCount|FileTemplateApply' crates/ docs/guide/ docs/reference/ || echo STRIP-CLEAN
rg -c '^### ' docs/reference/builtins.md          # expect 507
rg -n '"Import"|\bImport\[' crates/ docs/guide/ || echo IMPORT-CLEAN
cargo run -q -p rum-cli -- --eval 'FileExistsQ["/tmp"]'    # expect inert/undefined-symbol behavior, paste what happens
cargo run -q -p rum-cli -- --eval 'Needs["GuidePkg`", "/tmp/rum-guide-pkg/GuidePkg.rum"]; GuidePkg`answer'   # module system intact (create the pkg file first as in the guide)
git status --short
```

Note: after removal the six are ordinary undefined symbols — paste the
actual `FileExistsQ["/tmp"]` result so the report records the new behavior.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Outside-repo writes ONLY under the archive root.
- Zero behavior change to anything not on the removal list — loader and
  persistence tests must pass unchanged.
- No new `unsafe`. Minimal diff. Stop-loss: same failure twice → STOP.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file (deleted / edited / archived), with the
   registration/capability/attribute sites listed.
2. Verification — each command with verbatim output.
3. Guide lines touched beyond the two listed sections (if any).
4. Stop-loss / open questions (empty if none).
