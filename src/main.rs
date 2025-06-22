mod runeway;

use runeway::lexer;
use runeway::parser;

fn main() {
    // let x = 2; act foo(a, b) {return (a + b) * x;}; let bar = foo(3, 5); print(\"Result: \" + str(bar));
    let code = "let x; print(x);";

    println!("\nCode: {}", code);

    let tokens = lexer::tokenize(code);

    println!("\nTokens: {:?}\n", tokens);

    let ast = parser::parse(tokens);

    println!("\nAST-structure: {:?}\n\nRun:", ast);

    ast.run()
}
