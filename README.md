# 🧪 ᚱuneWay

*RuneWay* is a programming language written in Rust, with syntax inspired by Rust, Python, and JavaScript.

## 🔧 Features (WIP)

### ✨ Language Features
- [x] Comments (`//`, `/* */`)
- [x] Variables and Types (`let`, `int`, `float`, `string`, `bool`, `null`)
- [x] Binary and Unary Operations
- [x] String Interpolation (`f"..."`, `r"..."`)
- [x] Logic and Loops (`if`, `while`, `for`)
- [x] Functions (`act name() {}`)
- [ ] Static type annotations (`x: int`)

### 🧱 Architecture
- [x] AST-based Interpreter
- [ ] Semantic Checker
- [ ] AST Optimizer
- [ ] Bytecode compiler & VM

### 🪵 Built-ins
- [x] Native console out (`print`, `println`)
- [ ] Native console in (`input`)
- [ ] Native object id (`id()`, like in Python)
- [ ] Native type introspection (`type()`, like in Python)
- [ ] Primitive Types (`int`, `string`, `bool`, ...)

### 🧩 Standard Library
- [x] Module System (Imports)
- [x] std::buffered (demo)
- [ ] std::random
- [ ] std::math
- [ ] std::files
- [ ] std::net
- [ ] std::time

### 🔩 OOP
- [x] Method calls (`a.to_string()`)
- [ ] Custom classes

### 📢 Errors
- [ ] Line & Column Positioning
- [ ] SyntaxError, TypeError, ValueError (Python like)
- [ ] Error reporter (Rust like)
- [ ] User-defined Error Throwing


## Installation

### From source
First, clone the repository:
```bash
git clone https://github.com/username/runeway.git
cd runeway
```

#### Local install
Then, build the project:
```bash
cargo build --release
```
Usage:
```bash
./target/release/runeway examples/hello_world.rnw
```

#### Global install (Optional)
If you want to use it globally (e.g. as a CLI command):
```bash
cargo install --path .
```
Then you can just run:
```bash
runeway examples/hello_world.rnw
```

## 📖 Documentation

To be continued...


## 🤝 Contributing

We welcome your issues about bugs, ideas, or suggestions.


## 📬 Contact me

- Telegram: [@voiris](https://t.me/voiris)
- Telegram Channel: [Потёмки Войриса](https://t.me/voiris_shadow)
- Github: [github.com/Voiris](https://github.com/Voiris)


## 📄 License

MIT License. See [LICENSE](./LICENSE) for details.
