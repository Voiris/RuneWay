use std::any::TypeId;
use std::rc::Rc;
use crate::runeway::builtins::types::{RNWBoolean, RNWFloat, RNWInteger, RNWIterator, RNWList, RNWNullType, RNWString};
use crate::runeway::core::ast::{
    statement::{Statement, ImportItem},
    expression::{Expr, FStringExpr}
};
use crate::runeway::executor::runtime::controlflow::ControlFlow;
use crate::runeway::executor::runtime::environment::{EnvRef, Environment};
use crate::runeway::executor::runtime::libraries::{load_library, RNWModule};
use crate::runeway::executor::runtime::types::{RNWObject, RNWObjectRef, RNWRegisteredNativeFunction};

pub struct ASTInterpreter;

impl ASTInterpreter {
    pub fn execute(root_env: EnvRef, stmt: Statement) {
        Self::execute_local(root_env.clone(), stmt, None, true);
    }

    pub fn execute_many(root_env: EnvRef, stmts: Vec<Statement>) {
        for stmt in stmts {
            Self::execute_local(root_env.clone(), stmt, None, true);
        }
    }

    fn execute_many_local(root_env: EnvRef, stmts: Vec<Statement>, parent_env: Option<EnvRef>,
                          is_top_level: bool) -> ControlFlow {
        for stmt in stmts {
            match Self::execute_local(root_env.clone(), stmt, parent_env.clone(), is_top_level) {
                ControlFlow::Return(val) => return ControlFlow::Return(val),
                _ => ()
            }
        }
        ControlFlow::Nothing
    }

    //noinspection DuplicatedCode
    fn execute_local(root_env: EnvRef, stmt: Statement, parent_env: Option<EnvRef>, is_top_level: bool) -> ControlFlow {
        let env = if let Some(_parent_env) = parent_env {
            _parent_env
        } else {
            Rc::clone(&root_env)
        };

        if !is_top_level {
            match stmt {
                Statement::Expr(expr) => {
                    Self::evaluate(expr, Rc::clone(&env));
                    ControlFlow::Nothing
                }
                Statement::Let { name, value } => {
                    let val = Self::evaluate(value, Rc::clone(&env));
                    env.borrow_mut().define_variable(name.clone(), val);
                    ControlFlow::Nothing
                }
                Statement::Assign { name, value } => {
                    let val = Self::evaluate(value, Rc::clone(&env));
                    match env.borrow_mut().assign_variable(&name, val) {
                        Ok(()) => ControlFlow::Nothing,
                        Err(e) => panic!("{}", e),
                    }
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_branch
                } => {
                    let mut controlflow = ControlFlow::Nothing;
                    let condition_value = Self::evaluate(condition, Rc::clone(&env));

                    let branch = match condition_value.borrow().value().downcast_ref::<bool>() {
                        Some(true) => Some(then_branch),
                        Some(false) => else_branch,
                        None => panic!("Condition must be a boolean")
                    };

                    if let Some(statements) = branch {
                        for stmt in statements {
                            if (*stmt).is(&Statement::Break) {
                                controlflow = ControlFlow::Break;
                                break;
                            } else if (*stmt).is(&Statement::Continue) {
                                controlflow = ControlFlow::Continue;
                                break;
                            }
                            match Self::execute_local(root_env.clone(), *stmt, Some(Rc::clone(&env)), false) {
                                ControlFlow::Return(value) =>
                                    controlflow = ControlFlow::Return(value),
                                _ => {}
                            }
                        }
                    }
                    controlflow
                }
                Statement::While { condition, body } => {
                    'outer: loop {
                        let condition_value = Self::evaluate(condition.clone(), Rc::clone(&env));

                        match condition_value.borrow().value().downcast_ref::<bool>() {
                            Some(true) => {
                                for stmt in &body {
                                    let cf = Self::execute_local(
                                        root_env.clone(), *stmt.clone(), Some(Rc::clone(&env)), false);
                                    match cf {
                                        ControlFlow::Break => break 'outer,
                                        ControlFlow::Return(value) => {
                                            return ControlFlow::Return(value);
                                        }
                                        ControlFlow::Continue => {
                                            continue 'outer;
                                        }
                                        _ => ()
                                    }
                                }
                            }
                            Some(false) => break 'outer,
                            None => panic!("Condition must be a boolean")
                        }
                    }
                    ControlFlow::Nothing
                }
                Statement::Return(expr) => ControlFlow::Return(Self::evaluate(expr, Rc::clone(&env))),
                Statement::Break => ControlFlow::Break,
                Statement::Continue => ControlFlow::Continue,
                Statement::Act { name, parameters, body } => {
                    Self::register_act(root_env.clone(), env.clone(), name, parameters, body);

                    ControlFlow::Nothing
                }
                Statement::For { variable, iterable, body } => {
                    let iterable = Self::evaluate(iterable, Rc::clone(&env));
                    let mut iterator = match iterable.borrow_mut().as_any_mut().downcast_ref::<RNWIterator>() {
                        Some(iterator) => iterator,
                        None => panic!("Iterable must be a iterator")
                    }.clone();
                    'outer: loop {
                        let value = iterator.next();

                        if RNWNullType::is_type_equals(&value) {
                            break;
                        }

                        let iteration_env = Environment::new_enclosed(env.clone());
                        let mut borrow = iteration_env.borrow_mut();
                        borrow.define_variable(variable.clone(), Rc::clone(&value));
                        drop(borrow);

                        for stmt in &body {
                            match Self::execute_local(iteration_env.clone(), *stmt.clone(), None, false) {
                                ControlFlow::Return(value) => return ControlFlow::Return(value),
                                ControlFlow::Continue => continue 'outer,
                                ControlFlow::Break => break 'outer,
                                _ => ()
                            }
                        }
                    }

                    ControlFlow::Nothing
                },
                _ => panic!("Not implemented yet. Statement: {:?}", stmt),
            }
        } else {
            match stmt {
                Statement::Act { name, parameters, body } => {
                    Self::register_act(root_env.clone(), env.clone(), name, parameters, body);

                    ControlFlow::Nothing
                }
                Statement::Import { path, item } => {
                    Self::execute_import(env.clone(), path, item)
                }
                statement => {
                    let s = format!("{:?}", statement);
                    panic!(
                        "Statement {} forbidden on top-level",
                        s.split(&[' ', '{', '('][..]).next().unwrap_or(&s)
                    )
                }
            }
        }
    }

    fn execute_import(env: EnvRef, path: String, item: ImportItem) -> ControlFlow {
        let library_env = load_library(path.clone());

        let mut borrow = env.borrow_mut();
        match item {
            ImportItem::Alias(alias) => {
                borrow.define_variable(alias, RNWModule::new(path.clone(), library_env.clone()))
            }
            ImportItem::All => {
                borrow.merge(library_env.clone());
            }
            ImportItem::Selective(selective) => {
                let library_borrow = library_env.borrow();
                for symbol in selective.iter().cloned() {
                    let alias = symbol.alias.unwrap_or_else(|| symbol.original.clone());
                    if let Ok(function) = library_borrow.get_function(&symbol.original) {
                        let mut new_function = (*function).clone();
                        new_function.name = alias;
                        borrow.define_function(function.clone());
                    } else if let Ok(variable) = library_borrow.get_variable(&symbol.original) {
                        borrow.define_variable(alias, variable.clone());
                    } else {
                        panic!("Cannot import `{}` from `{}`", symbol.original, path);
                    }
                }
            }
        }

        ControlFlow::Nothing
    }

    fn register_act(root_env: EnvRef, env: EnvRef, name: String,
                    parameters: Vec<String>, body: Vec<Box<Statement>>) {
        let unboxed_body: Vec<Statement> = body.into_iter()
        .map(|stmt| (*stmt).clone())
        .collect();

        // Клонируем параметры для замыкания, чтобы их можно было использовать внутри
        let _parameters = parameters.clone();

        // Клонируем глобальную среду для замыкания, чтобы их можно было использовать внутри
        let _root_env = Rc::clone(&root_env);

        // Создаём замыкание, которое реализует Fn (без перемещения unboxed_body)
        let func = Rc::new(move |_args: &[RNWObjectRef]| {
            let __env = Environment::new_enclosed(_root_env.clone());
            let mut borrow = __env.borrow_mut();

            // Связываем параметры с аргументами
            for (_param, _arg) in _parameters.iter().zip(_args.iter()) {
                borrow.define_variable(_param.clone(), Rc::clone(&_arg));
            }

            drop(borrow);

            // Клонируем тело при каждом вызове, чтобы не перемещать из замыкания
            let body_clone = unboxed_body.clone();


            // Выполняем тело
            match ASTInterpreter::execute_many_local(Rc::clone(&_root_env), body_clone,
                                                     Some(__env.clone()), false) {
                ControlFlow::Return(value) => value,
                ControlFlow::Nothing => RNWNullType::new(),
                _ => unreachable!(),
            }
        });

        // Составляем вектор типов параметров (здесь универсальный тип dyn RNWObject)
        let params: Vec<TypeId> = parameters.iter()
            .map(|_| TypeId::of::<dyn RNWObject>())
            .collect();

        // Определяем функцию в окружении с вложенной средой
        let mut borrow = env.borrow_mut();
        borrow.define_function(
            RNWRegisteredNativeFunction::new(
                name,
                func,
                params
            ).into()
        );
        drop(borrow);
    }

    fn evaluate(expr: Expr, env: EnvRef) -> RNWObjectRef {
        match expr {
            Expr::Null => RNWNullType::new(),
            Expr::String(s) => RNWString::new(s),
            Expr::Integer(i) => RNWInteger::new(i),
            Expr::Float(f) => RNWFloat::new(f),
            Expr::Boolean(b) => RNWBoolean::new(b),
            Expr::List(vec) => {
                let mut list: Vec<RNWObjectRef> = Vec::new();
                for item in vec.iter() {
                    list.push(Self::evaluate(*item.clone(), Rc::clone(&env)));
                }
                RNWList::new(&list)
            }
            Expr::Iterator { start, end, step } => {
                let start: i64 = *Self::evaluate(*start, Rc::clone(&env)).borrow().value().downcast_ref::<i64>().unwrap();
                let end: i64 = *Self::evaluate(*end, Rc::clone(&env)).borrow().value().downcast_ref::<i64>().unwrap();
                let step: i64 = match step {
                    Some(int) => *Self::evaluate(*int, Rc::clone(&env)).borrow()
                        .value().downcast_ref::<i64>().unwrap(),
                    None => 1
                };
                RNWIterator::from_range(
                    start, end, step
                )
            }
            Expr::Call { ref callee, ref arguments } => {
                match *callee.clone() {
                    Expr::Variable(name) => {
                        match env.borrow().get_function(&name) {
                            Ok(func) => {
                                func.call(arguments.iter().map(
                                    |arg| Self::evaluate(arg.clone(), Rc::clone(&env))
                                ).collect::<Vec<RNWObjectRef>>().as_slice())
                            }
                            Err(e) => panic!("{}", e),
                        }
                    }
                    Expr::GetAttr { object, field } => {
                        let obj_val = Self::evaluate(*object, Rc::clone(&env));
                        let obj_borrow = obj_val.borrow();

                        let eval_args = || arguments.iter()
                            .map(|arg| Rc::clone(&Self::evaluate(arg.clone(), Rc::clone(&env))))
                            .collect::<Vec<RNWObjectRef>>();

                        if let Some(method) = obj_borrow.method(&field) {
                            let args_eval = eval_args();

                            method.call(Rc::clone(&obj_val), args_eval.as_slice())
                        } else if let Some(function) = obj_borrow.function(&field) {
                            let args_eval = eval_args();

                            function.call(args_eval.as_slice())
                        } else {
                            panic!("Method `{}` not found in `{:?}`", field, obj_borrow.as_object());
                        }
                    }
                    _ => panic!("Unexpected callee. Expr: {:?}", expr.clone()),
                }
            }
            Expr::Variable(name) => {
                match env.borrow().get_variable(&name) {
                    Ok(val) => Rc::clone(&val),
                    Err(e) => panic!("{}", e),
                }
            }
            Expr::BinaryOperation { left_operand, right_operand, operator } => {
                let left = Self::evaluate(*left_operand, Rc::clone(&env));
                let right = Self::evaluate(*right_operand, Rc::clone(&env));

                let result = left.borrow().binary_operation(right.clone(), operator.clone());

                match result {
                    Some(val) => val,
                    None => panic!("Binary operation `{} {} {}` is not supported",
                                   left.borrow().type_name(),
                                   operator.display(),
                                   right.borrow().type_name()
                    )
                }
            }
            Expr::UnaryOperation { operand, operator } => {
                let operand = Self::evaluate(*operand, Rc::clone(&env));

                let result = operand.borrow().unary_operation(operator.clone());

                match result {
                    Some(val) => val,
                    None => panic!("Unary operation `{}{}` is not supported",
                                   operator.display(),
                                   operand.borrow().type_name()
                    )
                }
            }
            Expr::GetAttr { object, field } => {
                let obj_val = Self::evaluate(*object, Rc::clone(&env));
                let obj_borrow = obj_val.borrow();

                match obj_borrow.field(&field) {
                    Some(val) => Rc::clone(&val),
                    None => panic!("Not founded field `{:?}` in {:?}", field, obj_borrow.as_object()),
                }
            }
            Expr::Slice { object, index } => {
                let obj_val = Self::evaluate(*object, Rc::clone(&env));
                let index_val = Self::evaluate(*index, Rc::clone(&env));
                if let Some(method) = obj_val.borrow().method("slice") {
                    method.call(Rc::clone(&obj_val), &[Rc::clone(&index_val)])
                } else {
                    panic!("Method `slice` not found in type <{}>", obj_val.borrow().type_name())
                }
            }
            Expr::FString(f_string_items) => {
                let mut string = String::new();
                for f_string_item in f_string_items.iter() {
                    match f_string_item {
                        FStringExpr::String(s) => string.push_str(s),
                        FStringExpr::Expr(expr) => {
                            let value = Self::evaluate(expr.clone(), Rc::clone(&env));
                            let borrowed = value.borrow();

                            if let Some(val) = borrowed.value().downcast_ref::<String>() {
                                string.push_str(val);
                                continue;
                            }

                            drop(borrowed);

                            let method = value.borrow().method("to_string").unwrap_or_else(|| {
                                panic!(
                                    "Method `to_string` not found in type <{}>",
                                    value.borrow().type_name()
                                )
                            });

                            let val = method.call(Rc::clone(&value), &[]);
                            let borrowed = val.borrow();
                            let str_val = borrowed.value().downcast_ref::<String>().unwrap_or_else(|| {
                                panic!(
                                    "`to_string` must return a string, got <{}>",
                                    borrowed.type_name()
                                )
                            });

                            string.push_str(str_val);
                        }
                    }
                }
                RNWString::new(string)
            }
            _ => panic!("Not implemented yet. Expression: {:?}", expr),
        }
    }

    pub fn entry(root_env: EnvRef, entry_function_name: &'static str) {
        match root_env.borrow().get_function(entry_function_name) {
            Ok(function) => {
                let result = function.call(&[]);
                let result = if RNWNullType::is_type_equals(&result) {
                    0
                } else {
                    match result.borrow().as_any().downcast_ref::<RNWInteger>() {
                        Some(val) => val.value,
                        None => panic!(
                            "Entry `{}` exit code must be <integer> or <null>. Got: <{}>",
                            entry_function_name, result.borrow().type_name()),
                    }
                };
                println!("\n\nProcess finished with exit code {}", result);
            }
            Err(e) => panic!("{}", e),
        }
    }
}