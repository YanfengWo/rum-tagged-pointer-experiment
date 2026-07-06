# Builtin Reference

`builtins.md` is generated from the installed `rum` builtin registry under the
default Cargo feature set. Do not edit it by hand.

Regenerate it with:

```bash
cargo run -p rum-stdlib --bin generate-builtin-reference
```

The current builtin count is stated in the header of `builtins.md` itself
(generated; not tracked here).

The `rum-stdlib` drift test regenerates the file in memory and fails when the
committed reference is out of sync.
