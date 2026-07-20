# Sealed scoping — design (2026-07-18; grounded + snapshot contract 2026-07-19)

Origin: WL15 core survey item 4 (Structured Package Format). User rulings
2026-07-18 (scope, tiers), 2026-07-19 (snapshot contract + re-link, §6).
Substrate grounded by brief-268 (`docs/dev/sealed-scoping/grounding-findings.md`);
§1 below is the corrected substrate, superseding the original claim.

- Implement the **scoping half** only. The **loader half** (directory
  scanning, an `init` file, file ordering) is ambient-authority territory
  and belongs to the future downstream `rum-fs` loader story — **never
  the kernel**. A "source unit" here is a **host-provided source
  string**, not a directory.
- **Private is REAL encapsulation** — a genuine, kernel-enforced trust
  boundary (sealed identity), not the WL-style naming convention.
- v1 is the **full three-tier** surface (Public / Package / default
  Private).

The distinguishing value (survey note 2026-07-07, sharpened 2026-07-19):
context-based privacy in WL — SPF included — is a *naming convention*,
not access control (a fully qualified name always resolves; it prevents
accidents, not adversaries). Because rum owns its kernel, "private" can
be **genuinely enforced**, and — with the §6 re-link contract — genuinely
enforced **without losing persistence**: one mode carries a boundary
stronger than a WL Module gensym (unforgeable rather than spellable) with
checkpointability a WL gensym design cannot have. The mma-hol strict-vs-
stable split (real boundary XOR persistable internal state) dissolves
into a single mode that exceeds both halves. **User ruling 2026-07-19:
this dual dividend is publication-narrative material — the batch-3
close-out adds it to README.md as a flagship divergence-with-value
feature. (rum-hol itself is a calibration thought experiment, not a
direction.)**

## 1. Substrate — corrected by grounding (brief-268)

The original §1 claimed a `Module` gensym carries `Some(identity)` and
that sealing could reuse `allocate_fresh_symbol`. **Brief-268 refuted
both** (probe: `Module[{x}, Hold[x]]` → `Hold[x$1]`; spelling `x$1`
directly, via `Symbol["Global`x$1"]`, and via `ToExpression` all reach
the escaped local's value). Module locals are name-mangled ordinary
symbols (`identity: None`); their uniqueness is textual and therefore
source-reachable. The actual unforgeability substrate:

- `Symbol.identity: Option<NonZeroU64>`; equality requires both the
  interned text id AND the identity to match (`rum-core/src/expr.rs`).
  Same text, different identity ⇒ distinct, non-equal symbols.
- **Evaluated rum source has no constructor for a non-`None` identity.**
  Direct spelling, `Symbol[...]`, and `ToExpression` all route through
  `resolve_symbol_name` → `active_symbol_for_name`, which returns
  `with_identity` ONLY on a `symbol_generations` hit (the
  `Remove`-generation mechanism) and `Symbol::new` (`identity: None`)
  otherwise.
- **The seal**: mint a FRESH identity for each private symbol (a NEW
  allocator — `allocate_fresh_symbol` is name-mangling and NOT
  reusable), rewrite the unit body via `substitute_lexical_bindings`
  (reusable as-is; it is the `Module` body-rewrite machinery and handles
  nested lexical shadowing), and **keep the private `(context, name)`
  OUT of `symbol_generations`**. Outside spelling then lands on the
  `identity: None` decoy → full-symbol equality fails → no sealed
  binding reached. **Zero resolver change: the existing miss branch IS
  the boundary.** `symbol_exists` on a decoy is likewise false (maps are
  keyed by full symbols), so sealed definitions never make bare names
  resolve on the context path.
- **Identity partition (director ruling):** sealed identities live in
  the bit-62 region (`SEALED_IDENTITY_MARKER = 1 << 62`, id = marker |
  counter). Rule-RHS alpha renames own bit 63; `Remove` generations are
  small per-name integers. **Guard:** `mark_symbol_removed` inherits the
  removed symbol's identity into `symbol_generations`
  (`max(current, removed) + 1`) — sealed-region identities MUST be
  excluded from that write, or removing a sealed symbol pollutes the
  name's generation into the sealed region (spelled names would resolve
  into it, and it could collide with a future minted seal).
- The Rust `with_identity` constructor and snapshot text remain
  **host-side trusted surfaces**, not reachable from evaluated rum
  source — same trust model as the rest of the sandbox. State this
  explicitly so no one later mistakes host reach for a language hole.

## 2. The three tiers (per source unit)

A source unit declares an owning **public context** (e.g. `` `Pkg` ``)
and, at its top, which of its symbols are Public or Package-scoped.
Everything the unit binds and does not declare is Private.

| Tier | Symbol form | Resolvable from | Snapshot |
|---|---|---|---|
| **Public** | ordinary interned (`identity: None`), placed in `` `Pkg` `` | anywhere via the context path / fully qualified name (WL rules) | via §6 re-registration |
| **Package** | ordinary interned, placed in `` `Pkg`PackageScope` `` | anywhere via fully qualified name — an honest **convention tier** signaling "for sibling units"; no resolver enforcement (enforcement would put a policy check on the hot path; see §8) | via §6 re-registration |
| **Private** (default) | **identity-sealed** (`with_identity`, readable name in `` `Pkg`Private` ``, bit-62 id), NOT in `symbol_generations` | only code holding the rewritten symbol (the unit's own body) | via §6 re-registration (+ re-link for leaked references) |

Private is the only tier that is a real boundary; Public/Package are
ordinary name-resolvable symbols (WL-conformant convenience), honest
about being conventions.

## 3. Mechanism

1. **Registration form** — the wrapper (settled, §7):

   ```
   DefinePackage["`Pkg`", Public[{f, g}], PackageScope[{h}], body...]
   ```

   `HoldAllComplete`, kernel builtin (the `Module` wiring class:
   catalog `special_with_hold` + adapter + session method). `Public` /
   `PackageScope` are inert declaration heads. The wrapper form doubles
   as the unit's snapshot representation (§6) — its second independent
   justification. v1 does NOT mutate `$ContextPath` (callers use
   qualified names or extend the path themselves).
2. **Pass 1 (scan, held, unevaluated):** validate the context string;
   collect the declared Public and Package name lists (each declaration
   head at most once, plain symbols only, current-context names only —
   foreign-context declarations are rejected). Walk the body's top-level
   forms (`CompoundExpression`-flattened) and collect **bound** symbols
   from the recognized definition-bearing forms: the ordinary/tagged
   assignment families (via the existing definition-target
   classification), `SetAttributes`/`SetOptions`-style targets, and
   message assignments. **v1 contract: sealing is scan-determined** —
   top-level forms the scan does not classify contribute no bindings
   (documented); a dynamically computed bind lands on whatever ordinary
   symbol evaluation produces. Symbols lowered into the
   registration-time current context are package-local candidates;
   explicitly foreign-context symbols are references, never sealed.
3. **Pass 2 (install):** classify (declared Public → `` `Pkg` ``;
   declared Package → `` `Pkg`PackageScope` ``; bound-but-undeclared →
   Private, minted via the sealed allocator). Build ONE `SymbolExprMap`
   (as-lowered symbol → target symbol), rewrite every body form with
   `substitute_lexical_bindings`, then evaluate the rewritten forms in
   order. Record the unit in the **sealed registry**: package context,
   the held ORIGINAL wrapper `Expr` (for §6 emission), a registration
   epoch, the name→sealed-symbol index, and the owned public/package
   symbol sets. All pre-install phases are pure (failures there are
   atomic). A body-evaluation failure clears the state of every sealed
   symbol this registration minted, records nothing, messages, and
   returns `$Failed`; an `InputRequired` suspension during install is
   treated the same way (fail-closed; v1 has no registration resume
   frame). Re-registering the same package context replaces the unit in
   place (old sealed state cleared, epoch bumped, fresh identities;
   public/package symbols are overwritten by the new body, WL-style).
4. **Resolution rules:** the resolver is untouched. Inside the unit,
   private references were rewritten to the sealed symbols (never via
   the name table). From outside, spelling in any form produces the
   decoy → no access. Public via context path/FQN; Package via FQN
   (convention).

## 4. Introspection (ruling: hidden by default — batch 2)

- `Names[]`, `Contexts[]`, `Definition`, `Information` do **not** reveal
  sealed privates from outside the owning unit. (There is NO `?symbol`
  prefix syntax today — grounding Q5; if one is ever added it must route
  through `Information`'s authorization point, not grow a second gate.)
- The **existing `Capability::Diagnostic`** (no new capability) reveals
  them — the owner/debug view.
- ``Names["`Pkg`Private`*"]`` must not leak names to a non-owner
  (existence hiding). Note the `explicit_context` branch in
  `names_matching_patterns` bypasses the `visible_contexts` filter — the
  sealed filter must sit before that branch.
- **The enumeration surface is wider than the display builtins**
  (grounding Q5): symbol completion, the global `Messages`/
  `MessageTags`/`MessageDefinitions` catalogs, `Save`/`DumpSave`'s
  `symbols_in_context`, and the host summary/context APIs all enumerate
  the same maps. Batch 2 covers the in-language surfaces and classifies
  the host APIs (trusted embedder surface vs capability-governed).
- Batch 1 explicitly does NOT hide names: the seal (unreachability) is
  batch 1; invisibility is batch 2.

## 5. Forgery-path coverage (hard requirement of "real encapsulation")

Each of these must be pinned by a test proving **no access** (batch 1):

- ``ToExpression["`Pkg`Private`x"]`` → `identity: None` decoy ≠ sealed →
  no binding reached.
- ``Symbol["`Pkg`Private`x"]`` → same.
- String-built name (``"`Pkg`Private`" <> "x"`` then `ToExpression`) →
  same.
- Direct fully qualified spelling in source → same (this case is why the
  Module model failed grounding).
- Assigning to the spelled decoy creates a distinct ordinary binding:
  the sealed binding is unaffected and the public interface still
  reaches the sealed private (discrimination test).
- Removing a sealed symbol writes NO `symbol_generations` entry (the §1
  guard), and the spelled name remains a plain decoy afterwards.
- Batch 2 adds: `Names`/`Contexts` non-enumeration, `Definition`/
  `Information` non-disclosure.

**Boundary note:** within-language forgery is fully closed (rum source
can only mint `identity: None` symbols). The Rust `with_identity` API
and snapshot text are host-side (trusted operator) surfaces, not
reachable from evaluated rum source. State this explicitly so no one
later mistakes host reach for a language hole.

## 6. Snapshot / restore — the re-registration + re-link contract (user ruling 2026-07-19)

Grounding fact (Q6): the public host snapshot is a TEXTUAL source image
(`snapshot_to_bytes` → `snapshot_definitions_source`; restore = reset +
evaluate). Symbol display never prints identity, so identity-bearing
state cannot round-trip through per-symbol text — and MUST NOT: a
textual identity literal would hand `ToExpression` a forgery path. (The
original §6 "round-trips like a Module gensym" was wrong twice: Module
gensyms carry no identity, and the snapshot is text.)

**Contract — three clauses:**

1. **The registration form IS the unit's snapshot representation**
   (snapshot-by-re-registration). The registry keeps each unit's held
   `DefinePackage` form; the snapshot renderer emits registered units
   FIRST (in registration order, re-registration keeping the original
   slot) as their original wrapper forms, and SKIPS every unit-owned
   definition key — public, package, and sealed (public bodies embed
   sealed symbols, so per-symbol textual rendering is impossible
   anyway). Restore re-runs registration: fresh identities, body
   re-rewritten, seal re-established. Identity numbers are not stable
   across restore and do not need to be — they are unobservable
   in-language (never displayed; restore resets every pre-restore
   capture).
2. **Post-registration mutations of PRIVATE symbols do not persist**
   (v1): restore returns a unit to its as-registered state (re-derive —
   the same spirit as the lazy-sequences "snapshot re-derives, no memo"
   ruling; for checkpoint/rollback embedding scenarios this is exactly
   the wanted isolation semantics). Public/Global mutations persist
   textually as today, emitted after the unit forms.
3. **Sealed references leaked into non-unit state** (the opaque-value
   pattern: sealed heads inside user-held values — the rum-hol thought
   experiment): the renderer must DETECT identity-bearing sealed
   references in emitted values. **Floor (mandatory): fail loudly at
   snapshot time — never emit text that silently decoy-degrades a
   sealed reference.** **Full mechanism (in the design per the
   2026-07-19 ruling): re-link** — emit such values inside a marked
   form carrying the owning unit + its registration epoch; during
   restore only (host-trusted restore mode; the marker form is refused
   in ordinary evaluation), after the unit's re-registration, rewrite
   the marked values' private names onto the freshly minted sealed
   symbols via the unit's name→sealed index. **Epoch rule:** marks from
   a STALE epoch (the unit was re-registered after the value was made)
   are NOT re-linked — live sessions already fail closed on stale sealed
   heads, and restore must never be more permissive than live; stale
   marks fail loudly. Discrimination is exact: only references that
   were genuinely identity-bearing at snapshot time are marked; stored
   decoys stay decoys.
   *Implementation note (3a, brief-271): the epoch rule is enforced as
   snapshot-time ORPHAN refusal — a snapshot is internally
   self-consistent (unit forms and markers are written together), so
   restore-side staleness cannot arise; an in-session value holding a
   superseded registration's identity shows up at snapshot time as an
   identity absent from every current `private_index` and refuses the
   snapshot there. Markers carry unit contexts; epochs ride along as
   diagnostics only. Sealed identities are session-scoped (numeric
   reuse across independent sessions is permitted — nothing in-language
   survives a reset), which is what makes fresh-mint-on-restore sound.*

Why this and not an identity-preserving serde section: text stays text
(single, auditable format), no identity number becomes persistent ABI,
no persisted identity counter, and the no-source-identity-literal
invariant holds even inside snapshot files. The payoff is the §0 dual
dividend: real encapsulation AND persistence in one mode.

`Definition[publicSym]` shows the public interface. Private definitions
are hidden from non-owners (batch 2); shown under `Diagnostic`, with the
readable private name marked private.

## 7. Sub-forks — settled

- **Declaration surface: the wrapper `DefinePackage` form** (one entry,
  clean two-pass boundary, no reliance on statement order — and it is
  the natural §6 snapshot unit). Rum-native names.
- **Host seam:** in-language builtin first; a `HostSession` method only
  if a consumer needs it (a2/a4 dogfood discipline).
- **Return value:** the package context string on success; registration
  failure messages + `$Failed` (malformed calls follow the normal
  argument-message conventions).
- **Identity partition:** bit-62 region + the `mark_symbol_removed`
  exclusion (§1).
- **Duplicate private display names across units** (same readable text,
  different identities): a batch-2 display concern — Diagnostic views
  qualify by owning unit; ordinary resolution is never involved.

## 8. Non-goals (explicit)

- No filesystem loader, directory scan, `init` file, or load-order
  mechanics — `rum-fs` downstream, never the kernel.
- No change to `Module`/`Block`/`With` gensym semantics (their
  name-mangling machinery stays as-is; only `substitute_lexical_bindings`
  is shared).
- Public/Package tiers are NOT hardened into unforgeable boundaries —
  honest naming conventions; only Private is sealed.
- **No identity literal in any evaluated-source or snapshot-text syntax,
  ever** — the §6 re-link marker carries names + unit/epoch, never
  identity numbers.
- No `$ContextPath` auto-mutation in v1.

## 9. Batch plan — ALL LANDED (line complete 2026-07-20)

Batch 1 = brief-269 (f2e1448); batch 2 = brief-270 (e9fb37c); batch 3a =
brief-271 rev 1 (4489c28); batch 3b = director close-out (guide §Sealed
Packages, README divergence section, this doc's final state). Original
plan follows.

1. **Mechanism (spine) — brief-269.** The `DefinePackage` registration
   chain (catalog + adapter + session method, `Public`/`PackageScope`
   inert heads, message declarations per the registry enforcement);
   two-pass scan; sealed allocator (bit-62) + registry + the
   session-snapshot struct ripple (four-site field mirror); body rewrite
   + install + failure cleanup + re-registration replace; the
   `mark_symbol_removed` exclusion; the §5 test set; resolution A/B gate
   (resolver byte-untouched — expect noise-level).
2. **Introspection.** Hidden-by-default across the full grounding-Q5
   enumeration list (`Names` incl. the explicit-context bypass, `NameQ`,
   `Contexts`, `Definition`/`Information`, completion, message catalogs,
   `Save`/`DumpSave`) + `Diagnostic` reveal + name-existence hiding +
   host-API classification.
3. **Snapshot + display + close-out.** Renderer unit emission +
   owned-key skip; leak-detection floor; **re-link** (marker form,
   restore-mode rewrite, epoch rule); `Definition` display rules;
   language-guide section (measured examples); reference regen;
   **README.md divergence-with-value section (user ruling 2026-07-19)**;
   tracker close-out.

Each batch gates before the next. This line touches symbol resolution
and scoping — a core path — so the gate watches for any default-path
cost (the private path must be cold; ordinary name resolution unchanged
when no sealed unit is loaded — batch 1 achieves this by construction:
the resolver is not modified at all).
