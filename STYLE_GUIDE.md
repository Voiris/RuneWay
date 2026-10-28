# RuneWay Style Guide

This guide documents the conventions used in the RuneWay codebase. It
supplements `rustfmt` and Clippy with project-specific rules for compiler
layers, diagnostics, source locations, runtime boundaries, tests, and
documentation.

When existing code and this guide disagree, make the smallest consistent
change. Update this document when the project intentionally adopts a different
convention.

## Tooling

Formatting is defined by `rustfmt.toml`. Some configured options require the
nightly formatter even though the project itself does not require nightly Rust.

Format and validate changes with:

```console
cargo +nightly fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

Use narrower package commands while iterating, but run the relevant workspace
checks before merging changes to shared compiler behavior. Do not override the
repository formatting settings or add formatting that `rustfmt` will remove.

## Language and naming

Write source code, identifiers, comments, diagnostics, tests, commit messages,
and documentation in English.

Follow standard Rust naming:

- modules, functions, methods, and variables use `snake_case`;
- structs, enums, traits, and enum variants use `PascalCase`;
- constants use `SCREAMING_SNAKE_CASE`;
- identifier wrappers end in `Id`, such as `HirId` and `MirLocalId`;
- type representations use a layer prefix when ambiguity is possible, such as
  `HirTy` and `MirTy`;
- error enums end in `Error` when callers can distinguish their failures.

Prefer complete domain names such as `source_id`, `runtime_function`, and
`return_type`. Short names are appropriate when conventional and local, such
as `id`, `ty`, `lhs`, `rhs`, and `op`.

Use compiler terminology consistently:

- **token** for lexer output;
- **AST** for parsed source structure;
- **HIR** for lowered, source-oriented semantic structure;
- **MIR** for executable compiler IR;
- **runtime function** for a compiler-known native function;
- **JIT** for in-process execution;
- **AOT** for object or executable generation.

## Imports

Import ordering and grouping are controlled by `rustfmt.toml`. Treat formatter
output as the source of truth instead of arranging imports manually.

Avoid glob imports outside tests. A test module may use `use super::*` when it
tests the adjacent implementation and the imported surface remains clear.

Import types from the layer that owns them. Do not re-declare a shared ABI,
source, or diagnostic type in a downstream crate for convenience.

## Workspace and crate boundaries

Each compiler crate has a distinct responsibility:

- `runec_source` owns source files, byte positions, spans, and source maps;
- `runec_errors` owns shared diagnostics and rendering;
- `runec_parse` owns lexing and parsing;
- `runec_ast` owns syntax-level structures;
- `runec_hir` owns AST lowering and HIR data;
- `runec_semantic` owns name resolution and type checking;
- `runec_mir` owns executable IR and HIR-to-MIR lowering;
- `runec_codegen_cranelift` owns Cranelift lowering and JIT/AOT backends;
- `runec_builtins` owns language-visible built-in declarations;
- `runec_abi` owns stable compiler/runtime contracts;
- `runec_runtime` owns native runtime implementations;
- `runec_utils` owns general-purpose shared utilities;
- `runec_test_utils` owns fixtures shared by compiler tests;
- `runec` owns CLI behavior and pipeline orchestration.

Keep dependencies flowing in the direction of the compiler pipeline. Avoid
making lower-level crates depend on orchestration or backend crates.

Place behavior in the crate that owns the invariant. Runtime symbol identifiers
belong in `runec_abi`, while their native addresses belong in `runec_runtime`.

## Module structure and visibility

Use this order within a module where practical:

1. child module declarations;
2. imports;
3. constants;
4. data types;
5. public or crate-visible entry points;
6. private helpers;
7. the `#[cfg(test)]` test module.

Keep visibility as narrow as possible:

- use private items by default;
- use `pub(super)` for communication with a parent module;
- use `pub(crate)` for APIs shared within one crate;
- use `pub` only for an intentional cross-crate API.

Split a module when a section has a separate responsibility or data model. Do
not create a module solely to shorten a small file.

## Compiler pipeline

Preserve the separation between compiler layers:

```text
source -> tokens -> AST -> HIR -> semantic analysis -> MIR -> codegen
```

Each lowering step should consume concepts from the previous layer and produce
only concepts owned by the next layer. Do not leak parser tokens into HIR or
Cranelift values into MIR.

Keep lowering functions focused and explicit. Prefer a small helper for each
expression, statement, or type family over one deeply nested function.

Unsupported syntax must produce a diagnostic or typed error. It must not
silently disappear or panic on user input.

When a pass records information outside its returned IR, keep that side data
keyed by the layer's stable identifier type.

## Source positions and spans

Use `BytePos`, `Span`, `Spanned<T>`, and `SourceId` instead of raw offsets or
ad-hoc ranges at compiler boundaries.

Preserve the most specific source span available while lowering. Diagnostics
should point to the expression, type, name, or token responsible for a failure
rather than an enclosing function or file.

Use checked position arithmetic when offsets may come from input or cross a
source boundary. Conversions to compact identifier and position types must
reject overflow.

Do not use character counts as byte offsets. RuneWay source is UTF-8, and all
source slicing must respect UTF-8 boundaries.

## Control flow

Prefer early validation and early returns over deeply nested branches.

Use `let ... else` when the alternative exits the current flow. Use exhaustive
`match` expressions for compiler enums; do not add wildcard arms merely to
avoid handling a new variant.

Use iterator methods when they make a transformation clearer. Prefer an
explicit loop when it performs several operations, maintains state, or emits
diagnostics.

Do not use `unwrap()`, `expect()`, `panic!()`, `todo!()`, or
`unimplemented!()` on paths reachable through user source or CLI input. Return
an error or emit a diagnostic instead. `expect()` is acceptable for a local
invariant that cannot be invalidated by input or configuration.

## Diagnostics and errors

Use structured diagnostics for errors tied to RuneWay source. Use ordinary
Rust errors for I/O, CLI orchestration, backend setup, and failures that do not
point into source code.

A diagnostic should contain:

- a concise message describing the failure;
- a primary label on the responsible span;
- secondary labels only when they explain a relationship;
- a note or help message when it provides an actionable next step.

Diagnostic messages should:

- start with lowercase unless they begin with a proper name;
- describe the failed requirement rather than an implementation detail;
- use RuneWay terminology consistently;
- avoid trailing punctuation for short labels and headings.

Do not discard errors from primary operations. Best-effort cleanup may ignore
an error only after the original failure has already been preserved.

## HIR, semantic analysis, and MIR

HIR should retain source meaning and stable identities needed by later passes.
Name resolution must record what a path refers to instead of making later
passes repeat lookup logic.

Semantic analysis owns type compatibility and language-level constraints. MIR
lowering may rely on validated semantic information, but it should still
report a controlled diagnostic if an invariant is absent.

MIR should represent executable behavior without parser or AST details. Keep
MIR construction deterministic so tests, diagnostics, and generated code do
not depend on hash iteration order.

Use newtype identifiers rather than interchangeable integer indexes. Keep
index conversion in the identifier implementation and validate narrowing
conversions.

## Runtime ABI and unsafe code

The runtime ABI is a compatibility boundary. Define function identifiers,
parameter types, return types, and symbol names in `runec_abi`. Keep runtime
implementations in `runec_runtime` and code generation in
`runec_codegen_cranelift`.

Do not duplicate symbol names or signatures across crates. Resolve them through
the shared ABI declarations.

Every `unsafe` block must have a nearby `SAFETY:` comment stating the invariant
that makes it valid. Public unsafe functions must include a `# Safety` section
documenting caller obligations.

Validate raw pointers before constructing slices. A null pointer is valid only
when the relevant ABI contract explicitly permits it.

Changes to runtime symbols or signatures require coordinated updates to the
ABI, runtime, codegen, and their tests. Treat incompatible ABI changes as
breaking changes.

## Code generation

Keep JIT and AOT lowering shared wherever their semantics are identical.
Backend adapters should configure output and execution, not reimplement MIR
lowering.

Translate RuneWay and MIR types to ABI and Cranelift types in dedicated
helpers. Handle every type variant explicitly.

Backend failures should retain enough context to identify the function or
stage being compiled. Do not expose raw backend errors when a short compiler
context message would make the failure actionable.

## Tests

Keep unit tests next to the implementation in a `#[cfg(test)] mod tests`
module. Put cross-crate CLI behavior in integration tests.

Name tests after observable behavior:

```rust
fn rejects_unknown_runtime_symbols()
fn lowers_path_expression_preserving_segments()
fn reports_mismatched_argument_types()
```

Prefer testing:

- valid and malformed source input;
- boundary values and identifier overflow;
- source span preservation;
- name resolution and type-checking failures;
- deterministic HIR and MIR construction;
- runtime ABI declarations and symbol resolution;
- shared JIT and AOT lowering behavior;
- CLI exit status and generated artifacts.

Test both the successful path and the nearest meaningful failure path. Assert
structured values rather than debug output when possible.

`unwrap()` and `expect()` are acceptable in tests when failure should abort the
test immediately. Use `expect()` when its message improves the failure report.

Tests that touch the filesystem must use a dedicated temporary location and
must not depend on developer-installed tools, global state, or network access.

Do not weaken assertions to accommodate nondeterministic behavior. Make the
production behavior deterministic instead.

## Comments and documentation

Prefer code that expresses intent through types, names, and small helpers. Add
a comment when it explains:

- why a compiler invariant exists;
- a source-location or Unicode constraint;
- a runtime ABI or safety requirement;
- behavior that would otherwise appear accidental.

Do not add comments that merely restate the next line of code.

Use `///` for public APIs and domain types whose contract is not evident from
their fields and signatures. Include examples when correct use is not obvious.

Update documentation when a change affects supported syntax, compiler stages,
CLI options, generated artifacts, runtime behavior, or contributor workflows.

## Scope of changes

Keep refactoring separate from behavioral changes when practical. Avoid
unrelated formatting, renaming, or cleanup in a focused fix.

Code and its direct tests belong in the same change. Documentation describing
that behavior should be updated with it.

New code must follow this guide. Existing code may be brought into alignment
incrementally when it is already being modified.
