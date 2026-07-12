# Testing

Run the full workspace test suite before changing shared compiler crates:

```shell
cargo test --workspace
```

For frontend work, keep lexer, parser, lowering, resolution, and type-checking tests close to the
crate that owns the behavior. This keeps failures local and makes regressions easier to diagnose.
