# Stateful Session Cases

Lane B fixtures live here. Each `.txt` file is one ordered session using the
same first-pass line format as Lane A:

- `input ||| expected_output`
- `input !!! id=name | type=... | reason=...`

Current fixture focus:

- `definitions_messages.txt`: assignments, options, messages, attributes,
  precedence, context basics, and definition lifecycle
- `message_switches.txt`: ordered `Off` / `On`, `$MessageList`, `Check`, and
  `Quiet` message-stream behavior
- `package_defaults.txt`: file-backed package loading, `$Packages` /
  `$ContextPath` state, package-scoped defaults, unqualified symbol lookup, and
  repeated `Needs` no-reload behavior
- `options_state.txt`: ordered `SetOptions` default mutation plus
  `OptionsPattern[target]` option-value scope behavior
- `scoping_control.txt`: ordered `Block` / `Module` / `With` behavior and
  control-flow restoration across steps
- `zz_fixture_isolation.txt`: cleanup between fixture batches in the
  multi-batch oracle transport

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
