# Ham Differential Test Seed Pack

Imported from:

- `side-mission/ham-review/ham-difftest`

Import date:

- `2026-03-28`

Contents:

- `golden/`
  - `arithmetic.txt` (`34` parseable entries including `1` known diff)
  - `control.txt` (`31` parseable entries)
  - `definitions.txt` (`8` parseable entries including `1` known diff)
  - `evaluation_control.txt` (`4` parseable entries including `1` known diff)
  - `lists.txt` (`38` parseable entries)
  - `ordering.txt` (`12` parseable entries)
  - `patterns.txt` (`17` parseable entries)
  - `scoping.txt` (`10` parseable entries)
  - `strings.txt` (`15` parseable entries)
- `bench/timing_suite.txt` (`20` expressions)
- `mem/memory_suite.txt` (`13` expressions)
- `regen.wls`
- `KNOWN_DIFFS.md`

Totals:

- `166` normal golden cases
- `3` known-difference cases
- `169` correctness entries overall

Counting note:

- these totals reflect parseable fixture entries only
- the original staging notes overcounted comment-header lines such as
  `# Format: input ||| expected_output`

Important caveats:

- These files reflect `ham`'s current output formatting conventions, not
  necessarily `rum`'s.
- Coverage is strongest for arithmetic, lists, control, patterns, ordering,
  basic scoping, and basic strings.
- Coverage is weak or absent for:
  - host-facing API behavior
  - adapter protocols
  - associations
  - richer functional/programming surfaces
  - package-loading workflows
  - multi-step session semantics beyond single-line `;` chains
- The imported `KNOWN_DIFFS.md` describes `ham` mismatches, not `rum`
  mismatches. It is useful as precedent and as a source of candidate oracle
  cases, but it is not a `rum` divergence document.
