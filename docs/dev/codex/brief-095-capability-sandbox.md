# brief-095 — capability-scoped builtins: sandbox profile for rum-host (fixes the fs-exposure issue)

## Goal

Fix `ISSUE-sandbox-profile-fs-builtins.md`: rum-host's default catalog exposes the filesystem
to untrusted code. Implement an ELEGANT, future-proof capability system (design agreed with
the director), NOT ad-hoc per-builtin exclusion:

1. **`Capability`** — an extensible enum classifying what ambient authority a builtin needs.
2. **Every builtin declares its capability** — compile-enforced, DEFAULT-DENY (no silent-Pure
   fallback; a new builtin cannot ship unclassified, and an unclassified/unsafe one is never
   silently treated as Pure).
3. **`CapabilityPolicy`** — the set of allowed capabilities, held by the session; the CALLER
   sets it. Enforcement happens **at builtin DISPATCH** in the eval loop (airtight — catches
   runtime-constructed calls like `Symbol["Get"][...]` / `ToExpression`, which input-scanning
   filters miss).
4. rum provides the PARTITION + the POLICY KNOB + ENFORCEMENT only. **No role/RBAC/identity
   modelling** — that is the caller's concern (a caller maps its roles/plugins → a
   `CapabilityPolicy`). Keep rum orthogonal.

Product-code, spans rum-runtime + rum-host. No new `unsafe`. mma-hol byte-identical (default
policy is unrestricted → zero behavior change).

## Design

### Capability (rum-runtime; re-export from rum-host)
```
enum Capability { Pure, FileRead, FileWrite }   // extensible: add Network/Process/Parallel/… later
```
Only the variants that map to CURRENT builtins (do NOT add speculative variants — the design
must make adding one trivial: new variant + classify the new builtins + it's auto-excluded from
any allowlist that doesn't name it).

### Classification (compile-enforced, default-deny)
Give `BuiltinHandler` a `fn capability(&self) -> Capability` implemented with an **EXHAUSTIVE
match, NO wildcard `_` arm** — so the compiler FORCES every current and future handler to be
classified (adding a variant without an arm = non-exhaustive-match compile error). Group the
many `Pure` handlers with `|`. Do NOT use `_ => Pure` (that is the silent-leak the issue warns
about). The fs builtins to classify:
- **FileRead**: `Get` (reads + evaluates!), `Needs`, `FindFile`, `FileNames`, `FileExistsQ`,
  `FileByteCount`, `FileDate`, `Import`.
- **FileWrite**: `Save`, `DumpSave`, `FileTemplateApply` (renders to a file).
- **Pure**: everything else — INCLUDING the pure path-STRING manipulators `FileNameJoin`,
  `DirectoryName`, `ParentDirectory`, `FileNameTake` (they touch no fs). Audit each handler:
  if it does real `std::fs`/`std::net`/`std::process` I/O it is NOT Pure. (Cross-check the
  fs list against `git grep` for `std::fs` in the handlers to make sure none are missed.)

### CapabilityPolicy + config (rum-runtime + rum-host)
- `CapabilityPolicy`: an allowed-capability set with a fast **`unrestricted`** representation
  (default). Constructible from a set: `CapabilityPolicy::unrestricted()` (default, all),
  `CapabilityPolicy::allow([Capability::Pure, Capability::FileRead])`, builder `.allow()/.deny()`.
- Session holds the policy. `HostConfig`/`HostSessionBuilder` gets it:
  `HostConfig::sandboxed()` = policy `{Pure}` (the fix's headline); default = unrestricted.
  Keep `with_core_builtins`/existing API working (unrestricted).
- CLI / REPL / Jupyter: unchanged (unrestricted default).

### Enforcement (dispatch, airtight + ZERO-COST when unrestricted)
At the point the eval loop dispatches a builtin: if `policy` is unrestricted → run (NO check —
this must be a near-free branch so CLI/mma-hol hot path is unchanged; rum's no-default-cost
rule). Otherwise, if `handler.capability() ∉ policy` → do NOT run; emit a message
(e.g. `General::sandbox` or `<Name>::sandbox`, "… not permitted in this sandbox") and return
`$Failed`. Enforce at dispatch (not by removing the symbol) so `Symbol["Get"][...]` /
`ToExpression["Get[..]"]` are ALSO blocked.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test -p rum-host               # incl. new sandbox tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical (default policy unrestricted)
```
Adversarial sandbox tests (rum-host, under `HostConfig::sandboxed()`): `Get`/`Needs`/`FindFile`/
`FileNames`/`FileExistsQ`/`Import`/`Save`/`DumpSave`/`FileTemplateApply` all → `$Failed` (+
message), INCLUDING absolute-path attempts AND runtime-constructed forms (`Symbol["Get"]["x"]`,
`ToExpression["Get[\"/etc/passwd\"]"]`); `Pure` work (arithmetic, list ops, the path-STRING
helpers) still runs; a fine-grained policy (`{Pure, FileRead}`) allows `Get` but still blocks
`Save`. Plus: the trusted/default profile runs the full catalog unchanged. Hot-path check:
confirm the unrestricted path adds no measurable dispatch cost (a quick before/after on a
matching/eval microbench, or argue the branch is a single `is_unrestricted` check hoistable
out of the hot loop).

## Hard rules

- No new `unsafe`. No `Expr`/serde change. DEFAULT-DENY classification (exhaustive match, no
  `_ => Pure`). Zero default-path cost (unrestricted policy must not slow the dispatch hot
  path). No role/identity modelling (caller's job).
- No `git commit`. Never edit any `mma-hol` file.
- Absolute no-regression: workspace + mma-hol byte-identical (default unrestricted). Enforcement
  is airtight at dispatch (runtime-constructed calls blocked).
- Paste REAL verbatim output; never fake a green.

## Report format

1. Design as built — the `Capability` enum, the exhaustive `capability()` classification
   (confirm NO wildcard; list the non-Pure arms), `CapabilityPolicy` + `HostConfig::sandboxed()`,
   the dispatch enforcement point.
2. Verification — the 5 gates + the adversarial sandbox tests (incl. absolute-path +
   runtime-constructed) + the hot-path/no-default-cost evidence, all verbatim.
3. Confirm: default profile byte-identical to before; CLI/REPL/Jupyter unaffected; no
   role/identity logic added.
4. Anything deferred (e.g. scoped-fs for a trusted-but-limited profile) noted for follow-up.

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. Capability sandbox landed; fs-exposure issue RESOLVED.

Committed. New `capabilities.rs` (Capability enum + CapabilityPolicy) + rum-runtime dispatch
enforcement + rum-host `HostConfig::sandboxed()`/policy setters + re-exports. Gate (Claude
reproduced):
- Classification is an EXHAUSTIVE match with NO `_ => Pure` wildcard (verified) — compile-
  enforced default-deny. All fs builtins classified fs-*: Get/Needs/FindFile/FileNames/
  FileExistsQ/FileByteCount/FileDate/Import = FileRead; Save/DumpSave/FileTemplateApply =
  FileWrite; ParentDirectory made string-only → Pure. Cross-checked std::fs handler files map
  to classified builtins (no missed fs surface).
- `cargo test -p rum-runtime` 784/0/2; `cargo test -p rum-host` 24/4/12 (adversarial sandbox
  tests: absolute-path + runtime-constructed `Symbol["Get"]`/`ToExpression` blocked; Pure runs;
  `{Pure,FileRead}` allows Get blocks Save); `--workspace` no failures; `mma_hol_cold_lane` 4/0
  byte-identical; fmt clean. Default unrestricted unchanged (CLI Get/ParentDirectory work).
- Hot-path A/B (release, ×10, 5 reps): 0.97–1.04x — zero default-path cost (unrestricted fast
  path). No unsafe, no Expr/serde change, no role/identity logic.

fs-exposure issue RESOLVED (ISSUE-sandbox-profile-fs-builtins.md marked resolved); TTT P2
unblocked; the v0.1 doc-honesty caveat is now MOOT — the host-embedding docs can positively
document `HostConfig::sandboxed()`. Deferred: scoped-fs roots for a trusted-but-limited profile.
