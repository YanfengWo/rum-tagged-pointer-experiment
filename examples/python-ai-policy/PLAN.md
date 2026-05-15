# Python AI Policy Example Plan

This directory is a placeholder for a future Python-hosted example project.
Do not treat it as active implementation work until the core `rum` runtime and
host-adapter APIs are stable enough.

## Goal

Build a small Python AI-infrastructure example that demonstrates `rum` as an
embedded symbolic policy / rewrite / validation layer.

The example should not become a full AI framework, training stack, serving
engine, or networked gateway. Python should own I/O, SDK calls, async runtime,
logging, and project-specific objects. `rum` should own the compact symbolic
middle layer:

- schema normalization
- defaults and option resolution
- capability matching
- provider / model / fallback planning
- rule-based validation
- structured diagnostics through `Failure[...]`-style results
- explanation traces for why a policy decision was made

## Candidate Shape

The first target should be an offline policy-router style example inspired by
projects such as LiteLLM, vLLM, and SGLang.

Python-side inputs should be plain dictionaries or dataclasses describing:

- task kind
- messages
- tool requirements
- budget and latency constraints
- privacy / locality constraints
- available providers and models
- provider health and rate-limit state
- model capabilities

The `rum` rule core should return a canonical plan such as:

- selected provider
- selected model
- fallback chain
- retry policy
- tool policy
- cost / latency notes
- validation failures or warnings

All provider calls should be mocked in the example. The point is policy
planning, not live API access.

## Architecture

Use a unified symbolic backend with thin project adapters:

```text
Python project adapter
  -> canonical rum IR
  -> rum policy / rewrite / validation engine
  -> canonical result IR
  -> Python project adapter
```

The first canonical IR should stay narrow. A possible starting point:

```wolfram
AIRequest[
  <|
    "Task" -> ...,
    "Messages" -> {...},
    "Tools" -> {...},
    "Constraints" -> <|
      "MaxCost" -> ...,
      "MaxLatency" -> ...,
      "Privacy" -> ...
    |>,
    "Providers" -> {...},
    "Capabilities" -> {...}
  |>
]
```

Future adapters can map external projects onto this IR:

- LiteLLM-style gateway request/config -> `AIRequest`
- vLLM / SGLang serving request -> `AIRequest`
- later, Axolotl / XTuner / LLaMA-Factory recipe config -> `TrainingRecipe`
- later, PyTorch FX graph -> `GraphRewrite`

Avoid designing one large universal IR up front.

## Development Strategy

This project cannot follow the same path as `mma-hol`, which can first be
developed as a pure Wolfram-language project and later migrated to `rum`.
Host examples must include the host-adapter boundary from the beginning,
because serialization, error mapping, request shape, and loading mechanics are
part of the example's purpose.

The recommended sequence is:

1. Define the canonical IR and adapter contract.
2. Implement a host-free rule core that can be tested from fixtures.
3. Write a minimal Python adapter for dict/dataclass input and output.
4. Add offline fixtures for provider/model capability planning.
5. Add end-to-end host tests once the `rum` embedding API is stable.

The rule core may still be prototyped against Wolfram-compatible association
data where useful, but the host boundary must be designed natively for `rum`.

## Scope Limits

Keep the first version around 3k-5k lines total.

In scope:

- canonical request / result IR
- provider/model routing rules
- fallback and retry policy planning
- option/default normalization
- structured validation failures
- explanation traces
- offline Python tests

Out of scope:

- real provider network calls
- production proxy server
- database or telemetry backend
- UI
- async serving infrastructure
- GPU scheduling
- tensor kernel generation
- full training recipe compiler
- full DNN JIT

If the example grows beyond 10k lines, cut scope rather than expanding it.

## Later Extensions

Once the first policy-router slice works, possible follow-up examples are:

- training recipe planner inspired by Axolotl, LLaMA-Factory, or XTuner
- RAG / evaluation planner inspired by LlamaIndex, Haystack, OpenCompass, or
  Ragas
- PyTorch FX symbolic graph rewrite demo for shape / dtype checking and simple
  fusion decisions

These should share the same backend style but use separate narrow canonical
IRs rather than forcing all domains into the initial `AIRequest` shape.
