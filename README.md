# 🧪 ᚱuneWay

*RuneWay* is a programming language written in Rust, with syntax inspired by Rust, Python, and JavaScript.

## 🔧 Features (WIP)

### ✨ Language Features

- [x] Comments (`//`, `/* */`)
- [ ] Variables and Types (`let`, `int`, `float`, `string`, `bool`, `null`)
- [ ] Binary and Unary Operations
- [ ] String Interpolation (`f"..."`, `r"..."`, `fr"..."`)
- [ ] Logic and Loops (`if`, `while`, `for`, `loop`)
- [ ] Functions (`act name() {}`)
- [ ] Static type annotations (`x: int`)
- [ ] Static type annotations in functions (`act f(x: int) -> int`)
- [ ] Constants (`const PI: float = 3.14`)
- [ ] Generic types (`SomeType<T, E, 3>`)

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
 HIR lowering + name resolution ──► HIR
   │
   ▼
 Semantic / type checking
   │
   ▼
 MIR lowering ──► MIR
   │
   ▼
 Cranelift IR codegen ──► native binary
```

- [x] Lexer (out: Tokens)
- [ ] Parser (out: AST. Based on: Recursive Descent + Pratt) — *in progress*
- [ ] HIR (out: HIR) — *in progress*
- [ ] Name resolution
- [ ] Semantic / type checker
- [ ] MIR (out: MIR)
- [ ] Cranelift IR codegen (out: native binary)

### 🪵 Built-ins

- [ ] Native console out (`print`, `println`)
- [ ] Native console in (`input`)
- [ ] Primitive Types (`int`, `string`, `bool`, ...)

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
- [ ] Contracts

### 📢 Errors

- [x] Error reporter (Rust like)
- [ ] User-defined Error Throwing

## Installation

To be continued...

## 📖 Documentation

To be continued...

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
