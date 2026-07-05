# FS Purity Plan — strip general file operations; kernel fs-free by construction

Status: ACTIVE (started 2026-07-06). Follow-up line to the release
transformation. Decision (user): general file operations leave the repository
entirely (archived privately for a future downstream `rum-fs` project, started
after rum ships); the module system and session persistence stay, with their
I/O injected by the host. No in-repo `rum-fs` crate.

## Disposition

KEEP in kernel (language semantics; I/O via a host-injected provider):
`Get`, `Needs`, `FindFile` (the module system and `$Path` resolution),
`Save`, `DumpSave` (session persistence). `$Context`/`$ContextPath`/`$Path`/
`$Packages` are pure state and unchanged. Capability classes (`FileRead`/
`FileWrite`) remain on these five — capability stays the runtime policy
layer; the provider is the construction-level layer.

REMOVE and archive (general fs surface, recycled later by downstream
`rum-fs`): `FileNames`, `FileExistsQ`, `FileDate`, `FileByteCount`, `Import`,
`FileTemplateApply`. Implementation today: the first five live in
`crates/rum-runtime/src/runtime/builtins_context_loader/file_names.rs`
(kernel-resident); `FileTemplateApply` in `crates/rum-stdlib/src/strings/`
(only the file-writing variant leaves; pure template evaluation stays).

## Steps

| Step | Owner | Content |
|---|---|---|
| M0 | Claude | This plan |
| brief-132 | Codex | Strip the six: archive verbatim copies (mirrored paths, MANIFEST entry flagged "future rum-fs"), remove implementations/registrations/capability arms/tests, regenerate the builtin reference, listed guide wording updates |
| brief-133 | Codex | Provider seam: a small read/probe/write trait in the kernel API; loader + persistence go through it; real `std::fs` provider lives in `rum-host` (default for trusted config; absent under `sandboxed()`); `rum-cli`/REPL behavior unchanged; hygiene test bans `std::fs` in non-test kernel-crate code |
| M1 | Claude | Final sweep, docs alignment (ARCHITECTURE/design-boundaries), TODO/memory, archive this plan + briefs |

Gates per brief: `cargo fmt --check`, `cargo test --workspace`, reference
drift test, behavior-preservation checks named in the brief. The loader is
not a per-eval hot path; the microbench A/B applies only if a brief touches
eval/dispatch paths.

## End state

- `rum-core`/`rum-runtime`/`rum-stdlib`: zero `std::fs` outside `#[cfg(test)]`,
  enforced by a hygiene test (same standing as the zero-`unsafe` line).
- Kernel fs surface = module system + persistence only, through the provider.
- Sandboxed hosts: capability denial unchanged (fires at dispatch first);
  provider absence is the second, construction-level layer. A future host can
  inject a virtual provider (in-memory packages) without any filesystem.
- Future `rum-fs` downstream repo recycles the archived six and owns all
  expansion (text I/O, directory ops, scoped roots, JSON/CSV). Network stays
  permanently out of the kernel; same pattern if ever wanted.
