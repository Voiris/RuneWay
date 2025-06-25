mod runeway;

use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;

use runeway::core::lexer::tokenize;
use runeway::core::parser::ParserProcess;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    // let x = 2; act foo(a, b) {return (a + b) * x;}; let bar = foo(3, 5); print(\"Result: \" + str(bar));
    // let x = 0; while x < 1000 { if x == 500 { break; } else { x = x + 1; } } print(x);
    let code = fs::read_to_string(Path::new(args.get(0).unwrap().as_str())).unwrap();

    println!("RuneWay V2 TEST:");
    println!("------------------------------------CODE------------------------------------\n{}", code);
    
    // Tokenizing
    println!("---------------------------------TOKENIZING---------------------------------");
    let t1 = SystemTime::now();
    let tokens = tokenize(code);
    let t2 = SystemTime::now();
    println!("Result: {:?}", tokens);
    println!("Time-spent: {:?}", t2.duration_since(t1).unwrap());
    // Parsing AST structure
    println!("--------------------------------AST STRUCTURE-------------------------------");
    let t1 = SystemTime::now();
    let parsed_ast = ParserProcess::new(tokens).parse_full();
    let t2 = SystemTime::now();
    
    println!("Result: {:?}", parsed_ast);
    println!("Time-spent: {:?}", t2.duration_since(t1).unwrap());
}
