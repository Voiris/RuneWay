use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::runeway::executor::runtime::types::{RNWObjectRef, RNWRegisteredNativeFunction};

pub struct Environment {
    parent: Option<EnvRef>,
    variables: HashMap<String, RNWObjectRef>,
    functions: HashMap<String, Rc<RNWRegisteredNativeFunction>>,
}

impl Environment {
    fn new(parent: Option<Rc<RefCell<Self>>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            parent,
            variables: HashMap::new(),
            functions: HashMap::new(),
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
    pub fn define_variable(&mut self, name: String, value: RNWObjectRef) {
        self.variables.insert(name, value);
    }

    /// Определяем новую функцию в текущем окружении (локально)
    pub fn define_function(&mut self, function: Rc<RNWRegisteredNativeFunction>) {
        self.functions.insert(function.name.clone(), function);
    }

    /// Получаем ссылку на переменную, если она есть в текущем или родительских окружениях
    pub fn get_variable(&self, name: &str) -> Result<RNWObjectRef, String> {
        if let Some(val) = self.variables.get(name) {
            Ok(Rc::clone(val))
        } else if let Some(parent_env) = &self.parent {
            parent_env.borrow().get_variable(name)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    /// Получаем ссылку на функцию, если она есть в текущем или родительских окружениях
    pub fn get_function(&self, name: impl AsRef<str>) -> Result<Rc<RNWRegisteredNativeFunction>, String> {
        if let Some(func) = self.functions.get(name.as_ref()) {
            Ok(Rc::clone(func))
        } else if let Some(parent_env) = &self.parent {
            parent_env.borrow().get_function(name.as_ref())
        } else {
            Err(format!("Function '{}' not defined", name.as_ref()))
        }
    }

    /// Обновляем значение переменной, ищем где она объявлена (текущий или родительские уровни)
    pub fn assign_variable(&mut self, name: &str, value: RNWObjectRef) -> Result<(), String> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign_variable(name, value)
        } else {
            Err(format!("Variable '{}' not defined", name))
        }
    }

    /// Заливаем один Environment в другой
    pub fn merge(&mut self, other: Rc<RefCell<Self>>) {
        let borrow = other.borrow();

        for (key, value) in borrow.variables.iter() {
            self.variables.insert(key.clone(), value.clone());
        }

        for (function, value) in borrow.functions.iter() {
            self.functions.insert(function.clone(), value.clone());
        }
    }
}

pub type EnvRef = Rc<RefCell<Environment>>;
