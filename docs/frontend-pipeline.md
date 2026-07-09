# Frontend Pipeline

The current RuneWay frontend pipeline is intentionally split into small stages:

1. Lex source text into tokens.
2. Parse tokens into AST nodes.
3. Lower AST nodes into HIR.
4. Resolve names against local, item, and built-in scopes.
5. Type-check expressions, statements, and function signatures.

This keeps syntax handling separate from semantic validation, which makes each stage easier to test
and extend while the language is still changing.
