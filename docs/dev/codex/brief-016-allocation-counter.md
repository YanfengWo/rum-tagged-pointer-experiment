# Brief 016 — B0: gated allocation counter (Layer B measurement scaffold)

## Goal

Layer B (`docs/layer-b-node-rewrite-plan.md`) fixes flaw 2 — the
multi-allocation call node. Its win is primarily in **live memory + locality**
(`Arc<Vec<Expr>>` is an `Arc` box plus a separate `Vec` buffer with a 24-byte
`Vec` struct of indirection; `Arc<[Expr]>` is one allocation with the elements
inline), not necessarily in malloc COUNT. The existing `RUM_PROFILE_REPR`
counters count `SharedArgs::new` calls but cannot see heap bytes or live memory.
Add a direct allocation counter so B1 has an objective before/after measure.

Provide an opt-in global-allocator shim that, when enabled, counts: total
allocation count, total bytes allocated, and **peak live bytes** (the key Layer B
metric). When disabled it must be effectively zero-cost (a single relaxed atomic
load + branch on the alloc path, negligible next to the malloc itself — but
prove there is no measurable regression).

## Expected, measurable result (the gate)

- With the env flag OFF: the microbench tier is unchanged versus before this
  brief (clean back-to-back A/B; the allocator path adds at most one relaxed
  atomic load + branch, which must not move the tier).
- With the flag ON: a known workload prints a sane allocation summary, e.g.
  `Do[Do[fExact[1],{i,1,100000}],{j,1,10}]` reports a non-trivial alloc count and
  byte total, and holding a large structure (e.g. `x = Range[200000];` kept live)
  reports a peak-live-bytes that scales with the structure.
- `cargo test -p rum-cli` (and workspace) green.

## Context pointers

- `crates/rum-cli/src/main.rs:5,12-13`: `use mimalloc::MiMalloc;` and
  `#[global_allocator] static GLOBAL_ALLOCATOR: MiMalloc = MiMalloc;`. The shim
  wraps this `MiMalloc`.
- The cold-load `AtomicU8` env-gate pattern to copy is in
  `crates/rum-core/src/profiling.rs` (`REPR_PROFILE_ENV`, the enabled-flag init).
  Use the same shape for the new `RUM_PROFILE_ALLOC` flag.
- `GlobalAlloc` requires `alloc`, `dealloc`; also override `alloc_zeroed` and
  `realloc` (delegate to the inner `MiMalloc` and account the size delta) so the
  byte accounting is correct.

## Design (implement; idiomatic Rust is yours)

1. In `crates/rum-cli/src/main.rs` (or a new small `crates/rum-cli/src/alloc_count.rs`
   module): define `struct CountingAlloc(MiMalloc);` implementing `GlobalAlloc`:
   - `alloc(layout)`: `let p = self.0.alloc(layout);` if `!p.is_null()` and the
     flag is on, `ALLOC_COUNT += 1; BYTES_TOTAL += layout.size(); LIVE += size;
     update PEAK_LIVE = max(PEAK_LIVE, LIVE)` (all `Relaxed` atomics). Return `p`.
   - `dealloc(ptr, layout)`: if flag on, `LIVE -= layout.size()` (saturating);
     then `self.0.dealloc(ptr, layout)`.
   - `alloc_zeroed` / `realloc`: delegate to inner, account the size change
     (realloc: `LIVE += new - old`, `BYTES_TOTAL += new` on grow; keep it simple
     and correct, not perfectly precise on shrink).
   - The flag is a single `AtomicU8` loaded via a `#[inline]` helper, init-once
     from `RUM_PROFILE_ALLOC` (non-empty, != "0"), same cold pattern as
     `profiling.rs`. When the flag is off, the only added work is that load + a
     branch.
   - `#[global_allocator] static GLOBAL_ALLOCATOR: CountingAlloc =
     CountingAlloc(MiMalloc);`.
2. Counters: `static ALLOC_COUNT/BYTES_TOTAL/LIVE/PEAK_LIVE: AtomicU64` (or
   `AtomicUsize`) in the same module.
3. Reporting: when the flag is on, print a one-line summary at the end of `main`
   (after the eval completes), e.g.
   `rum-profile alloc count=<n> bytes_total=<b> peak_live_bytes=<p>`. Reset is
   not needed (whole-process totals diff cleanly across runs since startup is
   constant). Guard the print so it only fires when `RUM_PROFILE_ALLOC` is set.

**Pitfall:** the gate flag init must NOT itself allocate inside the allocator
(no `String`/env lookup on the alloc path). Read the env ONCE into the `AtomicU8`
from `main` before the workload (e.g. an explicit `alloc_count::init()` call at
the top of `main`), and on the alloc path only do the atomic load — never touch
env or allocate. A re-entrant allocation inside `alloc` would deadlock/recurse.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-cli
cargo test -p rum-runtime
```

Green. Then the summary works:

```
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'Do[Do[fExact[1],{i,1,100000}],{j,1,10}]; fExact[1]=2; 1' 2>&1 | grep 'rum-profile alloc'
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'x = Range[200000]; Length[x]' 2>&1 | grep 'rum-profile alloc'
```

Report both lines (count, bytes_total, peak_live_bytes). Then prove zero-cost
when OFF with a clean back-to-back A/B (the change is uncommitted — stash it for
the baseline half), microbench tier, and confirm the tier does not regress:

```
bash scripts/core-language-microbench.sh --engine both --output-dir <dir>
```

Paste `if-true`, `downvalue-exact-hit`, `sameq-integer`, `length-list` for both
halves (baseline = stashed, new = this change). They must be equal within noise.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree (stash-pop to restore
  after the A/B baseline half).
- Files: `crates/rum-cli/src/main.rs` and an optional new
  `crates/rum-cli/src/alloc_count.rs`. Nothing else. No `mma-hol` edits. Remove
  any `.rum_history`.
- The allocator path must not allocate or touch env; flag is a pre-initialized
  atomic load. A re-entrant alloc is a hard bug — if tests hang, that is it.
- Stop-loss: same test failure twice, OR the OFF tier regresses measurably → STOP
  and report.
- Minimal diff. Paste real test summaries, the two alloc lines, and the A/B rows.

## Report format

1. Per-file changes, line ranges + one-line why.
2. Verification — verbatim `cargo fmt --check`, the test summaries, the two
   `rum-profile alloc` lines, and the OFF-path A/B rows (both halves).
3. Stop-loss / open questions (empty if none).
