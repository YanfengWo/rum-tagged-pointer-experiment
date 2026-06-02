# Divergence Ledger

Lane C data lives here.

Known differences should be tracked as checked fixture data, not buried in
comments or silently accepted in runner logic.

The active ledger is [known-differences.txt](./known-differences.txt). Each
non-comment line uses the same key/value field style as fixture `!!!` entries:

```text
id=name | type=temporary | status=open | fixture=cases/... | reason=...
```

Required fields are `id`, `type`, `status`, `fixture`, and `reason`.

Validate the ledger with:

```bash
cargo run -p rum-difftest -- lane-c
```

The validator checks that every checked fixture `!!! id=...` appears in the
ledger and that every ledger entry is still referenced by a fixture.
