# Same-binary rewrite switch diagnostic

This patch is an isolated causal diagnostic, not a production integration
proposal. Apply it to a fresh pinned source copy without the previous matcher,
depth, wide-copy, Flat or ledger overlays. Keep the already measured
`rewrite-shortcuts` sources and evidence unchanged. The patch includes both
functional shortcuts and a new safe runtime module; no separate module file
must be copied from this directory.

## One immutable executable, four launch modes

Build the release executable once. Use that exact file, SHA-256, target,
allocator and toolchain for every process. Set only
`RUM_REWRITE_SHORTCUTS_MODE` to `base`, `matcher`, `depth` or `combined` in
the child process environment. An absent variable selects `base`. An empty,
unknown, differently capitalized or non-Unicode value fails during Session
construction instead of silently selecting another mode.

`Session::empty` initializes a process-wide `OnceLock<u8>` before the Session
can evaluate anything. Environment parsing happens once, outside evaluation
timings. Each hot site reads the cached value through `OnceLock::get` and
checks one bit. There is no environment lookup, allocation, mutex or
initialization closure on the initialized hot path. Safe Rust remains the
only implementation language.

The mode is immutable for the lifetime of the process and shared by all
Sessions and threads. A later environment change does not change it. Run
each mode in a fresh process; never call `set_var` from Rust tests or try to
reset the lock. Environment mutation is unsafe in Rust 2024, and changing a
mode during evaluation or suspension is outside this design.

## Patch boundaries and common overhead

The input-depth site in `replace_owned` performs its cached mode query on
every entry. It substitutes one for selection only if the depth bit is set
and `levels.min >= 0`. Every specification with `min < 0` and every returned
outcome depth follows the existing implementation.

The matcher site performs its cached mode query after the existing prefilter
accepts a rule. The source constructs Blank/Named shape classification before the enabled check; the compiler may move this pure calculation into a selected branch.
Only an enabled matcher bit, empty guards, and `Blank` or
`Named(inner Blank)` allow the existing safe definition matcher. Unsupported
patterns retain the original matcher and suspension path. RHS substitution,
trace, cursor progression and later RHS evaluation are unchanged.

Consequently `base` here is a modified experimental control:
it pays both cached queries, bit/branch checks and the shared matcher
source eligibility calculation. Actual classification cost can differ after compiler optimization. `OnceLock::get` still checks initialization; branch
prediction and instruction costs can differ between modes. The same-binary
ratio estimates the effect of selecting the shortcut under this common
executable. It is not the exact ratio against an untouched original build,
and shared overhead cannot be subtracted as a constant. Keep an independently
measured untouched-original comparison as the production performance gate.

The alternative of a thread-local `Cell<u8>` removes a shared lock-state
read but adds TLS addressing and per-thread initialization concerns. Passing
bits through Session fields affects layout and snapshot/clone handling.
Neither is necessary for this first causal diagnostic. Keep the OnceLock
implementation fixed across all four modes.

## What is held fixed, and what is not

All modes use identical machine-code bytes, function ordering and offsets
within the executable. Both functional branches exist in every process.
Mode-dependent compilation, inlining and executable layout are thus removed
from the within-binary comparison. Separate processes with ASLR enabled can
still map those bytes at different absolute addresses; do not call their
absolute addresses identical. Record the ASLR policy. An optional second
lane may disable ASLR uniformly if the host permits it; refusal is recorded,
not worked around by altering the system policy.

Literal same-address mode switching inside one process would require a
different mutable control and introduce allocator/history/carryover issues.
It is unnecessary for the first experiment. Do not add a language-visible
mode-changing builtin or claim startup environment selection proves identical
virtual addresses.

If the same-binary focus gains remain while unrelated regressions disappear,
that supports a functional contribution plus a contribution from separately
compiled code differences. It does not uniquely prove address alignment as
the cause; compiler optimization, cache state and branch behavior remain
possible mechanisms. A disappearing gain likewise does not prove that the
original shortcut was useless: the combined executable and its shared switch
cost can change optimization opportunities.

## Required checks and measurement protocol

The new module has four independent process modes. With the previous 15 Stage 32
regression tests installed uniformly, the full workspace total remains 2256
passed, 7 ignored, 29 groups if the source pin and fixture set are unchanged.
Mode selection is checked in independent processes; no in-process environment mutation is used.

For each mode in a fresh process, run the same full value/alias checks,
guard/RHS-order cases, repeated bindings, unsupported pattern fallbacks,
negative and mixed levels, trace, capability denial, recursion limits and
queued versus suspended input tests. Compare unset mode with explicit `base`;
check invalid modes fail before evaluation. Verify a new Session in the same
process keeps the already selected mode. Threaded construction must select
one consistent mode without deadlock. The eight launch checks cover unset, all four valid names, empty, unknown and a leading-space name. Non-Unicode input, case variants, repeated Session construction and concurrent initialization have not received dedicated runtime checks; these remain coverage limits.

For timing, use the unchanged 112-label suite and the same focus/pipeline
inputs, scale, repetition count and warm-up policy as Stage 32. Initialize
the Session before any measured operation. Retain byte-identical A/A controls,
balanced serial process orders, fixed CPU/environment, all slower labels and
session-state observations. Do not build, upload or run ledger probes during
timing. Both positive and negative depth cases remain controls. A separate
counter build may verify the intended logical work; do not time it or mix
its bytes with this single-executable experiment.
