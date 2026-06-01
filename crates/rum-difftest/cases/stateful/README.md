# Stateful Session Cases

Lane B fixtures live here. Each `.txt` file is one ordered session using the
same first-pass line format as Lane A:

- `input ||| expected_output`
- `input !!! id=name | type=... | reason=...`

The runner keeps one `rum` session for all steps in a file. The current
first-pass Wolfram oracle path evaluates each step by replaying prior inputs in
a compound expression before the current input. That is enough for basic
definition, option, message, and precedence conformance checks, but it is not
the final interactive-oracle transport for every possible session edge.
