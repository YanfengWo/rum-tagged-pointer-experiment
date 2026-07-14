# Brief 228 — Interactive-input capability for Input / InputString / Dialog

Fix brief — unit-10 decision fix 1 (USER-RULED 2026-07-14), capability
hygiene. Source: `docs/dev/usability-pass/kernel-c-findings.md` (the
Input-family capability-label row) + the `Exit` precedent (brief-183,
which moved `Exit` from `Pure` to `Process` by the same reasoning). This
is an API-hygiene fix in the same family as the FS-purity line: a builtin
whose declared capability does not match what it actually depends on gets
moved to the right single-purpose bucket.

## The inconsistency

`Input`, `InputString`, and `Dialog` are registered with the default
capability `Pure` (`builtin_catalog/scoping.rs:97-112`, via `ordinary` /
`ordinary_with_hold`). But their behavior is to SUSPEND the evaluation
and request a value from the host's interactive input channel (an
`InputRequired`-class suspension). Depending on an interactive
surrounding is exactly what `Pure` says a builtin does NOT do — the same
mismatch the `Exit` row had (declared `Pure` while requesting host
lifecycle control).

rum's established doctrine (the FS-purity line): each dependency on the
outside world is a SINGLE-PURPOSE, spec-level capability —
`Clock` for the wall clock, `Random` for host entropy, `FileRead`/
`FileWrite` for the filesystem, `Process` for lifecycle. Interactive host
input is a distinct ambient authority with no bucket today.

## Required change

- **Add one capability variant** `Capability::Interactive`
  (`runtime/capabilities.rs`), for "may request interactive input from
  the host / may suspend for host input". Add its bit to
  `Capability::bit()` — bit 7 (`1 << 7`); the mask is `u8` and bits 0-6
  are taken, so bit 7 fits without a type change (confirm and state it).
- **Relabel the three heads** to `Capability::Interactive` via
  `.with_capability(...)` on their `BuiltinSpec` in `scoping.rs` (mirror
  how `debug.rs` sets `Capability::Clock` and `namespace.rs` sets
  `Capability::FileRead`). Update their registration descriptions to name
  the capability (as the Clock/FileRead heads' descriptions do).
- **Ripple the exhaustive matches**: `crates/rum-stdlib/src/docgen.rs:396`
  (`match capability` → add `Capability::Interactive => "Interactive"`).
  Grep for EVERY exhaustive match on `Capability` across the workspace and
  cover each — name them all in the report (the known two are `bit()` and
  docgen; find any others, e.g. a host-side mapping).
- **Capability policy behavior** (the point of the fix):
  - the unrestricted (trusted CLI / default `HostConfig`) policy still
    allows `Interactive`, so `Input`/`InputString`/`Dialog` work exactly
    as today in the interactive REPL and trusted embeddings — BYTE-
    IDENTICAL. `HostConfig::default()` is unrestricted; confirm the trace.
  - `HostConfig::sandboxed()` allows only `Capability::Pure`
    (`rum-host/src/config.rs`), so after the relabel it DENIES the three
    heads: a sandboxed call to `Input[]` returns the capability-denied
    shape (`General::sandbox` + `$Failed`, the same shape FileRead heads
    return in the sandbox) INSTEAD of raising an interactive suspension.
    This is the requirement: a restricted profile cannot block on
    interactive input.
- Regenerate `docs/reference/builtins.md` (the three heads' Capability
  lines change from `Pure` to `Interactive`).

## Why this is the general-embeddable-language shape, not a dun special-case

Interactive input is a legitimate capability a TRUSTED embedding may want
(a REPL-like host, an interactive CLI). Capability-GATING it — rather than
removing it — means trusted embeddings keep it and restricted profiles
deny it, from one mechanism. This serves both the restricted-sandbox case
and the rich-trusted-embedding case; it does not narrow rum to any single
downstream. State this framing in the report.

## Method

1. Read brief-183 (the `Exit` → `Process` relabel) and the FS-purity
   capability additions — same shape, same ripple discipline.
2. Add the variant + bit; relabel the three heads; ripple the matches.
3. Baseline-verify: unrestricted `Input`/`InputString`/`Dialog` behavior
   is byte-identical (they still suspend for input under the trusted CLI —
   probe with a provided stdin line); `sandboxed()` now denies them.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# trusted CLI (unrestricted): Input still reads a provided line —
#   printf 'foo\n' | cargo run -q -p rum-cli -- --eval 'InputString["p> "]'
#   (byte-identical to pre-change)
# a rum-host test: sandboxed() now denies Input/InputString/Dialog with
#   the capability-denied shape (mirror the existing
#   sandboxed_host_config_blocks_filesystem_builtins_at_dispatch test —
#   add the three interactive heads)
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/capabilities.rs` (the variant + bit)
- `crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs` (the three
  `.with_capability` relabels + descriptions)
- `crates/rum-stdlib/src/docgen.rs` (the capability→string match) +
  `docs/reference/builtins.md` (regenerated)
- any OTHER exhaustive `Capability` match the grep finds (name it)
- Tests: `rum-host` capability tests + `capabilities.rs` unit tests +
  any runtime Input tests; list each file; one new test allowed

MUST NOT touch: the Input/InputString/Dialog IMPLEMENTATIONS (the
suspension mechanism is unchanged — only the capability label moves), the
evaluator core, the dispatcher's capability-check logic (it already gates
by capability — the relabel is all that's needed), other capabilities,
anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`.
- The unrestricted-policy behavior of all three heads is byte-identical;
  baseline-verify it.
- No A/B (the capability check is already on the dispatch path; adding a
  bucket does not add a hot-path branch — the check is the same
  `allows(spec.capability)` compare). No mma-hol gate (mma-hol runs
  unrestricted and never calls Input).
- Stop-loss: if adding the variant forces a `u8`→wider change or an
  unexpected exhaustive match that ripples beyond the named files, STOP
  and report the ripple before proceeding.

## Report format

1. The change: the variant + bit, the three relabels, every exhaustive
   match rippled (named).
2. Probe panel: unrestricted Input byte-identical; sandboxed() denies.
3. Test survey old→new.
4. Verification outputs verbatim; the no-A/B + no-mma-hol justification.
5. Deviations / open questions (empty if none).
