# 🧪 ᚱuneWay

*RuneWay* is a programming language written in Rust, with syntax inspired by Rust, Python, and JavaScript.

> RuneWay is under active development. The compiler frontend currently reaches HIR name resolution
> and basic semantic type checking. MIR, code generation, and an executable CLI pipeline are not
> implemented yet.

## 🔧 Features (WIP)

### ✨ Language Features

- [x] Comments (`//`, `/* */`)
- [x] Function declarations and calls (`act name() {}`)
- [x] Local bindings (`let`, `let mut`)
- [x] Primitive literals (`int`, `float`, `bool`, `char`, `string`)
- [x] Static type annotations in functions (`act f(x: int) -> int`)
- [x] Tuple and array type annotations
- [ ] Binary and Unary Operations
- [ ] String Interpolation (`f"..."`, `r"..."`, `fr"..."`)
- [ ] Logic and Loops (`if`, `while`, `for`, `loop`)
- [ ] Constants (`const PI: float = 3.14`)
- [ ] Generic types (`SomeType<T, E, 3>`)
- [ ] Assignments and destructuring lowering
- [ ] `null` and nullable types

### 🧱 Architecture

RuneWay is a compiled language. The pipeline is:

```
source (.rnw)
   │
   ▼
 Lexer ──► Tokens
   │
   ▼
 Parser (Recursive Descent + Pratt) ──► AST
   │
   ▼
 HIR lowering ──► unresolved HIR
   │
   ▼
 Name resolution ──► resolved HIR
   │
   ▼
 Semantic type checking ──► TypeInfo
   │
   ▼
 MIR lowering ──► MIR
   │
   ▼
 Cranelift IR codegen ──► native binary
```

- [x] Lexer (out: Tokens)
- [ ] Parser (out: AST, Recursive Descent + Pratt) — *in progress*
- [ ] HIR (out: HIR) — *in progress*
- [x] Basic name resolution for functions, parameters, locals, primitive types, and built-ins
- [x] Basic semantic type checking for calls, arguments, locals, and function returns
- [x] Built-in declarations separated from the runtime ABI
- [x] Runtime ABI declarations and native `print`/`println` symbols
- [ ] MIR (out: MIR)
- [ ] Cranelift IR codegen (out: native binary)
- [ ] `runec` CLI pipeline

### Current Frontend Scope

The implemented frontend path is:

```text
AST
 └─► HIR lowering
      └─► name resolution
           └─► type checking
                └─► TypeInfo
```

HIR currently represents functions, parameters, local bindings, literals, blocks, paths, calls,
tuple types, and array types. Resolved value paths use a common `Res` representation for locals,
definitions, and compiler-provided built-ins.

Semantic analysis currently provides:

- duplicate and unresolved name diagnostics;
- primitive, struct, and enum type resolution;
- local and function signature type information;
- argument count and type checks;
- function return type checks;
- built-in contract constraints such as `Display`.

These stages are covered by unit tests, but they are not yet connected into a source-to-binary
end-to-end pipeline.

### Compiler Crates

- `runec_ast` — syntax tree definitions.
- `runec_parse` — lexer and parser.
- `runec_hir` — HIR definitions and AST-to-HIR lowering.
- `runec_semantic` — name resolution and type checking.
- `runec_builtins` — language-visible built-in declarations and constraints.
- `runec_abi` — stable runtime function IDs and ABI signatures.
- `runec_runtime` — native implementations and runtime symbol resolution.
- `runec` — future compiler CLI entry point.

### 🪵 Built-ins

- [x] Language declarations for `print` and `println`
- [x] `Display` constraint implemented for `str`
- [x] Native runtime symbols for string output
- [ ] MIR/ABI lowering from RuneWay `str` to `(ptr, len)`
- [ ] `Display` implementations for numeric and user-defined types
- [ ] Native console in (`input`)

### 🧩 Standard Library

- [ ] Module System (Imports: `import some_module::some_submodule`)
- [ ] std::buffered
- [ ] std::http
- [ ] std::json
- [ ] std::itertools
- [ ] std::random
- [ ] std::decimal
- [ ] std::math
- [ ] std::files
- [ ] std::time
- [ ] std::mem

### 🔩 OOP (or not exactly)

- [ ] Method calls (`a.iter()`)
- [ ] User-defined structs
- [ ] User-defined contracts and implementations

### 📢 Errors

- [x] Error reporter (Rust like)
- [x] Basic name resolution and type checking errors
- [ ] User-defined Error Throwing

## Installation

RuneWay does not provide a usable compiler binary yet. To build and test the current workspace:

```shell
cargo build --workspace
cargo test --workspace
```

## 📖 Documentation

The codebase and this README currently serve as the primary project documentation.

<!--Temporary removed

## 🤝 Contributing

We welcome your issues about bugs, ideas, or suggestions.
-->

## 📬 Contact me

- Telegram: [@voiris](https://t.me/voiris)
- Telegram Channel: [Потёмки Войриса](https://t.me/voiris_shadow)
- Github: [github.com/Voiris](https://github.com/Voiris)
- Carrd: [Voiris](https://voiris.carrd.co/)

## 📄 License

MIT License. See [LICENSE](LICENSE) for details.
