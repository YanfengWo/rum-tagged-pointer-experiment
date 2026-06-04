# Divergence Ledger

Lane C data lives here.

Known differences should be tracked as checked fixture data, not buried in
comments or silently accepted in runner logic.

The active ledger is [known-differences.txt](./known-differences.txt). Each
non-comment line uses the same key/value field style as fixture `!!!` entries:

```text
id=name | type=temporary | status=open | fixture=cases/... | reason=... | tracked_by=TODO.md:... | close_when=...
```

Required fields are `id`, `type`, `status`, `fixture`, `reason`,
`tracked_by`, and `close_when`. `tracked_by` should name the TODO, matrix, or
roadmap item that owns the divergence. `close_when` should state the concrete
condition for removing the ledger entry.

Validate the ledger with:

```bash
cargo run -p rum-difftest -- lane-c
```

The validator checks that every checked fixture `!!! id=...` appears in the
ledger, that every ledger entry is still referenced by a fixture, and that
each ledger row carries the required tracking metadata.
