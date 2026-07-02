# Issue: `rum-host` needs a sandbox profile — default builtin catalog exposes the filesystem

**Filed**: 2026-07-02, from the TTT project (Tauri Typst editor showcasing rum as its
untrusted-plugin sandbox). Verified against this tree at `162ff90` (main).
**Requested by**: TTT director review (AUDIT.md §4 invariant check).
**Priority**: not v0.1-blocking for rum itself; **blocking for TTT P2** (plugin host).
Sequencing per the owner: land after the current pre-release wrap-up.

## Problem

`rum-host` positions itself as the embedding facade for hosts that run **untrusted**
code. Its hardening surface (resource limits, cancel, reset, poison boundary, structured
`HostValue` I/O) is solid — but the default builtin catalog gives evaluated code direct
filesystem access with the host process's full authority:

- **Reads**: `Get` (reads a file and *evaluates* it), `Needs`, `FindFile`, `FileNames`
  (directory listing), file-template loading
  (`rum-runtime/src/runtime/builtins_context_loader.rs`,
  `.../builtins_context_loader/file_names.rs`, `.../strings/template.rs`).
- **Writes**: `Save` / `DumpSave` — kernel-state persistence straight to disk via
  `fs::write` / append `OpenOptions` (`rum-runtime/src/runtime/package_tools.rs:81-85`,
  `write_saved_content` at `package_tools.rs:520-536`); file-template rendering also
  writes an output file (`rum-runtime/src/runtime/strings/template.rs:101`).

Two aggravating factors:

1. **`package_search_paths` is not a boundary**: `load_target_candidates`
   (`rum-runtime/src/runtime/resolution.rs`) pushes an absolute spec directly into the
   candidate list, so an empty/scoped search path does not stop
   `Get["/etc/passwd"]` or `DumpSave["/anywhere/file", ...]`.
2. **No exclusion mechanism**: `HostConfig::install_core_builtins` is all-or-nothing and
   the builtin catalog (`rum-runtime/src/runtime/builtin_catalog/namespace.rs`) is a flat
   list — a host cannot install "the language minus the fs group", and there is no
   host-side API to remove or stub individual builtins after install.

Net effect: a default `HostSession` running untrusted code can read, execute-from, and
write arbitrary paths. The in-memory `EvaluationRollbackSnapshot`/`LoaderSnapshot`
rollback machinery is fine (never touches the fs) — the gap is only the builtins above.

## Proposed fix

1. **Tag builtins with a capability group** in the catalog (e.g. `pure`, `fs-read`,
   `fs-write`), so installation can filter by group. Make the group a **required** field
   on `BuiltinSpec` — default-deny, so a future fs-touching builtin cannot silently leak
   into sandboxed hosts by omission.
2. **Add a sandbox profile to `rum-host`** — e.g. `HostConfig::sandboxed()` (or a
   `BuiltinProfile` enum on the config) that installs the catalog minus the
   loader/persistence/template-file group. Symbols should be *absent* (or fail cleanly
   with a message + `$Failed`), never silently no-op.
3. **Adversarial tests** in `rum-host`: under the sandboxed profile, `Get`, `Needs`,
   `FindFile`, `FileNames`, `Save`, `DumpSave`, and template-to-file are all dead,
   including absolute-path attempts; plus a regression test that every catalog entry
   declares a group.
4. **Non-goal**: the default/trusted profiles (CLI, REPL, Jupyter) keep the full catalog —
   this is purely an opt-in host profile.

## Consumer context (why TTT needs this)

TTT's whole premise is "editor plugins anyone can write in Wolfram Language, running in
rum, that *by construction* cannot hurt the user". Its security audit invariant is: *an
untrusted plugin cannot do anything except return a value from a resource-limited rum
evaluation.* With the default catalog that invariant is false today. TTT will keep a
belt-and-suspenders seal on its own side plus an adversarial fs-escape suite regardless,
but the allowlist-style profile belongs upstream where the catalog lives. TTT's P2
(plugin host) is gated on this issue.
