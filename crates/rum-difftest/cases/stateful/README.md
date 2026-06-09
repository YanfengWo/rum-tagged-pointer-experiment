# Stateful Session Cases

Lane B fixtures live here. Each `.txt` file is one ordered session using the
same first-pass line format as Lane A:

- `input ||| expected_output`
- `input !!! id=name | type=... | reason=...`

The runner keeps one `rum` session for all steps in a file. The Wolfram oracle
path evaluates the same ordered steps as fixture batches in one local kernel
process per Lane B run, with cleanup between fixture batches, so definitions,
options, messages, attributes, and precedence checks run against real session
state instead of replayed setup expressions. Lane B compares values, per-step
messages, and per-step print output.

Fixture-local package files live under `packages/`. They are support files for
explicit `Needs[..., File[...]]` / `Get[...]` cases and are not collected as
Lane B fixtures.

Known-difference lines can be used for discovered long-tail divergences that
should stay documented without breaking the default lane. The current fixture
has no active known-difference lines.
