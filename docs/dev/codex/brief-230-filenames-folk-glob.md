# Brief 230 — FileNames folk glob through the FileIo provider (FileRead-gated)

Fix brief — unit-10 decision fix 3 (USER-RULED 2026-07-14). Source:
`docs/dev/usability-pass/kernel-c-findings.md` (the FileNames
candidate-addition row) + the unit-9 mma-hol gate discovery (run_all.wls
uses the read-only glob idiom, which silently no-ops because FileNames is
absent). Restore FileNames as a folk directory glob, routed through the
`FileIo` provider seam and gated by `Capability::FileRead` — so it is
denied in `sandboxed()` (no provider) and available in trusted/CLI
contexts.

## Background: why it was removed and how it must come back

`FileNames` existed pre-brief-132; the FS-purity line removed it because
its implementation called `std::fs::read_dir` DIRECTLY (archived at
`../Archive/rum-history/crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs.pre-brief-132`).
The FS-purity line routes ALL filesystem access through the `FileIo`
provider trait (`runtime/file_io.rs`), which today has `read_to_string`,
`probe`, and `write_string` — but NO directory enumeration. So restoring
FileNames requires one new provider method; direct `std::fs` in the
kernel is a hard-line violation (zero std::fs in non-test kernel code).

This provider-seam expansion is deliberate and doctrine-consistent:
directory enumeration is a natural sub-capability of `FileRead` (a
trusted embedding granted FileRead reasonably lists directories, not just
reads named files), it stays behind the same capability gate and the same
provider seam, and `sandboxed()` (no provider) denies it exactly as it
denies `Get`.

## Required change

1. **Provider method.** Add `fn list_dir(&self, path: &Path) ->
   Result<Vec<PathBuf>, FileIoError>` (or the shape that fits the trait's
   conventions — mirror `read_to_string`/`probe` signatures) to the
   `FileIo` trait (`runtime/file_io.rs`). It returns the immediate entries
   of one directory (one level; recursion for depth is the kernel's job,
   see below). Implement it in `StdFileIo`
   (`rum-host/src/file_io.rs`) via `std::fs::read_dir` (std::fs is
   allowed THERE — the host provider is the seam; the KERNEL must not call
   std::fs). Errors map to `FileIoError` like the existing methods.
2. **The builtin.** Add `eval_file_names` in
   `builtins_context_loader.rs`, gated FIRST by
   `ensure_capability_allowed("FileNames", Capability::FileRead)` (the
   brief-225 guard) THEN `has_file_io()` (emit the no-file-io message +
   `$Failed` if absent, like `eval_get`). Folk call forms (mirror the
   archived shape):
   - `FileNames[]` → pattern `"*"`, dir `"."`, depth 1
   - `FileNames[pattern]` → dir `"."`, depth 1
   - `FileNames[pattern, dirs]` → depth 1
   - `FileNames[pattern, dirs, depth]` → depth as given (`Infinity` allowed)
   Match entries against the pattern with the EXISTING wildcard matcher
   (the one `Names`/`Clear` use — cite it; do NOT hand-roll or use a glob
   crate). Recurse into subdirectories for depth > 1 by calling
   `list_dir` per level (the kernel does the walk; the provider lists one
   level). Sort + dedup the results; return a `List` of path strings. A
   malformed arg returns an inert call (the archived shape); a non-
   existent directory yields no matches (not an error — folk lenient,
   like the archived version).
3. **Registration.** Register `FileNames` in `builtin_catalog/namespace.rs`
   next to `FindFile`/`Get`, `.protected().with_capability(Capability::
   FileRead)` (mirror `FindFile` exactly — same file, same capability).
4. **Reference + docgen.** Regenerate `docs/reference/builtins.md`; add a
   folk-scope Limits note (ASCII path handling, no symlink-loop
   protection beyond the depth cap, provider-mediated). MERGE if a note
   exists.

## Bounds + safety (folk scope, but no foot-guns)

- The path-math siblings (`FileNameJoin`, `DirectoryName`,
  `ParentDirectory`, `FileNameTake`) already exist in the current tree —
  do NOT touch them; FileNames is the only removed head being restored.
- Depth recursion must be bounded: cap the total entries collected (a
  large finite ceiling, e.g. reuse whatever element-limit convention the
  codebase has, or a fixed cap — state it) so `FileNames["*", {"/"},
  Infinity]` cannot walk an entire filesystem unboundedly. A hit ceiling
  returns the bounded results (folk truncation), not a hang.
- Deterministic ordering: sort the results so output is reproducible.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes (use /tmp scratch dirs/files ONLY):
#  - mkdir /tmp/fn230 with a.wl b.wl c.txt; FileNames["*.wl", {"/tmp/fn230"}]
#    -> {"/tmp/fn230/a.wl", "/tmp/fn230/b.wl"} (sorted)
#  - FileNames["*", {"/tmp/fn230"}] -> all three
#  - a rum-host test: sandboxed() (no provider) -> FileNames returns the
#    no-file-io/denied shape, NOT a directory listing
#  - depth recursion on a nested /tmp dir
git status --short            # only MAY-listed files
```

## Perf gate + acceptance

- **No microbench A/B** — FileNames is a new leaf builtin, not on any hot
  path; state it.
- **mma-hol Strict gate at MY gate** — this is a FileIo-provider-seam +
  loader change; the director runs the full acceptance suite. (Bonus: the
  suite's `run_all.wls` FileNames discovery idiom, which silently no-opped
  and forced the explicit-list patch, now has a real implementation — but
  the director keeps the explicit-list patch for determinism; do NOT
  depend on FileNames in the gate procedure.) Note it; do not run it
  yourself.

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/file_io.rs` (the `list_dir` trait method)
- `crates/rum-host/src/file_io.rs` (StdFileIo impl via std::fs)
- `crates/rum-runtime/src/runtime/builtins_context_loader.rs`
  (`eval_file_names`) + any small helper module it needs (name it)
- `crates/rum-runtime/src/runtime/builtin_catalog/namespace.rs` (register)
- `crates/rum-runtime/src/runtime/capabilities.rs` (the test that maps
  names→capabilities, add FileNames→FileRead)
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
- Tests: runtime loader tests + rum-host provider/capability tests; list
  each; one new test file allowed

MUST NOT touch: the existing path-math heads, `Get`/`Needs`/`FindFile`
logic (only add alongside), the dispatcher, the `sandboxed()` no-provider
default, any std::fs in the KERNEL (provider only), anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies
  (no glob crate — use the in-tree wildcard matcher), no new `unsafe`, no
  std::fs in kernel code (rum-runtime); std::fs lives ONLY in the
  rum-host StdFileIo provider.
- FileRead-gated: `sandboxed()` denies it (no provider); unrestricted CLI
  allows it. Baseline-verify both.
- Folk scope: no symlink-cycle chasing beyond the depth/entry cap, ASCII
  paths, deterministic sorted output.
- Stop-loss: if `list_dir` cannot be added to the trait without breaking
  an existing provider impl, or the wildcard matcher cannot be reused for
  filename patterns, STOP and report.

## Report format

1. The change: the provider method, the builtin, the capability gate, the
   entry/depth bound, the reused matcher.
2. Probe panel: the /tmp glob cases; sandboxed() denial; depth recursion.
3. Test survey (new tests).
4. Verification outputs verbatim; the no-A/B justification + a note the
   mma-hol Strict gate is the director's.
5. Deviations / open questions (empty if none).
