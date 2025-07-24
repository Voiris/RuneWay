use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::runtime::types::{RNWFunction, RNWMethod, RNWObjectRef, RNWRegisteredNativeFunction, RNWRegisteredNativeMethod};

#[derive(Clone, Debug)]
pub struct Environment {
    parent: Option<EnvRef>,
    variables: HashMap<String, RNWObjectRef>,
}

impl Environment {
    fn new(parent: Option<Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent,
            variables: HashMap::new(),
        }))
    }

    /// Создаём глобальное окружение без родителя
    pub fn new_global() -> Rc<RefCell<Self>> {
        Self::new(None)
    }

    /// Создаём вложенное окружение с указанием родителя
    pub fn new_enclosed(parent: Rc<RefCell<Self>>) -> Rc<RefCell<Self>> {
        Self::new(Some(parent))
    }

    /// Определяем новую переменную в текущем окружении (локально)
    pub fn define_variable(&mut self, name: String, value: RNWObjectRef) -> () {
        self.variables.insert(name, value);
    }

    /// Определяем новую функцию в текущем окружении (локально)
    pub fn define_function(&mut self, function: Rc<RNWRegisteredNativeFunction>) -> () {
        self.variables.insert(function.name.clone(), RNWFunction::new(function.clone()));
    }

    /// Определяем новую функцию в текущем окружении (локально)
    pub fn define_method(&mut self, method: Rc<RNWRegisteredNativeMethod>) -> () {
        self.variables.insert(method.name.clone(), RNWMethod::new(method.clone()));
    }

    /// Получаем ссылку на переменную, если она есть в текущем или родительских окружениях
    pub fn get_variable(&self, name: &str) -> Option<RNWObjectRef> {
        if let Some(val) = self.variables.get(name) {
            Some(Rc::clone(val))
        } else if let Some(parent_env) = &self.parent {
            parent_env.borrow().get_variable(name)
        } else {
            None
        }
    }

    /// Обновляем значение переменной, ищем где она объявлена (текущий или родительские уровни)
    pub fn assign_variable(&mut self, name: &str, value: RNWObjectRef) -> RWResult<()> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign_variable(name, value)
        } else {
            Err(
                RuneWayError::new(RuneWayErrorKind::Runtime(Some("NameError".to_string())))
                    .with_message(format!("Variable '{}' not defined", name))
            )
        }
    }

    /// Заливаем один Environment в другой
    pub fn merge(&mut self, other: Rc<RefCell<Self>>) -> RWResult<()> {
        let borrow = other.borrow();

        for (key, value) in borrow.variables.iter() {
            self.variables.insert(key.clone(), value.clone());
        }

        Ok(())
    }

    /// Рекурсивно собираем все переменные и функции из текущего и родительских окружений
    pub fn collect_all_names(&self) -> HashSet<String> {
        let mut vars: HashSet<String> = HashSet::new();

        for var_name in self.variables.keys() {
            vars.insert(var_name.to_string());
        }

        if let Some(parent) = &self.parent {
            let parent_vars = parent.borrow().collect_all_names();
            vars.extend(parent_vars);
        }

        vars
    }

    pub fn find_similar_strings(&self, string: impl AsRef<str>, threshold: usize) -> Vec<String> {
        let library_names: HashSet<String> = self.collect_all_names();

        library_names.iter().cloned()
            .filter(
                |name| strsim::levenshtein(name, string.as_ref()) <= threshold
            )
            .collect()
    }
}

pub type EnvRef = Rc<RefCell<Environment>>;
