extern crate core;

mod runeway;

use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;

use runeway::executor::runtime::types::RNWObject;
use crate::runeway::core::parser::parse_code;
use crate::runeway::executor::interpreter::ASTInterpreter;
use crate::runeway::executor::runtime::environment::Environment;
use crate::runeway::builtins;
use crate::runeway::stdlibs;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let code = fs::read_to_string(Path::new(args.get(0).unwrap().as_str())).unwrap();

    println!("------------------------------------CODE------------------------------------\n{}", code);

    println!("\n\n-----------------------------------RUNEWAY----------------------------------");
    println!("-----------------------------------PARSING----------------------------------");
    let t1 = SystemTime::now();
    let parsed_ast = match parse_code(code) {
        Ok(x) => x,
        Err(e) => panic!("{}", e)
    };
    let t2 = SystemTime::now();
    println!("Result:\n{:?}", parsed_ast);
    println!("Time-spent: {:?}", t2.duration_since(t1).unwrap());
    
    println!("------------------------------------RUN-------------------------------------");

    let t1 = SystemTime::now();
    let env = Environment::new_global();
    builtins::prelude(env.clone());
    stdlibs::prelude();
    ASTInterpreter::execute_many(env.clone(), parsed_ast);
    ASTInterpreter::entry(env.clone(), "main");
    let t2 = SystemTime::now();

    println!("\n\nTime-spent: {:?}", t2.duration_since(t1).unwrap());
}
