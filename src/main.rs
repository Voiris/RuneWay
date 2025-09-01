extern crate core;

mod runeway;

use std::path::{Path, PathBuf};
use std::process::{exit, Termination};
use std::time::SystemTime;
use std::{env, fs};

use crate::runeway::builtins;
use crate::runeway::core::parser::{parse_code, ParsedCode};
use crate::runeway::executor::interpreter::ASTInterpreter;
use crate::runeway::stdlibs;
use runeway::runtime::environment::Environment;
use crate::runeway::compiler::bytecode::BytecodeCompiler;
use crate::runeway::executor::vm::VM;

fn execute(working_dir: &Path, parsed_code: ParsedCode, filename: String, code: String) {
    let env = Environment::new_builtins_global();
    builtins::prelude(env.clone());
    stdlibs::prelude();
    match ASTInterpreter::execute_many(
        env.clone(),
        parsed_code.ast,
        working_dir,
        filename.clone(),
        &code,
    ) {
        Ok(()) => (),
        Err(e) => {
            e.with_source(filename, code).report();
            exit(1)
        }
    }
    match ASTInterpreter::entry(env.clone(), "main") {
        Ok(()) => (),
        Err(e) => {
            e.with_source(filename, code).report();
            exit(1)
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let path = PathBuf::from(args.get(1).unwrap().as_str());

    let path = if !path.is_absolute() {
        env::current_dir().unwrap().join(path)
    } else {
        path
    };

    let (filename, code) = if let Some(file_name) = path.file_name() {
        (
            file_name.to_str().unwrap(),
            fs::read_to_string(&path).unwrap(),
        )
    } else {
        panic!("No filename found.");
    };
    let working_dir = path.as_path().parent().unwrap();

    println!(
        "------------------------------------CODE------------------------------------\n{}",
        code
    );
    println!("\n\n-----------------------------------RUNEWAY----------------------------------");

    match args.get(0).unwrap().as_str() {
        "run" => {
            println!("-----------------------------------PARSING----------------------------------");
            let t1 = SystemTime::now();
            let parsed_code = match parse_code(filename.to_string(), code.clone()) {
                Ok(x) => x,
                Err(e) => {
                    e.with_source(filename.to_string(), code).report();
                    exit(1)
                }
            };
            let t2 = SystemTime::now();
            /*
            println!("Parsed: {}", parsed_code.ast.iter()
                .map(|x| format!("{:?}", x)).collect::<Vec<String>>().join("\n"));
             */
            println!("Time-spent: {:?}", t2.duration_since(t1).unwrap());

            println!(
                "------------------------------------RUN-------------------------------------"
            );

            let t1 = SystemTime::now();
            if args.contains(&"--benchmark".to_string()) {
                use std::time::Instant;

                let iterations = 100;
                let mut times = vec![];

                for _ in 0..iterations {
                    let start = Instant::now();
                    execute(
                        working_dir,
                        parsed_code.clone(),
                        filename.to_owned(),
                        code.clone(),
                    ); // замените на ваш метод
                    times.push(start.elapsed().as_secs_f64() * 1000.0); // ms
                }

                let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let mean = times.iter().sum::<f64>() / times.len() as f64;
                let stddev = (times.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                    / times.len() as f64)
                    .sqrt();

                println!(
                    "min: {min:.4}ms, max: {max:.4}ms, mean: {mean:.4}ms, stddev: {stddev:.4}ms"
                );
            } else {
                execute(
                    working_dir,
                    parsed_code.clone(),
                    filename.to_owned(),
                    code.clone(),
                );
            }

            let t2 = SystemTime::now();

            println!("\n\nTime-spent: {:?}", t2.duration_since(t1).unwrap());
        }
        "build" => {
            println!("---------------------------------COMPILING----------------------------------");
            let compiled = match BytecodeCompiler::compile(working_dir.to_path_buf(), filename.to_string(), "main") {
                Ok(x) => x,
                Err(e) => {
                    e.report();
                    exit(1);
                }
            };
            println!("{:?}", compiled);
            println!("------------------------------------RUN-------------------------------------");
            let vm = VM::new(compiled);

            let t1 = SystemTime::now();
            VM::run(&vm).report();

            let t2 = SystemTime::now();

            println!("\n\nTime-spent: {:?}", t2.duration_since(t1).unwrap());
        }
        _ => unimplemented!(),
    }
}
