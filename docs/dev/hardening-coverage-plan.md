# Hardening Coverage Plan — host lifecycle, limit paths, stress corpus

Status: ACTIVE (started 2026-07-07). The infrastructure half (A-half) of
the coverage-rebuild debt that has stood since the external acceptance
corpus was removed. Scope: kernel/host contract only — stdlib BREADTH
coverage is explicitly deferred until stdlib growth plateaus (the batch
discipline already ships coverage-axes tests with every addition).

## Why now

Three lines (fs-purity, stdlib expansion, codec family) landed on the same
thinned safety net, and each added lifecycle surface of its own: FileIo and
EntropySource providers with their noio/noentropy paths, native_stack_limit,
dual-form capability checks, session PRNG state. rum-host is what dun and
the pyo3 binding will consume; its coverage is a prerequisite for downstream
integration.

## Steps

| Brief | Content |
|---|---|
| 149 (diagnostic) | cargo-llvm-cov baseline (0.6.21, pinned toolchain): workspace summary + per-file tables for rum-host and the rum-runtime lifecycle/limit surfaces; a verified list of uncovered-or-thin functions and untested `TerminatedEvaluation` tag paths; ranked hole list |
| 150 | Host-lifecycle test buildout filling the measured holes (cancel/timeout/poison/reset/snapshot/capability/provider seams/Send contract), measured delta re-run |
| 151 | rum-native stress corpus: deep recursion near the guards, large rewriting workloads, cancellation and timeout under load, guarded by scripts/guarded-test-run.sh; slow tests marked `#[ignore]` with a documented invocation |
| M1 | Closure: re-measured baseline recorded, TODO/memory, archive, bundle |

Gates: fmt + workspace green as always; 149 is evidence-only; 150/151 must
show the before/after coverage delta on the targeted files (same tool, same
flags). Coverage numbers are DIAGNOSTIC — the gate is the measured hole
list shrinking, not a vanity percentage target.

## Out of scope

Stdlib breadth coverage (deferred to the plateau); fuzzing (parked as a
future candidate); CI wiring (no remote).
