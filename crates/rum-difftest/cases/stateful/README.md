# Stateful Session Cases

Lane B fixtures live here. Each `.txt` file is one ordered session using the
same first-pass line format as Lane A:

- `input ||| expected_output`
- `input !!! id=name | type=... | reason=...`

The runner keeps one `rum` session for all steps in a file. The Wolfram oracle
path evaluates the same ordered steps in one local kernel process for that
fixture, so definitions, options, messages, attributes, and precedence checks
run against real session state instead of replayed setup expressions. Lane B
compares values, per-step messages, and per-step print output.

Known-difference lines can be used for discovered long-tail divergences that
should stay documented without breaking the default lane. The current fixture
uses that for exact `$MessageList` `HoldForm` shape, one definition precedence
edge, and one indexed default-value lookup edge.
