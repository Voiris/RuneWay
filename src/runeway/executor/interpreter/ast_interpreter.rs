use crate::runeway::builtins::types::{
    RNWBoolean, RNWDict, RNWFloat, RNWInteger, RNWIterator, RNWList, RNWNullType, RNWString,
    RNWTuple, RNWUnsignedInteger,
};
use crate::runeway::core::ast::expression::SpannedExpr;
use crate::runeway::core::ast::statement::{AnnotatedParameter, SpannedStatement};
use crate::runeway::core::ast::{
    expression::{Expr, FStringExpr},
    statement::{ImportItem, Statement},
};
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::core::spanned::Spanned;
use crate::runeway::core::utils::assert::assert_incorrect_type;
use crate::runeway::runtime::controlflow::ControlFlow;
use crate::runeway::runtime::environment::{EnvRef, Environment};
use crate::runeway::runtime::libraries::RNWModule;
use crate::runeway::runtime::types::types_reg::register_type;
use crate::runeway::runtime::types::{
    cast_to, RNWMethod, RNWObject, RNWObjectRef, RNWRegisteredNativeFunction,
    RNWRegisteredNativeMethod, RNWType, RNWTypeId, UserDefinedClass,
};
use colored::*;
use std::collections::HashMap;
use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use crate::runeway::core::ast::operators::BinaryOperator;
use crate::runeway::core::parser::parse_code;
use crate::runeway::core::utils::get_rc_id;
use crate::runeway::runtime::libraries;

pub struct ASTInterpreter;

impl ASTInterpreter {
    //noinspection RsExternalLinter
    pub fn execute(
        root_env: EnvRef,
        stmt: SpannedStatement,
        working_dir: &Path,
        filename: String,
        code: &String,
    ) -> RWResult<()> {
        Self::execute_local(
            root_env.clone(),
            stmt,
            None,
            true,
            working_dir,
            &filename,
            code,
        )?;
        Ok(())
    }

    pub fn execute_many(
        root_env: EnvRef,
        stmts: Vec<SpannedStatement>,
        working_dir: &Path,
        filename: String,
        code: &String,
    ) -> RWResult<()> {
        for stmt in stmts {
            Self::execute_local(
                root_env.clone(),
                stmt,
                None,
                true,
                working_dir,
                &filename,
                code,
            )?;
        }
        Ok(())
    }

    fn execute_many_local(
        root_env: EnvRef,
        stmts: Vec<SpannedStatement>,
        parent_env: Option<EnvRef>,
        is_top_level: bool,
        working_dir: &Path,
        filename: &String,
        code: &String,
    ) -> RWResult<ControlFlow> {
        for stmt in stmts {
            match Self::execute_local(
                root_env.clone(),
                stmt,
                parent_env.clone(),
                is_top_level,
                working_dir,
                filename,
                code,
            )? {
                ControlFlow::Return(r) => return Ok(ControlFlow::Return(r)),
                _ => (),
            }
        }
        Ok(ControlFlow::Nothing)
    }

    //noinspection DuplicatedCode
    fn execute_local(
        root_env: EnvRef,
        stmt: SpannedStatement,
        parent_env: Option<EnvRef>,
        is_top_level: bool,
        working_dir: &Path,
        filename: &String,
        code: &String,
    ) -> RWResult<ControlFlow> {
        let env = if let Some(_parent_env) = parent_env {
            _parent_env
        } else {
            Rc::clone(&root_env)
        };

        if !is_top_level {
            let result = match stmt.node {
                Statement::Expr(expr) => {
                    Self::evaluate(expr, Rc::clone(&env), filename)?;
                    ControlFlow::Nothing
                }
                Statement::Let {
                    name,
                    value,
                    annotation,
                } => {
                    let val = Self::evaluate(value, Rc::clone(&env), filename)?;

                    if let Some(annotation) = Self::handle_annotation(&annotation, &env, filename)?
                    {
                        let borrow = val.borrow();
                        assert_incorrect_type(annotation, borrow.rnw_type_id())?;
                    }

                    env.borrow_mut().define_variable(name.clone(), val);
                    ControlFlow::Nothing
                }
                Statement::LetVoid { name, annotation } => {
                    let static_type = Self::handle_annotation(&annotation, &env, filename)?;

                    env.borrow_mut()
                        .define_uninitiated_variable(name.clone(), static_type);

                    ControlFlow::Nothing
                }
                Statement::Assign { name, value } => {
                    let val = Self::evaluate(value, Rc::clone(&env), filename)?;
                    env.borrow_mut().assign_variable(&name, val)?;
                    ControlFlow::Nothing
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    let mut controlflow = ControlFlow::Nothing;

                    let branch = {
                        let condition_value =
                            Self::evaluate(condition.clone(), Rc::clone(&env), filename)?;

                        let casted_condition_value =
                            cast_to(&condition_value, RNWBoolean::rnw_type_id())?;

                        let cond = *casted_condition_value
                            .borrow()
                            .value()
                            .downcast_ref::<bool>()
                            .unwrap();

                        if cond { Some(then_branch) } else { else_branch }
                    };

                    if let Some(statements) = branch {
                        let enclosed_env = Environment::new_enclosed(env.clone());
                        for stmt in statements {
                            if stmt.node.is(&Statement::Break) {
                                controlflow = ControlFlow::Break;
                                break;
                            } else if stmt.node.is(&Statement::Continue) {
                                controlflow = ControlFlow::Continue;
                                break;
                            } else if let ControlFlow::Return(value) = Self::execute_local(
                                root_env.clone(),
                                *stmt,
                                Some(enclosed_env.clone()),
                                false,
                                working_dir,
                                filename,
                                code,
                            )? {
                                controlflow = ControlFlow::Return(value)
                            }
                        }
                    }
                    controlflow
                }
                Statement::While { condition, body } => {
                    'outer: loop {
                        let condition_value =
                            Self::evaluate(condition.clone(), Rc::clone(&env), filename)?;

                        let casted_condition_value =
                            cast_to(&condition_value, RNWBoolean::rnw_type_id())?;

                        let cond = *casted_condition_value
                            .borrow()
                            .value()
                            .downcast_ref::<bool>()
                            .unwrap();

                        if cond {
                            for stmt in &body {
                                let cf = Self::execute_local(
                                    root_env.clone(),
                                    *stmt.clone(),
                                    Some(Rc::clone(&env)),
                                    false,
                                    working_dir,
                                    filename,
                                    code,
                                )?;
                                match cf {
                                    ControlFlow::Break => break 'outer,
                                    ControlFlow::Return(value) => {
                                        return Ok(ControlFlow::Return(value));
                                    }
                                    ControlFlow::Continue => {
                                        continue 'outer;
                                    }
                                    _ => (),
                                }
                            }
                        } else {
                            break 'outer;
                        }
                    }
                    ControlFlow::Nothing
                }
                Statement::Return(expr) => {
                    ControlFlow::Return(Self::evaluate(expr, Rc::clone(&env), filename)?)
                }
                Statement::Break => ControlFlow::Break,
                Statement::Continue => ControlFlow::Continue,
                Statement::Act {
                    name,
                    parameters,
                    return_annotation,
                    body,
                } => {
                    let function = Self::handle_function_act(
                        env.clone(),
                        name,
                        parameters,
                        return_annotation,
                        body,
                        working_dir,
                        filename,
                        code,
                    )?;

                    let mut borrow = env.borrow_mut();
                    borrow.define_function(function);
                    drop(borrow);

                    ControlFlow::Nothing
                }
                Statement::For {
                    variable,
                    iterable,
                    body,
                } => {
                    let iterable_val = Self::evaluate(iterable.clone(), Rc::clone(&env), filename)?;
                    let mut iterator = match iterable_val
                        .borrow_mut()
                        .as_any_mut()
                        .downcast_ref::<RNWIterator>()
                    {
                        Some(iterator) => iterator,
                        None => {
                            return Err(RuneWayError::new(RuneWayErrorKind::type_error())
                                .with_message("Iterable must be an iterator")
                                .with_label(
                                    "This must return an iterator",
                                    &iterable.span,
                                    filename,
                                )
                                .with_help(format!(
                                    "Try to add {} after iterable value",
                                    ".iter()".bright_green()
                                )));
                        }
                    }
                    .clone();
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
                            match Self::execute_local(
                                iteration_env.clone(),
                                *stmt.clone(),
                                None,
                                false,
                                working_dir,
                                filename,
                                code,
                            )? {
                                ControlFlow::Return(value) => {
                                    return Ok(ControlFlow::Return(value));
                                }
                                ControlFlow::Continue => continue 'outer,
                                ControlFlow::Break => break 'outer,
                                _ => (),
                            }
                        }
                    }

                    ControlFlow::Nothing
                }
                Statement::Assert(expr) => {
                    let obj = Self::evaluate(expr.clone(), Rc::clone(&env), filename)?;
                    let casted_obj = cast_to(&obj, RNWBoolean::rnw_type_id())?;
                    let casted_obj_borrow = casted_obj.borrow();
                    match casted_obj_borrow.value().downcast_ref::<bool>() {
                        Some(true) => (),
                        Some(false) => {
                            return Err(RuneWayError::new(RuneWayErrorKind::error_with_code(
                                "AssertionError",
                            ))
                            .with_message("Statement is not asserted")
                            .with_label(
                                "This expression is not asserted",
                                &expr.span,
                                filename,
                            ));
                        }
                        None => unreachable!(),
                    }

                    ControlFlow::Nothing
                }
                _ => panic!("Not implemented yet. Statement: {:?}", stmt),
            };
            Ok(result)
        } else {
            match stmt.node {
                Statement::Act {
                    name,
                    parameters,
                    return_annotation,
                    body,
                } => {
                    let function = Self::handle_function_act(
                        env.clone(),
                        name,
                        parameters,
                        return_annotation,
                        body,
                        working_dir,
                        filename,
                        code,
                    )?;

                    let mut borrow = env.borrow_mut();
                    borrow.define_function(function);
                    drop(borrow);

                    Ok(ControlFlow::Nothing)
                }
                Statement::Class { name, body } => {
                    let class_obj = Self::handle_class(
                        name.clone(),
                        body,
                        &env.clone(),
                        working_dir,
                        filename,
                        code,
                    )?;

                    let (rnw_type_id, type_obj) = {
                        let borrow = class_obj.borrow();
                        (
                            borrow.rnw_type_id(),
                            RNWType::new(borrow.rnw_type_id(), borrow.type_name()),
                        )
                    };

                    register_type(rnw_type_id, type_obj);

                    let mut borrow = env.borrow_mut();
                    borrow.define_variable(name, class_obj);
                    drop(borrow);

                    Ok(ControlFlow::Nothing)
                }
                Statement::Import { path, item } => {
                    Self::execute_import(env.clone(), path, item, &stmt.span, working_dir, filename)
                }
                _ => Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                    .with_message("Found forbidden statement on top-level")
                    .with_label(
                        "This statement is forbidden on top-level",
                        &stmt.span,
                        filename,
                    )),
            }
        }
    }

    fn handle_class(
        name: String,
        body: Vec<Box<SpannedStatement>>,
        env: &EnvRef,
        working_dir: &Path,
        filename: &String,
        code: &String,
    ) -> RWResult<RNWObjectRef> {
        let class_obj = UserDefinedClass::new(&name);

        let unboxed_body = body
            .iter()
            .map(|s| *s.clone())
            .collect::<Vec<SpannedStatement>>();

        for stmt in unboxed_body.iter().cloned() {
            match stmt.node {
                Statement::Act {
                    name: func_name,
                    parameters,
                    return_annotation,
                    body,
                } => {
                    let method_obj = {
                        let method = Self::handle_method_act(
                            env.clone(),
                            format!("{}.{}", name, func_name),
                            parameters,
                            return_annotation,
                            body,
                            working_dir,
                            filename,
                            code,
                        )?;
                        RNWMethod::new(method)
                    };

                    let mut class_borrow = class_obj.borrow_mut();
                    class_borrow.set_attr(func_name.as_str(), method_obj)?
                }
                _ => {
                    return Err(RuneWayError::new(RuneWayErrorKind::syntax_error())
                        .with_message("Found forbidden statement in class")
                        .with_label("This statement is forbidden in class", &stmt.span, filename));
                }
            }
        }

        Ok(class_obj)
    }

    fn execute_import(
        env: EnvRef,
        path: String,
        item: ImportItem,
        span: &Range<usize>,
        working_dir: &Path,
        filename: &String,
    ) -> RWResult<ControlFlow> {
        let library_env = ASTInterpreter::load_library(path.clone(), working_dir)?;

        let mut borrow = env.borrow_mut();
        match item {
            ImportItem::Alias(alias) => {
                borrow.define_variable(alias, RNWModule::new(path.clone(), library_env.clone()))
            }
            ImportItem::All => borrow.merge(library_env.clone())?,
            ImportItem::Selective(selective) => {
                let library_borrow = library_env.borrow();
                for symbol in selective.iter().cloned() {
                    let alias = symbol.alias.unwrap_or_else(|| symbol.original.clone());
                    if let Some(variable) = library_borrow.get_variable(&symbol.original) {
                        borrow.define_variable(alias, variable.clone())
                    } else {
                        let symbol_name = symbol.original.clone();
                        let mut err =
                            RuneWayError::new(RuneWayErrorKind::error_with_code("ImportError"))
                                .with_message(format!(
                                    "Symbol `{}` not found in module `{}`",
                                    (&symbol_name).bright_yellow(),
                                    path.bright_yellow()
                                ))
                                .with_label("In this declaration", span, filename);

                        if let Some(matches_name) =
                            library_borrow.find_similar_strings(symbol_name).first()
                        {
                            err = err.with_help(format!(
                                "Maybe you mean `{}`?",
                                matches_name.bright_green()
                            ));
                        }

                        return Err(err);
                    }
                }
            }
        }

        Ok(ControlFlow::Nothing)
    }

    fn execute_act(
        params: &Vec<String>,
        args: &[RNWObjectRef],
        unboxed_body: &Vec<SpannedStatement>,
        env: &EnvRef,
        working_dir: &Path,
        filename: &String,
        code: &String,
    ) -> RWResult<RNWObjectRef> {
        let __env = Environment::new_enclosed(env.clone());
        let mut borrow = __env.borrow_mut();

        // Связываем параметры с аргументами
        for (_param, _arg) in params.iter().zip(args.iter()) {
            borrow.define_variable(_param.clone(), Rc::clone(&_arg));
        }

        drop(borrow);

        // Клонируем тело при каждом вызове, чтобы не перемещать из замыкания
        let body_clone = unboxed_body.clone();

        // Выполняем тело
        match ASTInterpreter::execute_many_local(
            Rc::clone(&env),
            body_clone,
            Some(__env.clone()),
            false,
            &working_dir,
            &filename,
            &code,
        )
        .map_err(|e| e.with_source(filename.clone(), code.clone()))?
        {
            ControlFlow::Return(value) => Ok(value),
            ControlFlow::Nothing => Ok(RNWNullType::new()),
            _ => unreachable!(),
        }
    }

    fn handle_function_act(
        env: EnvRef,
        name: String,
        parameters: Vec<AnnotatedParameter>,
        return_annotation: Option<Spanned<String>>,
        body: Vec<Box<SpannedStatement>>,
        working_dir: &Path,
        filename: &String,
        code: &String,
    ) -> RWResult<Rc<RNWRegisteredNativeFunction>> {
        // Готовим данные для замыкания
        let unboxed_body: Vec<SpannedStatement> =
            body.into_iter().map(|stmt| (*stmt).clone()).collect();

        let _parameters: Vec<_> = parameters.iter().map(|ap| ap.name.clone()).collect();

        let _env = Rc::clone(&env);

        let _working_dir = working_dir.to_path_buf();

        let _filename = filename.to_owned();
        let _code = code.to_owned();

        // Создаём замыкание, которое реализует Fn (без перемещения unboxed_body)
        let func = Rc::new(move |_args: &[RNWObjectRef]| {
            Self::execute_act(
                &_parameters,
                _args,
                &unboxed_body,
                &_env,
                &_working_dir,
                &_filename,
                &_code,
            )
        });

        // Составляем вектор типов параметров (здесь универсальный тип dyn RNWObject)
        let mut params_type_ids: Vec<RNWTypeId> = Vec::new();
        for ap in parameters.iter() {
            params_type_ids
                .push(Self::handle_annotation(&ap.annotation, &env, filename)?.unwrap_or(0));
        }

        let return_type = Self::handle_annotation(&return_annotation, &env, filename)?;

        Ok(RNWRegisteredNativeFunction::new_with_return_type(
            name,
            func,
            params_type_ids,
            return_type,
        ))
    }

    fn handle_method_act(
        env: EnvRef,
        name: String,
        parameters: Vec<AnnotatedParameter>,
        return_annotation: Option<Spanned<String>>,
        body: Vec<Box<SpannedStatement>>,
        working_dir: &Path,
        filename: &String,
        code: &String,
    ) -> RWResult<Rc<RNWRegisteredNativeMethod>> {
        // Готовим данные для замыкания
        let unboxed_body: Vec<SpannedStatement> =
            body.into_iter().map(|stmt| (*stmt).clone()).collect();

        let _parameters: Vec<_> = parameters.iter().map(|ap| ap.name.clone()).collect();

        let _env = Rc::clone(&env);

        let _working_dir = working_dir.to_path_buf();

        let _filename = filename.to_owned();
        let _code = code.to_owned();

        // Создаём замыкание, которое реализует Fn (без перемещения unboxed_body)
        let func = Rc::new(move |_this: RNWObjectRef, _args: &[RNWObjectRef]| {
            let mut __args = vec![_this];
            __args.extend_from_slice(_args);
            Self::execute_act(
                &_parameters,
                __args.as_slice(),
                &unboxed_body,
                &_env,
                &_working_dir,
                &_filename,
                &_code,
            )
        });

        // Составляем вектор типов параметров (здесь универсальный тип dyn RNWObject)
        let mut params_type_ids: Vec<RNWTypeId> = Vec::new();
        for ap in parameters.iter() {
            params_type_ids
                .push(Self::handle_annotation(&ap.annotation, &env, filename)?.unwrap_or(0));
        }

        let return_type = Self::handle_annotation(&return_annotation, &env, filename)?;

        Ok(RNWRegisteredNativeMethod::new_with_return_type(
            name,
            func,
            params_type_ids,
            return_type,
        ))
    }

    fn handle_annotation(
        annotation: &Option<Spanned<String>>,
        env: &EnvRef,
        filename: &String,
    ) -> RWResult<Option<RNWTypeId>> {
        if let Some(annotation) = annotation.clone() {
            let r#type = env.borrow().get_variable(&annotation.node);
            let r#type = if let Some(r#type) = r#type {
                r#type
            } else {
                let mut err =
                    RuneWayError::new(RuneWayErrorKind::error_with_code("AnnotationError"))
                        .with_message(format!(
                            "Cannot find type: `{}`",
                            (&annotation.node).bright_yellow()
                        ))
                        .with_label("This is not found", &annotation.span, filename);

                println!(
                    "{:#?}",
                    env.borrow().find_similar_strings(annotation.node.clone(),)
                );

                if let Some(matches_name) =
                    env.borrow().find_similar_strings(annotation.node).first()
                {
                    err =
                        err.with_help(format!("Maybe you mean `{}`?", matches_name.bright_green()));
                }

                return Err(err);
            };
            if let Some(r#type) = r#type.borrow().as_any().downcast_ref::<RNWType>() {
                Ok(Some(r#type.rnw_type_id))
            } else {
                Err(
                    RuneWayError::new(RuneWayErrorKind::error_with_code("AnnotationError"))
                        .with_message(format!(
                            "Annotation must be a type identifier. Got <{}> type",
                            (&annotation.node).bright_yellow()
                        ))
                        .with_label("This is not a type", &annotation.span, filename),
                )
            }
        } else {
            Ok(None)
        }
    }

    //noinspection DuplicatedCode
    fn evaluate(expr: SpannedExpr, env: EnvRef, filename: &String) -> RWResult<RNWObjectRef> {
        let result = match expr.node {
            Expr::Null => RNWNullType::new(),
            Expr::String(s) => RNWString::new(s),
            Expr::Integer(i) => RNWInteger::new(i),
            Expr::UInteger(u) => RNWUnsignedInteger::new(u),
            Expr::Float(f) => RNWFloat::new(f),
            Expr::Boolean(b) => RNWBoolean::new(b),
            Expr::List(vec) => {
                let mut list: Vec<RNWObjectRef> = Vec::new();
                for item in vec.iter() {
                    list.push(Self::evaluate(*item.clone(), Rc::clone(&env), filename)?);
                }
                RNWList::new(&list)
            }
            Expr::Tuple(vec) => {
                let mut list: Vec<RNWObjectRef> = Vec::new();
                for item in vec.iter() {
                    list.push(Self::evaluate(*item.clone(), Rc::clone(&env), filename)?);
                }
                RNWTuple::new(&list)
            }
            Expr::Iterator { start, end, step } => {
                let start_obj = Self::evaluate(*start, Rc::clone(&env), filename)?;
                let end_obj = Self::evaluate(*end, Rc::clone(&env), filename)?;
                let step_obj = match step {
                    Some(int) => Self::evaluate(*int, Rc::clone(&env), filename)?,
                    None => RNWInteger::new(1),
                };

                let start_is_float = {
                    let start_borrow = start_obj.borrow();
                    start_borrow.as_any().is::<RNWFloat>()
                };
                let end_is_float = {
                    let end_borrow = end_obj.borrow();
                    end_borrow.as_any().is::<RNWFloat>()
                };
                let step_is_float = {
                    let step_borrow = step_obj.borrow();
                    step_borrow.as_any().is::<RNWFloat>()
                };

                let cast_type_id = if start_is_float || end_is_float || step_is_float {
                    RNWFloat::rnw_type_id()
                } else {
                    RNWInteger::rnw_type_id()
                };

                let casted_start_obj = cast_to(&start_obj, cast_type_id)?;
                let casted_end_obj = cast_to(&end_obj, cast_type_id)?;
                let casted_step_obj = cast_to(&step_obj, cast_type_id)?;

                let start_borrow = casted_start_obj.borrow();
                let end_borrow = casted_end_obj.borrow();
                let step_borrow = casted_step_obj.borrow();

                if start_is_float || end_is_float || step_is_float {
                    let start = start_borrow.value().downcast_ref::<f64>().unwrap();
                    let end = end_borrow.value().downcast_ref::<f64>().unwrap();
                    let step = step_borrow.value().downcast_ref::<f64>().unwrap();

                    RNWIterator::from_f64_range(*start, *end, *step)
                } else {
                    let start = start_borrow.value().downcast_ref::<i64>().unwrap();
                    let end = end_borrow.value().downcast_ref::<i64>().unwrap();
                    let step = step_borrow.value().downcast_ref::<i64>().unwrap();

                    RNWIterator::from_i64_range(*start, *end, *step)
                }
            }
            Expr::Call {
                ref callee,
                ref arguments,
            } => {
                let result = match &(*callee).node {
                    Expr::Variable(name) => {
                        let function = env.borrow().get_variable(&name);
                        match function {
                            Some(function) => {
                                let mut args = Vec::with_capacity(arguments.len());
                                for arg in arguments.iter().cloned() {
                                    args.push(Self::evaluate(arg, Rc::clone(&env), filename)?);
                                }
                                match function.borrow().call(&args) {
                                    Some(r) => r,
                                    None => {
                                        return Err(RuneWayError::new(
                                            RuneWayErrorKind::type_error(),
                                        )
                                        .with_message(format!(
                                            "<{}> object is not callable",
                                            function.borrow().type_name()
                                        ))
                                        .with_label("Not callable", &(*callee).span, filename));
                                    }
                                }
                            }
                            None => {
                                return Err(RuneWayError::new(RuneWayErrorKind::name_error())
                                    .with_message(format!("Variable '{}' not defined", name))
                                    .with_label("Not defined", &(*callee).span, filename));
                            }
                        }
                    }
                    Expr::AttributeAccess { object, field } => {
                        let obj_val = Self::evaluate(*object.clone(), Rc::clone(&env), filename)?;
                        let callable = {
                            let obj_borrow = obj_val.borrow();

                            if let Some(callable) = obj_borrow.get_attr(&field) {
                                callable.clone()
                            } else {
                                return Err(RuneWayError::new(RuneWayErrorKind::error_with_code(
                                    "AttributeError",
                                ))
                                .with_message(format!(
                                    "Method `{}` not found in type: {}",
                                    (&field).bright_yellow(),
                                    obj_borrow.type_name().bright_yellow()
                                ))
                                .with_label(
                                    "Not found attribute",
                                    &(*callee).span,
                                    filename,
                                ));
                            }
                        };
                        let mut args_eval = Vec::with_capacity(arguments.len());
                        for arg in arguments.iter().cloned() {
                            let evaluated = Self::evaluate(arg, Rc::clone(&env), filename)?;
                            args_eval.push(evaluated);
                        }

                        let mut args = if RNWMethod::is_type_equals(&callable) {
                            let class: Option<UserDefinedClass> = {
                                let borrow = obj_val.borrow();
                                borrow.as_any().downcast_ref::<UserDefinedClass>().cloned()
                            };
                            if let Some(class) = class {
                                if class.is_instance {
                                    vec![Rc::clone(&obj_val)]
                                } else {
                                    vec![class.new_instance()]
                                }
                            } else {
                                vec![Rc::clone(&obj_val)]
                            }
                        } else {
                            Vec::new()
                        };
                        args.extend(args_eval);

                        match callable.borrow().call(args.as_slice()) {
                            Some(r) => r,
                            None => {
                                return Err(RuneWayError::new(RuneWayErrorKind::type_error())
                                    .with_message(format!(
                                        "<{}> object is not callable",
                                        callable.borrow().type_name()
                                    )));
                            }
                        }
                    }
                    _ => panic!("Unexpected callee. Expr: {:?}", expr.clone()),
                };
                result.map_err(|e| e.with_secondary_label("In this call", &expr.span, filename))?
            }
            Expr::Variable(name) => match env.borrow().get_variable(&name) {
                Some(var) => Rc::clone(&var),
                None => {
                    return Err(RuneWayError::new(RuneWayErrorKind::name_error())
                        .with_message(format!("Variable '{}' not defined", name))
                        .with_label("Not defined variable", &expr.span, filename));
                }
            },
            Expr::BinaryOperation {
                left_operand,
                right_operand,
                operator,
            } => {
                let left = Self::evaluate(*left_operand.clone(), Rc::clone(&env), filename)?;
                let right = Self::evaluate(*right_operand.clone(), Rc::clone(&env), filename)?;

                match &operator {
                    BinaryOperator::Is => {
                        RNWBoolean::new(get_rc_id(left) == get_rc_id(right))
                    }
                    _ => {
                        let result = left
                            .borrow()
                            .binary_operation(right.clone(), operator.clone());
                        match result {
                            Some(val) => val,
                            None => {
                                return Err(RuneWayError::new(RuneWayErrorKind::error_with_code(
                                    "OperationError",
                                ))
                                    .with_message(format!(
                                        "Not supported binary operation: `{} {} {}`",
                                        left.borrow().type_name().bright_yellow(),
                                        operator.display().bright_red(),
                                        right.borrow().type_name().bright_yellow()
                                    ))
                                    .with_label(
                                        "Not supported binary operation",
                                        &expr.span,
                                        filename,
                                    ));
                            }
                        }
                    }
                }
            }
            Expr::UnaryOperation { operand, operator } => {
                let operand = Self::evaluate(*operand, Rc::clone(&env), filename)?;

                let result = operand.borrow().unary_operation(operator.clone());

                match result {
                    Some(val) => val,
                    None => {
                        return Err(RuneWayError::new(RuneWayErrorKind::error_with_code(
                            "OperationError",
                        ))
                        .with_message(format!(
                            "Not supported unary operation: `{}{}`",
                            operator.display().bright_red(),
                            operand.borrow().type_name().bright_yellow()
                        ))
                        .with_label(
                            "Not supported unary operation",
                            &expr.span,
                            filename,
                        ));
                    }
                }
            }
            Expr::AttributeAccess { object, field } => {
                let obj_val = Self::evaluate(*object, Rc::clone(&env), filename)?;
                let obj_borrow = obj_val.borrow();

                match obj_borrow.get_attr(&field) {
                    Some(val) => Rc::clone(&val),
                    None => {
                        return Err(RuneWayError::new(RuneWayErrorKind::error_with_code(
                            "AttributeError",
                        ))
                        .with_message(format!(
                            "Field `{}` not found in type: {}",
                            (&field).bright_yellow(),
                            obj_borrow.type_name().bright_yellow()
                        ))
                        .with_label(
                            "Not found attribute",
                            &expr.span,
                            filename,
                        ));
                    }
                }
            }
            Expr::SetAttr { object, value } => match object.node {
                Expr::AttributeAccess { object: obj, field } => {
                    let obj_val = Self::evaluate(*obj, Rc::clone(&env), filename)?;
                    let value_val = Self::evaluate(*value, Rc::clone(&env), filename)?;

                    let mut obj_borrow = obj_val.borrow_mut();

                    obj_borrow.set_attr(&field, value_val.clone())?;

                    value_val
                }
                _ => unreachable!(),
            },
            Expr::Slice { object, index } => {
                let obj_val = Self::evaluate(*object, Rc::clone(&env), filename)?;
                let index_val = Self::evaluate(*index, Rc::clone(&env), filename)?;
                if let Some(callable) = obj_val.borrow().get_attr("slice") {
                    match callable
                        .borrow()
                        .call(vec![Rc::clone(&obj_val), Rc::clone(&index_val)].as_slice())
                    {
                        Some(r) => r?,
                        None => {
                            return Err(RuneWayError::new(RuneWayErrorKind::type_error())
                                .with_message(format!(
                                    "<{}> object is not callable",
                                    callable.borrow().type_name()
                                ))
                                .with_label("Not callable", &expr.span, filename));
                        }
                    }
                } else {
                    return Err(RuneWayError::new(RuneWayErrorKind::error_with_code(
                        "AttributeError",
                    ))
                    .with_message(format!(
                        "Cannot use slice on type: {}. Required method `{}`",
                        obj_val.borrow().type_name().bright_yellow(),
                        "slice".bright_yellow(),
                    ))
                    .with_label("Not found attribute", &expr.span, filename));
                }
            }
            Expr::FString(f_string_items) => {
                let mut string = String::new();
                for f_string_item in f_string_items.iter() {
                    match f_string_item {
                        FStringExpr::String(s) => string.push_str(s),
                        FStringExpr::Expr(expr) => {
                            let value = Self::evaluate(expr.clone(), Rc::clone(&env), filename)?;
                            let borrowed = value.borrow();

                            if let Some(val) = borrowed.value().downcast_ref::<String>() {
                                string.push_str(val);
                                continue;
                            }

                            drop(borrowed);

                            let val = cast_to(&value, RNWString::rnw_type_id())?;

                            let borrowed = val.borrow();
                            let Some(str_val) = borrowed.value().downcast_ref::<String>() else {
                                return Err(RuneWayError::new(RuneWayErrorKind::type_error())
                                    .with_message(format!(
                                        "Expected type <{}> from method `{}`. Got: <{}>",
                                        RNWString::type_name().bright_yellow(),
                                        "to_string".bright_yellow(),
                                        borrowed.type_name().bright_yellow(),
                                    ))
                                    .with_label("Wrong returns value", &expr.span, filename));
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
                    let key_value = Self::evaluate(*key.clone(), Rc::clone(&env), filename)?;
                    let val_value = Self::evaluate(*val.clone(), Rc::clone(&env), filename)?;
                    map.insert(
                        key_value
                            .borrow()
                            .as_any()
                            .downcast_ref::<RNWString>()
                            .ok_or_else(|| {
                                RuneWayError::new(RuneWayErrorKind::type_error()).with_message(
                                    format!(
                                        "Key value mast be a string. Not <{}>",
                                        key_value.borrow().type_name().bright_yellow()
                                    ),
                                )
                            })?
                            .value
                            .clone(),
                        val_value,
                    );
                }
                RNWDict::new(map)
            }
            _ => panic!("Not implemented yet. Expression: {:?}", expr),
        };
        Ok(result)
    }

    fn load_library(import_path: String, working_dir: &Path) -> RWResult<EnvRef> {
        if let Some(name) = import_path.strip_prefix("std::") {
            if !libraries::loaded::is_loaded(&import_path) {
                let module = match libraries::stdlib_registry::get_stdlib(name) {
                    Some(loader) => loader(),
                    None => {
                        return Err(
                            RuneWayError::new(RuneWayErrorKind::error_with_code("ImportError"))
                                .with_message(format!("Cannot load library `{}`", import_path)),
                        );
                    }
                };
                libraries::loaded::register_loaded(&import_path, module.clone());
                return Ok(module);
            }
            Ok(libraries::loaded::get_loaded(&import_path)
                .unwrap_or_else(|| panic!("InternalError: Cannot load the library '{}'", import_path)))
        } else {
            let import_path = if !import_path.ends_with(".rnw") {
                import_path + ".rnw"
            } else {
                import_path
            };
            let mut path = PathBuf::from(&import_path);
            if !path.is_absolute() {
                path = working_dir.join(&path);
            }
            if !path.is_file() {
                return Err(
                    RuneWayError::new(RuneWayErrorKind::error_with_code("FileSystemError"))
                        .with_message(format!(
                            "Path is not a file or it does not exists: {}",
                            path.display()
                        )),
                );
            }
            path = path.canonicalize().map_err(|e| {
                RuneWayError::new(RuneWayErrorKind::error_with_code("FileSystemError"))
                    .with_message(format!("Cannot canonicalize path: {}", path.display()))
                    .with_note(format!("Raw Error: {}", e))
            })?;
            if !libraries::loaded::is_loaded(&import_path) {
                let (filename, code) = if let Some(file_name) = path.file_name() {
                    (file_name.to_str().unwrap(), fs::read_to_string(&path)?)
                } else {
                    panic!("Internal: No filename found.");
                };
                let parsed_code = parse_code(filename.to_owned(), code.clone())
                    .map_err(|e| e.with_source(filename, &code))?;
                let env = Environment::new_global();
                ASTInterpreter::execute_many(
                    env.clone(),
                    parsed_code.ast,
                    working_dir,
                    filename.to_string(),
                    &code,
                )
                    .map_err(|e| e.with_source(filename, &code))?;
                libraries::loaded::register_loaded(&import_path, env.clone());
                return Ok(env);
            }
            Ok(libraries::loaded::get_loaded(&import_path)
                .unwrap_or_else(|| panic!("InternalError: Cannot load the library '{}'", import_path)))
        }
    }

    pub fn entry(root_env: EnvRef, entry_function_name: &'static str) -> RWResult<()> {
        let function = root_env.borrow().get_variable(entry_function_name);
        match function {
            Some(function) => {
                let result =
                    match function.borrow().call(&[]) {
                        Some(r) => r?,
                        None => {
                            return Err(RuneWayError::new(RuneWayErrorKind::type_error())
                                .with_message(format!(
                                    "Cannot use not callable <{}> object as entry",
                                    function.borrow().type_name()
                                )));
                        }
                    };
                let code = if RNWNullType::is_type_equals(&result) {
                    0
                } else {
                    match result.borrow().as_any().downcast_ref::<RNWInteger>() {
                        Some(val) => val.value,
                        None => {
                            return Err(RuneWayError::new(RuneWayErrorKind::type_error())
                                .with_message(format!(
                                    "Entry `{}` exit code must be <{}> or <{}>. Got: <{}>",
                                    entry_function_name,
                                    RNWInteger::type_name().bright_yellow(),
                                    RNWNullType::type_name().bright_yellow(),
                                    result.borrow().type_name().bright_yellow(),
                                )));
                        }
                    }
                };
                println!("\n\nProcess finished with exit code {}", code);
                Ok(())
            }
            None => Err(RuneWayError::new(RuneWayErrorKind::name_error())
                .with_message(format!("Variable '{}' not found", entry_function_name))),
        }
    }
}
