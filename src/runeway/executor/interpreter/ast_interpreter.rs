use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Range;
use std::path::Path;
use std::rc::Rc;
use ariadne::{Color, Fmt};
use crate::runeway::builtins::types::{RNWBoolean, RNWDict, RNWFloat, RNWInteger, RNWIterator, RNWList, RNWNullType, RNWString, RNWTuple};
use crate::runeway::core::ast::{
    statement::{Statement, ImportItem},
    expression::{Expr, FStringExpr}
};
use crate::runeway::core::ast::expression::SpannedExpr;
use crate::runeway::core::ast::statement::SpannedStatement;
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::controlflow::ControlFlow;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::{load_library, RNWModule};
use crate::runeway::runtime::types::{cast_to, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction};

pub struct ASTInterpreter;

impl ASTInterpreter {
    pub fn execute(root_env: EnvRef, stmt: SpannedStatement, working_dir: &Path,
                   filename: impl AsRef<str>, code: &String) -> RWResult<()> {
        Self::execute_local(root_env.clone(), stmt, None, true,
                            working_dir, filename.as_ref(), code)?;
        Ok(())
    }

    pub fn execute_many(root_env: EnvRef, stmts: Vec<SpannedStatement>, working_dir: &Path,
                        filename: impl AsRef<str>, code: &String) -> RWResult<()> {
        for stmt in stmts {
            Self::execute_local(root_env.clone(), stmt, None, true,
                                working_dir, filename.as_ref(), code)
                .map_err(
                    |e| e.with_code_base(filename.as_ref(), code)
                )?;
        }
        Ok(())
    }

    fn execute_many_local(root_env: EnvRef, stmts: Vec<SpannedStatement>, parent_env: Option<EnvRef>,
                          is_top_level: bool, working_dir: &Path,
                          filename: &str, code: &String) -> RWResult<ControlFlow> {
        for stmt in stmts {
            match Self::execute_local(root_env.clone(), stmt, parent_env.clone(),
                                      is_top_level, working_dir, filename, code) {
                Ok(ControlFlow::Return(r)) => return Ok(ControlFlow::Return(r)),
                Ok(_) => (),
                Err(e) => {
                    return Err(e.with_code_base(filename, code))
                }
            }
        }
        Ok(ControlFlow::Nothing)
    }

    //noinspection DuplicatedCode
    fn execute_local(root_env: EnvRef, stmt: SpannedStatement, parent_env: Option<EnvRef>,
                     is_top_level: bool, working_dir: &Path,
                     filename: &str, code: &String) -> RWResult<ControlFlow> {
        let env = if let Some(_parent_env) = parent_env {
            _parent_env
        } else {
            Rc::clone(&root_env)
        };

        if !is_top_level {
            let result = match stmt.node {
                Statement::Expr(expr) => {
                    Self::evaluate(expr, Rc::clone(&env))?;
                    ControlFlow::Nothing
                }
                Statement::Let { name, value } => {
                    let val = Self::evaluate(value, Rc::clone(&env))?;
                    env.borrow_mut().define_variable(name.clone(), val);
                    ControlFlow::Nothing
                }
                Statement::Assign { name, value } => {
                    let val = Self::evaluate(value, Rc::clone(&env))?;
                    env.borrow_mut().assign_variable(&name, val)?;
                    ControlFlow::Nothing
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_branch
                } => {
                    let mut controlflow = ControlFlow::Nothing;

                    let branch = {
                        let condition_value = Self::evaluate(condition.clone(), Rc::clone(&env))?;

                        let casted_condition_value = cast_to::<RNWBoolean>(&condition_value)?;

                        let cond = *casted_condition_value.borrow().value().downcast_ref::<bool>().unwrap();

                        if cond {
                            Some(then_branch)
                        } else {
                            else_branch
                        }
                    };

                    if let Some(statements) = branch {
                        for stmt in statements {
                            if (*stmt).node.is(&Statement::Break) {
                                controlflow = ControlFlow::Break;
                                break;
                            } else if (*stmt).node.is(&Statement::Continue) {
                                controlflow = ControlFlow::Continue;
                                break;
                            }
                            match Self::execute_local(root_env.clone(), *stmt, Some(Rc::clone(&env)),
                                                      false, working_dir, filename, code)? {
                                ControlFlow::Return(value) =>
                                    controlflow = ControlFlow::Return(value),
                                _ => ()
                            }
                        }
                    }
                    controlflow
                }
                Statement::While { condition, body } => {
                    'outer: loop {
                        let condition_value =
                            Self::evaluate(condition.clone(), Rc::clone(&env))?;

                        let casted_condition_value = cast_to::<RNWBoolean>(&condition_value)?;

                        let cond = *casted_condition_value.borrow().value().downcast_ref::<bool>().unwrap();

                        if cond {
                            for stmt in &body {
                                let cf = Self::execute_local(
                                    root_env.clone(), *stmt.clone(), Some(Rc::clone(&env)),
                                    false, working_dir, filename, code)?;
                                match cf {
                                    ControlFlow::Break => break 'outer,
                                    ControlFlow::Return(value) => {
                                        return Ok(ControlFlow::Return(value));
                                    }
                                    ControlFlow::Continue => {
                                        continue 'outer;
                                    }
                                    _ => ()
                                }
                            }
                        } else {
                            break 'outer;
                        }
                    }
                    ControlFlow::Nothing
                }
                Statement::Return(expr) => ControlFlow::Return(Self::evaluate(expr, Rc::clone(&env))?),
                Statement::Break => ControlFlow::Break,
                Statement::Continue => ControlFlow::Continue,
                Statement::Act { name, parameters, body } => {
                    Self::register_act(env.clone(), name, parameters, body, working_dir, filename, code)?
                }
                Statement::For { variable, iterable, body } => {
                    let iterable_val = Self::evaluate(iterable.clone(), Rc::clone(&env))?;
                    let mut iterator = match iterable_val.borrow_mut().as_any_mut().downcast_ref::<RNWIterator>() {
                        Some(iterator) => iterator,
                        None => {
                            return Err(
                                RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                    .with_message("Iterable must be an iterator")
                                    .with_label("This must return an iterator", &iterable.span, None)
                                    .with_help(format!(
                                        "Try to add {} after iterable value",
                                        ".iter()".fg(Color::BrightGreen)
                                    ))
                            );
                        }
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
                            match Self::execute_local(iteration_env.clone(), *stmt.clone(), None,
                                                      false, working_dir, filename, code)? {
                                ControlFlow::Return(value) => return Ok(ControlFlow::Return(value)),
                                ControlFlow::Continue => continue 'outer,
                                ControlFlow::Break => break 'outer,
                                _ => ()
                            }
                        }
                    }

                    ControlFlow::Nothing
                },
                _ => panic!("Not implemented yet. Statement: {:?}", stmt),
            };
            Ok(result)
        } else {
            match stmt.node {
                Statement::Act { name, parameters, body } => {
                    Self::register_act(env.clone(), name, parameters, body, working_dir, filename, code)
                }
                Statement::Import { path, item } => {
                    Self::execute_import(env.clone(), path, item, &stmt.span, working_dir, filename, code)
                }
                _ => Err(
                    RuneWayError::new(RuneWayErrorKind::Syntax)
                        .with_message("Found forbidden statement on top-level")
                        .with_label("This statement is forbidden on top-level", &stmt.span, None)
                )
            }
        }
    }

    fn execute_import(env: EnvRef, path: String, item: ImportItem, span: &Range<usize>,
                      working_dir: &Path, filename: &str, code: &String) -> RWResult<ControlFlow> {
        let library_env = load_library(path.clone(), working_dir)?;

        let mut borrow = env.borrow_mut();
        match item {
            ImportItem::Alias(alias) => {
                borrow.define_variable(alias, RNWModule::new(
                    path.clone(),
                    library_env.clone()
                ))
            }
            ImportItem::All => {
                borrow.merge(library_env.clone())?
            }
            ImportItem::Selective(selective) => {
                let library_borrow = library_env.borrow();
                for symbol in selective.iter().cloned() {
                    let alias = symbol.alias.unwrap_or_else(|| symbol.original.clone());
                    if let Some(variable) = library_borrow.get_variable(&symbol.original) {
                        borrow.define_variable(alias, variable.clone())
                    } else {
                        let symbol_name = symbol.original.clone();
                        let mut err =
                            RuneWayError::new(RuneWayErrorKind::Runtime(Some("ImportError".to_string())))
                                .with_message(
                                    format!(
                                        "Symbol `{}` not found in module `{}`",
                                        (&symbol_name).fg(Color::BrightYellow),
                                        path.fg(Color::BrightYellow)
                                    )
                                )
                                .with_label("In this declaration", span, None);

                        if let Some(matches_name) = library_borrow.find_similar_strings(
                            symbol_name, 2
                        ).first() {
                            err = err.with_help(format!(
                                "Maybe you mean `{}`?",
                                matches_name.fg(Color::BrightGreen)
                            ));
                        }

                        return Err(err);
                    }
                }
            }
        }

        Ok(ControlFlow::Nothing)
    }

    fn register_act(env: EnvRef, name: String, parameters: Vec<String>,
                    body: Vec<Box<SpannedStatement>>, working_dir: &Path,
                    filename: &str, code: &String) -> RWResult<ControlFlow> {
        let unboxed_body: Vec<SpannedStatement> = body.into_iter()
        .map(|stmt| (*stmt).clone())
        .collect();

        // Клонируем параметры для замыкания, чтобы их можно было использовать внутри
        let _parameters = parameters.clone();

        // Клонируем глобальную среду для замыкания, чтобы их можно было использовать внутри
        let _env = Rc::clone(&env);

        let _working_dir = working_dir.to_path_buf();

        let _filename = filename.to_owned();
        let _code = code.to_owned();

        // Создаём замыкание, которое реализует Fn (без перемещения unboxed_body)
        let func = Rc::new(move |_args: &[RNWObjectRef]| {
            let __env = Environment::new_enclosed(_env.clone());
            let mut borrow = __env.borrow_mut();

            // Связываем параметры с аргументами
            for (_param, _arg) in _parameters.iter().zip(_args.iter()) {
                borrow.define_variable(_param.clone(), Rc::clone(&_arg));
            }

            drop(borrow);

            // Клонируем тело при каждом вызове, чтобы не перемещать из замыкания
            let body_clone = unboxed_body.clone();


            // Выполняем тело
            match ASTInterpreter::execute_many_local(Rc::clone(&_env), body_clone, Some(__env.clone()),
                                                     false, &_working_dir,
                                                     (&_filename).as_ref(), &_code)? {
                ControlFlow::Return(value) => Ok(value),
                ControlFlow::Nothing => Ok(RNWNullType::new()),
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

        Ok(ControlFlow::Nothing)
    }

    //noinspection DuplicatedCode
    fn evaluate(expr: SpannedExpr, env: EnvRef) -> RWResult<RNWObjectRef> {
        let result = match expr.node {
            Expr::Null => RNWNullType::new(),
            Expr::String(s) => RNWString::new(s),
            Expr::Integer(i) => RNWInteger::new(i),
            Expr::Float(f) => RNWFloat::new(f),
            Expr::Boolean(b) => RNWBoolean::new(b),
            Expr::List(vec) => {
                let mut list: Vec<RNWObjectRef> = Vec::new();
                for item in vec.iter() {
                    list.push(Self::evaluate(*item.clone(), Rc::clone(&env))?);
                }
                RNWList::new(&list)
            }
            Expr::Tuple(vec) => {
                let mut list: Vec<RNWObjectRef> = Vec::new();
                for item in vec.iter() {
                    list.push(Self::evaluate(*item.clone(), Rc::clone(&env))?);
                }
                RNWTuple::new(&list)
            }
            Expr::Iterator { start, end, step } => {
                let start: i64 = *(Self::evaluate(*start, Rc::clone(&env))?
                    .borrow().value().downcast_ref::<i64>().unwrap());
                let end: i64 = *(Self::evaluate(*end, Rc::clone(&env))?
                    .borrow().value().downcast_ref::<i64>().unwrap());
                let step: i64 = match step {
                    Some(int) => *(Self::evaluate(*int, Rc::clone(&env))?.borrow()
                        .value().downcast_ref::<i64>().unwrap()),
                    None => 1
                };
                RNWIterator::from_range(
                    start, end, step
                )
            }
            Expr::Call { ref callee, ref arguments } => {
                match &(*callee).node {
                    Expr::Variable(name) => {
                        let function = env.borrow().get_variable(&name);
                        match function {
                            Some(function) => {
                                let mut args = Vec::with_capacity(arguments.len());
                                for arg in arguments.iter().cloned() {
                                    args.push(Self::evaluate(arg, Rc::clone(&env))?);
                                }
                                match function.borrow().call(&args) {
                                    Some(r) => r?,
                                    None => return Err(
                                        RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                            .with_message(
                                                format!(
                                                    "<{}> object is not callable",
                                                    function.borrow().type_name()
                                                ))
                                            .with_label("Not callable", &(*callee).span, None)
                                    )
                                }
                            }
                            None => return Err(
                                RuneWayError::new(RuneWayErrorKind::Runtime(Some("NameError".to_string())))
                                    .with_message(format!("Variable '{}' not defined", name))
                                    .with_label("Not defined", &(*callee).span, None)
                            )
                        }
                    }
                    Expr::GetAttr { object, field } => {
                        let obj_val = Self::evaluate(*object.clone(), Rc::clone(&env))?;
                        let callable = {
                            let obj_borrow = obj_val.borrow();

                            if let Some(callable) = obj_borrow.field(&field) {
                                callable.clone()
                            } else {
                                return Err(
                                    RuneWayError::new(RuneWayErrorKind::Runtime(Some("AttributeError".to_string())))
                                        .with_message(format!(
                                            "Method `{}` not found in type: {}",
                                            (&field).fg(Color::BrightYellow),
                                            obj_borrow.type_name().fg(Color::BrightYellow)
                                        ))
                                        .with_label("Not found attribute", &(*callee).span, None)
                                );
                            }
                        };
                        let mut args_eval = Vec::with_capacity(arguments.len());
                        for arg in arguments.iter().cloned() {
                            let evaluated = Self::evaluate(arg, Rc::clone(&env))?;
                            args_eval.push(evaluated);
                        }

                        let mut args = if RNWModule::is_type_equals(obj_val.clone()) {
                            Vec::new()
                        } else {
                            vec![Rc::clone(&obj_val)]
                        };
                        args.extend(args_eval);

                        match callable.borrow().call(args.as_slice()) {
                            Some(r) => r?,
                            None => return Err(
                                RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                    .with_message(
                                        format!(
                                            "<{}> object is not callable",
                                            callable.borrow().type_name()
                                        ))
                            )
                        }
                    }
                    _ => panic!("Unexpected callee. Expr: {:?}", expr.clone()),
                }
            }
            Expr::Variable(name) => {
                match env.borrow().get_variable(&name) {
                    Some(var) => Rc::clone(&var),
                    None => return Err(
                        RuneWayError::new(RuneWayErrorKind::Runtime(Some("NameError".to_string())))
                            .with_message(format!("Variable '{}' not defined", name))
                            .with_label("Not defined variable", &expr.span, None)
                    )
                }
            }
            Expr::BinaryOperation { left_operand, right_operand, operator } => {
                let left = Self::evaluate(*left_operand.clone(), Rc::clone(&env))?;
                let right = Self::evaluate(*right_operand.clone(), Rc::clone(&env))?;

                let result = left.borrow().binary_operation(right.clone(), operator.clone());

                match result {
                    Some(val) => val,
                    None => {
                        return Err(
                            RuneWayError::new(RuneWayErrorKind::Runtime(Some("OperationError".to_string())))
                                .with_message(
                                    format!(
                                        "Not supported binary operation: `{} {} {}`",
                                        left.borrow().type_name().fg(Color::BrightYellow),
                                        operator.display().fg(Color::BrightRed),
                                        right.borrow().type_name().fg(Color::BrightYellow)
                                    )
                                )
                                .with_label("Not supported binary operation",
                                            &expr.span, None)
                        )
                    }
                }
            }
            Expr::UnaryOperation { operand, operator } => {
                let operand = Self::evaluate(*operand, Rc::clone(&env))?;

                let result = operand.borrow().unary_operation(operator.clone());

                match result {
                    Some(val) => val,
                    None => return Err(
                        RuneWayError::new(RuneWayErrorKind::Runtime(Some("OperationError".to_string())))
                            .with_message(
                                format!(
                                    "Not supported unary operation: `{}{}`",
                                    operator.display().fg(Color::BrightRed),
                                    operand.borrow().type_name().fg(Color::BrightYellow)
                                )
                            )
                            .with_label("Not supported unary operation", &expr.span, None)
                    )
                }
            }
            Expr::GetAttr { object, field } => {
                let obj_val = Self::evaluate(*object, Rc::clone(&env))?;
                let obj_borrow = obj_val.borrow();

                match obj_borrow.field(&field) {
                    Some(val) => Rc::clone(&val),
                    None => return Err(
                        RuneWayError::new(RuneWayErrorKind::Runtime(Some("AttributeError".to_string())))
                            .with_message(format!(
                                "Field `{}` not found in type: {}",
                                (&field).fg(Color::BrightYellow),
                                obj_borrow.type_name().fg(Color::BrightYellow)
                            ))
                            .with_label("Not found attribute", &expr.span, None)
                    )
                }
            }
            Expr::Slice { object, index } => {
                let obj_val = Self::evaluate(*object, Rc::clone(&env))?;
                let index_val = Self::evaluate(*index, Rc::clone(&env))?;
                if let Some(callable) = obj_val.borrow().field("slice") {
                    match callable.borrow().call(vec![Rc::clone(&obj_val), Rc::clone(&index_val)].as_slice()) {
                        Some(r) => r?,
                        None => return Err(
                            RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                .with_message(
                                    format!(
                                        "<{}> object is not callable",
                                        callable.borrow().type_name()
                                    ))
                                .with_label("Not callable", &expr.span, None)
                        )
                    }
                } else {
                    return Err(
                        RuneWayError::new(RuneWayErrorKind::Runtime(Some("AttributeError".to_string())))
                            .with_message(format!(
                                "Cannot use slice on type: {}. Required method `{}`",
                                obj_val.borrow().type_name().fg(Color::BrightYellow),
                                "slice".fg(Color::BrightYellow),
                            ))
                            .with_label("Not found attribute", &expr.span, None)
                    );
                }
            }
            Expr::FString(f_string_items) => {
                let mut string = String::new();
                for f_string_item in f_string_items.iter() {
                    match f_string_item {
                        FStringExpr::String(s) => string.push_str(s),
                        FStringExpr::Expr(expr) => {
                            let value = Self::evaluate(expr.clone(), Rc::clone(&env))?;
                            let borrowed = value.borrow();

                            if let Some(val) = borrowed.value().downcast_ref::<String>() {
                                string.push_str(val);
                                continue;
                            }

                            drop(borrowed);

                            let val = cast_to::<RNWString>(&value)?;

                            let borrowed = val.borrow();
                            let Some(str_val) = borrowed.value().downcast_ref::<String>() else {
                                return Err(
                                    RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                        .with_message(format!(
                                            "Expected type <{}> from method `{}`. Got: <{}>",
                                            RNWString::type_name().fg(Color::BrightYellow),
                                            "to_string".fg(Color::BrightYellow),
                                            borrowed.type_name().fg(Color::BrightYellow),
                                        ))
                                        .with_label("Wrong returns value", &expr.span, None)
                                );
                            };

                            string.push_str(str_val);
                        }
                    }
                }
                RNWString::new(string)
            }
            Expr::Dict(vec) => {
                let mut map = HashMap::new();
                for (key, val) in vec.iter() {
                    let key_value = Self::evaluate(*key.clone(), Rc::clone(&env))?;
                    let val_value = Self::evaluate(*val.clone(), Rc::clone(&env))?;
                    map.insert(
                        key_value.borrow().as_any().downcast_ref::<RNWString>().ok_or_else(
                            || {
                                RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                    .with_message(format!(
                                        "Key value mast be a string. Not <{}>",
                                        key_value.borrow().type_name().fg(Color::BrightYellow)
                                    ))
                            }
                        )?.value.clone(),
                        val_value
                    );
                }
                RNWDict::new(map)
            }
            _ => panic!("Not implemented yet. Expression: {:?}", expr),
        };
        Ok(result)
    }

    pub fn entry(root_env: EnvRef, entry_function_name: &'static str) -> RWResult<()> {
        let function = root_env.borrow().get_variable(entry_function_name);
        match function {
            Some(function) => {
                let result = match function.borrow().call(&[]) {
                    Some(r) => r?,
                    None => return Err(
                        RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                            .with_message(
                                format!(
                                    "Cannot use not callable <{}> object as entry",
                                    function.borrow().type_name()
                                ))
                    )
                };
                let code = if RNWNullType::is_type_equals(&result) {
                    0
                } else {
                    match result.borrow().as_any().downcast_ref::<RNWInteger>() {
                        Some(val) => val.value,
                        None => return Err(
                            RuneWayError::new(RuneWayErrorKind::Runtime(Some("TypeError".to_string())))
                                .with_message(format!(
                                    "Entry `{}` exit code must be <{}> or <{}>. Got: <{}>",
                                    entry_function_name,
                                    RNWInteger::type_name().fg(Color::BrightYellow),
                                    RNWNullType::type_name().fg(Color::BrightYellow),
                                    result.borrow().type_name().fg(Color::BrightYellow),
                                ))
                        )
                    }
                };
                println!("\n\nProcess finished with exit code {}", code);
                Ok(())
            }
            None => Err(
                RuneWayError::new(RuneWayErrorKind::Runtime(Some("NameError".to_string())))
                    .with_message(format!("Variable '{}' not found", entry_function_name))
            )
        }
    }
}