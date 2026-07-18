# Sealed scoping — design (2026-07-18)

Origin: WL15 core survey item 4 (Structured Package Format). User
rulings 2026-07-18:

- Implement the **scoping half** only. The **loader half** (directory
  scanning, an `init` file, file ordering) is ambient-authority territory
  and belongs to the future downstream `rum-fs` loader story — **never
  the kernel**. A "source unit" here is a **host-provided source
  string**, not a directory.
- **Private is REAL encapsulation** — a genuine, kernel-enforced trust
  boundary (sealed context), not the WL-style naming convention.
- v1 is the **full three-tier** surface (Public / Package / default
  Private).

The distinguishing value (survey note, 2026-07-07): context-based
privacy in WL — SPF included — is a *naming convention*, not access
control (a fully qualified name always resolves; it prevents accidents,
not adversaries). Because rum owns its kernel, "private" can be
**genuinely enforced** — qualified-name resolution refused from outside,
introspection gated — turning declarative scoping into a real trust
boundary WL cannot offer. This is the same LCF/Module-boundary
discipline the project already relies on (`mma-hol-module-is-the-trust-boundary`),
generalized from one `Module` to a whole source unit.

## 1. Substrate — the unforgeability primitive already exists

- `Symbol` carries `identity: Option<NonZeroU64>`; equality is
  `id == id && identity == identity` (`rum-core/src/expr.rs`). A gensym
  (`Module`) carries `Some(identity)`; a symbol built by `Symbol[]` /
  `ToExpression` goes through `Symbol::new` → `None`. **Same text, different
  identity ⇒ different symbol.** `identity` is serialized, so it survives
  snapshot (proven by `Module` gensym round-trip).
- The forgery trap: resolution runs `active_symbol_for_name`, which
  consults `symbol_generations : (context, name) → identity`. **If a
  private name were registered there, anyone who spells that name via
  `ToExpression` would get the same identity and reach the sealed
  binding — i.e. it would be forgeable.**
- Therefore sealing MUST follow the **`Module` model**: allocate a fresh
  identity for each private symbol, **rewrite the source unit's body** to
  reference the identity-bearing symbols (reuse
  `scoping/lexical/module.rs::allocate_fresh_symbol` + body rewrite), and
  **keep that identity out of `symbol_generations`**. Outside code
  spelling the name lands on an `identity: None` symbol ≠ the sealed one
  → no access. This covers `ToExpression`, `Symbol[]`, and string-built
  names uniformly, because they all route through name resolution.

## 2. The three tiers (per source unit)

A source unit declares an owning **public context** (e.g. `` `Pkg` ``)
and, at its top, which of its symbols are Public or Package-scoped.
Everything undeclared is Private.

| Tier | Symbol form | Resolvable from | Snapshot |
|---|---|---|---|
| **Public** | ordinary interned (`identity: None`), placed in `` `Pkg` `` | anywhere via the context path (WL rules) | trivial |
| **Package** | ordinary interned, placed in `` `Pkg`PackageScope` `` | only sibling units of the same package | trivial |
| **Private** (default) | **identity-sealed** (`with_identity`, readable name, fresh id), NOT in `symbol_generations` | only inside the owning unit's own body (via the rewrite) | round-trips like a `Module` gensym |

Private is the only tier that is a real boundary; Public/Package are
ordinary name-resolvable symbols (WL-conformant convenience), and are
honest about being conventions.

## 3. Mechanism

1. **Source-unit registration** (host-provided string; a new form/seam —
   naming is a minor fork, §7). Two passes, SPF-style:
   - **Pass 1 (scan, unevaluated):** collect the declared Public and
     Package names. Everything else the unit binds is Private.
   - **Pass 2 (install):**
     - Public/Package names → ordinary interned symbols in the unit's
       public / package context.
     - Private names → `allocate_fresh_symbol` (fresh identity, readable
       name in a `` `Pkg`Private` `` context), body rewritten to
       reference them, identities **not** registered in
       `symbol_generations`. Reuses the `Module` rewrite machinery.
2. **Resolution rules:**
   - Inside the unit's body: private names resolve to their sealed
     symbols via the rewrite (never via the name table).
   - From outside: private names do not resolve (no entry) → the
     boundary. Public via context path; Package only within the same
     package's units.

## 4. Introspection (ruling: hidden by default)

- `Names[]`, `Contexts[]`, `Definition`, `?symbol`, `Information` do
  **not** reveal sealed privates from outside the owning unit.
- The **existing `Capability::Diagnostic`** (no new capability) reveals
  them — the owner/debug view. Within the owning unit's own scope,
  privates are naturally visible (it is their scope).
- `Names["`Pkg`Private`*"]` must not leak private names to a non-owner
  (existence hiding, per the "default hidden" ruling).

## 5. Forgery-path coverage (hard requirement of "real encapsulation")

Each of these must be pinned by a test proving **no access**:

- `ToExpression["`Pkg`Private`x"]` → `identity: None` symbol ≠ sealed →
  no binding reached.
- `Symbol["`Pkg`Private`x"]` → same.
- String-built name (`"`Pkg`Private`" <> "x"` then `ToExpression`) →
  same.
- `Names[...]` / `Contexts[]` do not enumerate the sealed names (non-owner).
- `Definition` / `?` do not print the private body (non-owner).

**Boundary note:** within-language forgery is fully closed (rum source
can only mint `identity: None` symbols). The Rust `with_identity` API can
mint an identity, but that is a **host-side** (trusted operator) surface,
not reachable from evaluated rum source — same trust model as the rest of
the sandbox. A snapshot's text contains identity numbers, but reading /
editing a snapshot is a host-level act, out of the in-language boundary.
State this explicitly so no one later mistakes host reach for a language
hole.

## 6. Snapshot / restore & Definition display

- Sealed private symbols round-trip like `Module` gensyms (identity
  serialized; the definition map is keyed by the identity-bearing
  symbol). Round-trip test required.
- `Definition[publicSym]` shows the public interface. Private definitions
  are hidden from non-owners; shown under `Diagnostic` / to the owner,
  with the readable private name marked private.

## 7. Open sub-forks (minor; director to settle in briefs unless flagged)

- **Declaration surface naming/shape.** A wrapper form over a
  host-provided body, e.g. `DefinePackage["`Pkg`", Public[{f, g}],
  PackageScope[{h}], body...]`, vs standalone top-of-unit markers
  (`Public[...]`, `PackageScope[...]`) with private-by-default. Proposal:
  the wrapper form (one entry, clean two-pass boundary, no reliance on
  statement order). Rum-native names; classic `BeginPackage`/`Begin`/`End`
  remain the substrate underneath (as SPF does). Confirm with user if a
  strong preference exists; otherwise director picks the wrapper form.
- **Host seam.** Whether registration is a pure in-language builtin or
  also exposed as a `HostSession` method (`eval_source_unit`) for the
  embedding API. Default: in-language builtin first; host method only if
  a consumer needs it (matches the a2/a4 dogfood discipline).

## 8. Non-goals (explicit)

- No filesystem loader, directory scan, `init` file, or load-order
  mechanics — that is `rum-fs` downstream, never the kernel.
- No change to `Module`/`Block`/`With` gensym semantics (they are the
  reused substrate).
- Public/Package tiers are NOT hardened into unforgeable boundaries —
  they are honest naming conventions; only Private is sealed.

## 9. Batch plan

1. **Mechanism (spine).** Source-unit registration form; two-pass
   declaration scan; private identity-sealing (reuse gensym + `Module`
   body rewrite); Public/Package/Private resolution rules; the
   forgery-path closure with the §5 test set.
2. **Introspection.** Hidden-by-default across
   `Names`/`Contexts`/`Definition`/`?`/`Information`; `Diagnostic` reveal;
   name-existence hiding.
3. **Snapshot + display + close-out.** Snapshot/restore round-trip of
   sealed privates; `Definition` display rules; language-guide section
   (measured examples); reference regen; tracker close-out.

Each batch gates before the next. This line touches symbol resolution and
scoping — a core path — so the gate watches for any default-path cost
(the private path must be cold; ordinary name resolution unchanged when
no sealed unit is loaded).
