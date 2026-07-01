# VM design research notes — how mature interpreted-language VMs avoid the byte-identical trap

Status: research, 2026-07-01. Motivated by brief-073's failure: our general
`compile_body` HAND-WROTE a separate compiler that RE-DERIVED WL evaluation semantics
(which args to `Eval`, hold behavior, evaluation order) and got them wrong for arbitrary
bodies (`drule_tests` ON `List[38,11]`). This note captures how PyPy and the
closure-generation tradition avoid exactly this class of bug, and what it means for rum.

## The blind spot (why brief-073 failed)

We built a fast path by RE-SPECIFYING the language's evaluation semantics in a second
place (the op-compiler decided "Eval here, hold there, this order"). Any second
specification of semantics can DIVERGE from the real evaluator — and did. slice 1 (pure
construction template) and slices 2/3 (hand-verified exact combinator shapes) worked
precisely because they did NOT re-derive general semantics — slice 1 precomputes only
STRUCTURE (constant subtrees + binding slots; construction is semantics-free), slices 2/3
hand-checked three specific bodies. The general compiler crossed into re-deriving
semantics and broke.

## The governing principle (all mature designs share it)

**Derive the fast path FROM the interpreter; never re-specify semantics.** The fast
path must inherit the interpreter's evaluation decisions by construction, so it CANNOT
diverge. Three concrete embodiments, ranked by applicability to rum:

### 1. Compile-to-closures (Feeley 1987; pl-rants) — the tractable, directly-applicable one
Walk the AST ONCE; emit a chain/tree of closures (each AST node → a closure capturing its
compiled children). Running the chain replaces re-walking the tree. Crucially, each
closure PERFORMS the interpreter's own step (it CALLS the real evaluation primitives),
so evaluation ORDER, side-effect sequencing, and dynamic dispatch are preserved BY
CONSTRUCTION — "the code is very similar to the interpreter," just with the node-type
dispatch (`match expr`) precomputed away. Reported ~2x over tree-walking. Rust shape:
`Box<dyn Fn(&mut Session, &Bindings) -> Result<Expr, EvalError>>`; each closure captures
its child closures by move.

### 2. Lineiform (2022) — the Rust meta-JIT path, IF we go closure-generation first
A meta-JIT library for Rust interpreters structured as closure-generation: it lifts the
x86 the closure chain already emits to an IR, inlines/const-folds, lowers back — so it
optimizes the interpreter's ACTUAL behavior (correct by construction, "no sync bugs
between interpreter and compiler"). Early-stage, x86-only. The takeaway: structure the
evaluator as closure-generation FIRST; a meta-JIT can specialize it LATER for bigger wins
without a hand-written JIT.

### 3. PyPy meta-tracing (Bolz et al.) — the gold-standard principle (too heavy to port)
The JIT is GENERATED from the interpreter: the tracer records the interpreter's ACTUAL
execution of a hot loop, so the compiled trace has the interpreter's semantics by
construction; guards + deoptimization handle divergence from the traced path. This is the
purest form of "interpreter is the spec." It requires RPython + a trace-generator toolchain
(a huge dependency) — NOT portable onto rum's hand-written Rust evaluator. But it is the
principle our failure violated, stated in its strongest form.

## What this means for rum's general VM

Replace the op-compiler-that-re-derives-semantics with **closure generation that DELEGATES
semantics to the existing evaluator**:
- Compile a body ONCE into a closure chain. The compilation precomputes STRUCTURE only —
  constant subtrees (pre-shared), binding/local slots, the control-flow shape — and NEVER
  the semantics.
- Each application-node closure CONSTRUCTS the call and hands it to the real
  `evaluate_call` / `evaluate` (which decides hold/attributes/eval-order/dispatch). So the
  compiled path is byte-identical to the tree-walk BY CONSTRUCTION, for ARBITRARY bodies —
  exactly the property brief-073's op-compiler lacked.
- The direct win is more MODEST than the op approach hoped (you skip the AST re-match + the
  substitution re-walk, not the dispatch itself), but it is SAFE for arbitrary bodies and
  is the correct foundation for a later meta-JIT (Lineiform-style) or heavier specialization.

This reframes the general VM from "a second semantics specification" (fragile) to "a
precomputed structure that runs the interpreter's own steps" (safe). It also explains our
data: the wins so far are structure/dispatch-shaving (bounded), which is consistent with
the ~parity real-load measurements — a bigger win needs the meta-JIT layer, not a bigger
hand-written compiler.

## Open questions to resolve before redesigning (do NOT rush the decision — user 2026-07-01)

- Does closure-generation-delegating-to-`evaluate_call` give enough NEW win over slice 1's
  template to justify the rewrite, or is its value mainly the safe-for-arbitrary-bodies
  generality + the meta-JIT foundation? (It is a CAPABILITY line — mma-hol payoff is not
  the gate; see `docs/bytecode-vm-general-plan.md` + memory `rum-is-general-not-just-mma-hol`.)
- Is a Rust closure chain's allocation/indirection cheaper than the current op-stream + PC
  executor? (Measure structure, not mma-hol payoff.)
- Could a Lineiform-style meta-JIT ever be worth it for rum, or is that too x86-specific /
  early-stage?

## Backend & meta-JIT decision guidance (discussion 2026-07-01 — reference before any codegen work)

Distilled from a discussion of "if we ever want a bigger win than closure generation."

- **Compile-time LLVM ≠ runtime JIT.** rustc's default backend is LLVM (macOS `gcc` is
  clang = LLVM too), and it compiles rum for ANY arch at BUILD time — that is why closure
  generation (brief-074) is portable and FREE: the closures are plain Rust, AOT-compiled
  by LLVM; there is NO codegen of our own. But that build-time LLVM is not in the shipped
  binary and gives NO runtime JIT.
- **If we do NOT do a meta-JIT: nothing to do.** Default LLVM AOT is all we need; the
  closure layer has no backend concerns. This is the expected resting state.
- **If we ever do a meta-JIT (far-future, uncertain): embed Cranelift as a runtime
  codegen LIBRARY** — this is ADDING a dependency rum calls at runtime, NOT "switching
  rustc's backend" (rustc keeps using LLVM to build rum). Cranelift > embedded LLVM
  (llvm-sys/inkwell via ORC) for a JIT: pure-Rust, FAST compile, portable (x86-64 + arm64
  + …). PyPy deliberately wrote its own backends rather than use LLVM precisely because
  LLVM's codegen is too SLOW for a JIT's fast-compile needs.
- **NEVER hand-write a codegen backend.** Instruction selection / register allocation /
  arch lowering / scheduling is a research-grade, many-person-year effort; a naive
  hand-written codegen produces WORSE machine code than the LLVM-AOT tree-walker (the
  tree-walker's real work — `evaluate_call`, matching, arithmetic — is already
  LLVM-optimized; a naive JIT only replaces glue with worse glue → net loss; cf.
  Lineiform's still-immature hand-rolled x86). Use Cranelift; only write the FRONT (rum
  body / closure chain → Cranelift IR).
- **Use the meta-JIT IDEAS, do not sink to raw machine code.** Borrow the PRINCIPLE
  (derive-from-interpreter, correct-by-construction — already embodied by closure
  generation) and the GOAL (specialize the closure chain). Give a HIGH-LEVEL IR to
  Cranelift and let IT descend to the arch; never lift host machine code or hand-write
  x86 (Lineiform's mechanism — the reason it is x86-only: it needs a per-arch host-code
  disassembler AND a per-arch codegen).
- **Even with Cranelift, a meta-JIT's value is UNCERTAIN.** To be worth it, the JIT must
  OUT-specialize LLVM's AOT (inline the closure chain, cross-node const-fold, eliminate
  redundant checks) — Lineiform's goal, but early-stage and unproven. So: closure
  generation is the sensible layer (capability + a free, portable, modest win); a meta-JIT
  is SPECULATIVE — attempt it only if there is a concrete reason to believe specialization
  beats AOT, otherwise stop at the closure layer.
- **Dev-machine note:** currently x86_64 (Intel Mac, `rustc` host `x86_64-apple-darwin`),
  so Lineiform's x86-only would even run here — but rum is a general kernel and must not
  tie itself to x86; a portable backend (Cranelift) is the only acceptable JIT choice.

**Bottom line for decisions:** no meta-JIT → just default LLVM AOT, zero backend worry
(this is brief-074). Meta-JIT → embed Cranelift as a runtime codegen lib + borrow the
meta-JIT ideas + let Cranelift own the arch; never hand-write codegen, never sink to
x86_64; and only if specialization is plausibly worth more than AOT.

## Sources

- [Tracing the Meta-Level: PyPy's Tracing JIT Compiler (Bolz et al.)](https://dl.acm.org/doi/pdf/10.1145/1565824.1565827)
- [Motivating JIT Compiler Generation — RPython docs](https://rpython.readthedocs.io/en/latest/jit/overview.html)
- [Using closures for code generation (Feeley & Lapalme, 1987)](http://www.iro.umontreal.ca/~feeley/papers/FeeleyLapalmeCL87.pdf)
- [Compiling to Closures — pl-rants](https://pl-rants.net/posts/compile-to-closures/)
- [Lineiform, a meta-JIT library for Rust interpreters — Red Vice](https://redvice.org/2022/lineiform-rust-meta-jit/)
- [Threaded Code Generation with a Meta-Tracing JIT Compiler (arXiv 2106.12496)](https://arxiv.org/pdf/2106.12496)
