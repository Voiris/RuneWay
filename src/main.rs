mod runeway;

use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;
use runeway::lexer;
use runeway::parser;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // let x = 2; act foo(a, b) {return (a + b) * x;}; let bar = foo(3, 5); print(\"Result: \" + str(bar));
    // let x = 0; while x < 1000 { if x == 500 { break; } else { x = x + 1; } } print(x);
    let code = fs::read_to_string(Path::new(args.get(0).unwrap().as_str())).unwrap();

    println!("\nCode: {}", code);

    let tokens = lexer::tokenize(code);

    println!("\nTokens: {:?}\n", tokens);

    let ast = parser::parse(tokens);

    println!("\nAST-structure: {:?}\n\nRun:", ast);

    let start = SystemTime::now();

    ast.run();

    let end = SystemTime::now();

    println!("\n\nTime-elapsed is: {:?}", end.duration_since(start).unwrap());
}
