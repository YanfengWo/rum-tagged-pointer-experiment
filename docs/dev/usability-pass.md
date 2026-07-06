# Usability Pass — tracker

Status: ACTIVE (opened 2026-07-07). Mainline after the breadth lines
(stdlib expansion, codec family, hardening coverage, A+B groups) closed.

The "works" layer is in; this pass raises the "works well" layer. The
`RandomPassword` arc (briefs 157-159) is the worked example: "works"
first (157), then flexibility (158), convenience + safety (159).

## Directives (user, 2026-07-07)

- **High coverage.** Every installed builtin is swept — the WL-inherited
  surface included, not only rum-native additions. 605 installed at open:
  25 stdlib catalog families = 445, kernel surface = 160.
- **Compatible enhancement.** Where a WL-inherited builtin can be
  improved, improve it WITHOUT breaking WL shape: default behavior stays
  WL-conformant; new powers arrive as new options (or as rum-native
  sibling functions). Never change what an existing WL call form
  returns.

## The three axes (review lenses)

- **Flexibility** — can it meet complex real-world needs? Options where
  WL has them; operator/curried forms; escape hatches for the
  non-default case; composability with the rest of the library.
- **Convenience** — can a user turn intent into code easily? Named
  constants over magic literals; sensible defaults; readable option
  surfaces; the common case short.
- **Safety** — does the design prevent foot-guns? `Protected` where
  constant-like; fail-closed on malformed input (message + `$Failed`,
  never a silent wrong answer); capability gating; determinism where
  surprising non-determinism would bite.

## Method

1. **Audit briefs** sweep one unit each and write a findings file
   `docs/dev/usability-pass/<unit>-findings.md`: one row per builtin —
   including "OK" rows; the all-rows table IS the coverage proof — and
   every claimed gap backed by a probe transcript or a code citation.
   Audit briefs change no behavior.
2. Claude triages findings into **fix briefs** (the 158/159 shape:
   small, axis-labeled, compat-checked).
3. Standing guards: folk scope (no CAS creep — heavy inputs get a
   message/fallback, not an algorithm arms race); testing policy (WL
   documentation knowledge may inform design; no external-engine runs);
   the feature-gating policy for anything dependency-adjacent.

## Sweep units

| # | Unit | Count | Audit | Fixes |
|---|------|-------|-------|-------|
| 1 | strings | 99 | brief-161 | |
| 2 | structure + functional | 89 | | |
| 3 | predicates + associations | 83 | | |
| 4 | control + logic + rewrite | 63 | | |
| 5 | arithmetic + number_theory + combinatorics + integer_digits + polynomials + calculus | 62 | | |
| 6 | codecs (json/csv/ini/toml/xml/yaml/schema) + bytes + datetime | 34 | | |
| 7 | character_classes + random (light re-check; 157-159 fresh) | 15 | | |
| 8 | kernel A: definitions/values/attributes/protection | ~50 | | |
| 9 | kernel B: scoping/eval control/patterns/forms | ~55 | | |
| 10 | kernel C: system state/contexts/packages/paths/diagnostics/tests | ~55 | | |

Counts for units 1-7 are exact (catalog arrays, 445 total); units 8-10
split the 160-name kernel list (installed reference minus stdlib
catalog) and are enumerated exactly when their audit brief is written.
