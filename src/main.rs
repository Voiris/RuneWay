extern crate core;

mod runeway;

use std::{env, fs};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::SystemTime;

use runeway::runtime::types::RNWObject;
use crate::runeway::core::parser::{parse_code, ParsedCode};
use crate::runeway::executor::interpreter::ASTInterpreter;
use runeway::runtime::environment::Environment;
use crate::runeway::builtins;
use crate::runeway::stdlibs;

fn execute(working_dir: &Path, parsed_code: ParsedCode, filename: &str, code: String) {
    let env = Environment::new_global();
    builtins::prelude(env.clone());
    stdlibs::prelude();
    match ASTInterpreter::execute_many(env.clone(), parsed_code.ast, working_dir, filename, &code) {
        Ok(()) => (),
        Err(e) => {
            e.report();
            exit(1)
        }
    }
    match ASTInterpreter::entry(env.clone(), "main") {
        Ok(()) => (),
        Err(e) => {
            e.report();
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
        (file_name.to_str().unwrap(), fs::read_to_string(&path).unwrap())
    } else {
        panic!("No filename found.");
    };
    let working_dir = path.as_path().parent().unwrap();

    println!("------------------------------------CODE------------------------------------\n{}", code);
    println!("\n\n-----------------------------------RUNEWAY----------------------------------");
    println!("-----------------------------------PARSING----------------------------------");
    let t1 = SystemTime::now();
    let parsed_code = match parse_code(code.clone()) {
        Ok(x) => x,
        Err(e) => {
            e.with_code_base(filename, code).report();
            exit(1)
        }
    };
    let t2 = SystemTime::now();
    /*
    println!("Parsed: {}", parsed_code.ast.iter()
        .map(|x| format!("{:?}", x)).collect::<Vec<String>>().join("\n"));
     */
    println!("Time-spent: {:?}", t2.duration_since(t1).unwrap());

    match args.get(0).unwrap().as_str() {
        "run" => {
            println!("------------------------------------RUN-------------------------------------");

            let t1 = SystemTime::now();
            if args.contains(&"--benchmark".to_string()) {
                use std::time::Instant;

                let iterations = 100;
                let mut times = vec![];

                for _ in 0..iterations {
                    let start = Instant::now();
                    execute(working_dir, parsed_code.clone(), filename, code.clone()); // замените на ваш метод
                    times.push(start.elapsed().as_secs_f64() * 1000.0); // ms
                }

                let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = times.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                let mean = times.iter().sum::<f64>() / times.len() as f64;
                let stddev = (times.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / times.len() as f64).sqrt();

                println!("min: {min:.4}ms, max: {max:.4}ms, mean: {mean:.4}ms, stddev: {stddev:.4}ms");
            } else {
                execute(working_dir, parsed_code.clone(), filename, code.clone());
            }

            let t2 = SystemTime::now();

            println!("\n\nTime-spent: {:?}", t2.duration_since(t1).unwrap());
        }
        "build" => {

        }
        _ => unimplemented!(),
    }
}
