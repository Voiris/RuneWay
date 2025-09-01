use crate::runeway::core::errors::{RWResult, RuneWayError, RuneWayErrorKind};
use crate::runeway::core::utils::assert::assert_incorrect_type;
use crate::runeway::runtime::types::{
    RNWFunction, RNWObjectRef, RNWRegisteredNativeFunction, RNWTypeId,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct Environment {
    parent: Option<EnvRef>,
    variables: HashMap<String, EnvField>
}

impl Environment {
    fn new(parent: Option<EnvRef>) -> EnvRef {
        Rc::new(RefCell::new(Self {
            parent,
            variables: HashMap::new()
        }))
    }

    /// Создаём супер-глобальное окружение без родителя для builtins
    pub fn new_builtins_global() -> EnvRef {
        Self::new(None)
    }

    /// Создаём глобальное окружение с доступом к builtins
    pub fn new_global(builtins: EnvRef) -> EnvRef {
        Self::new(Some(builtins))
    }

    /// Создаём вложенное окружение с указанием родителя
    pub fn new_enclosed(parent: EnvRef) -> EnvRef {
        Self::new(Some(parent))
    }

    /// Определяем новую переменную в текущем окружении (локально)
    pub fn define_variable(&mut self, name: String, value: RNWObjectRef) {
        self.variables.insert(name, EnvField::new(value.clone()));
    }

    /// Определяем новую функцию в текущем окружении (локально)
    pub fn define_function(&mut self, function: Rc<RNWRegisteredNativeFunction>) {
        self.variables.insert(
            function.name.clone(),
            EnvField::new_with_type(
                RNWFunction::new(function.clone()),
                Some(RNWFunction::rnw_type_id()),
            ),
        );
    }

    pub fn define_uninitiated_variable(&mut self, name: String, static_type: Option<RNWTypeId>) {
        self.variables
            .insert(name, EnvField::new_uninitiated(static_type));
    }

    /// Получаем ссылку на переменную, если она есть в текущем или родительских окружениях
    pub fn get_variable<T: ToString>(&self, name: T) -> Option<RNWObjectRef> {
        if let Some(field) = self.variables.get(&name.to_string()) {
            field.value.clone()
        } else if let Some(parent_env) = &self.parent {
            parent_env.borrow().get_variable(name.to_string())
        } else {
            None
        }
    }

    fn assign_variable_value(
        &mut self,
        name: impl ToString,
        value: &RNWObjectRef,
    ) -> RWResult<bool> {
        let name = name.to_string();
        if let Some(field) = self.variables.get(&name).cloned() {
            if let Some(static_type_id) = field.static_type {
                let borrow = value.borrow();
                assert_incorrect_type(static_type_id, borrow.rnw_type_id())?;
            }
            self.variables.insert(
                name,
                EnvField::new_with_type(value.clone(), field.static_type),
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Обновляем значение переменной, ищем где она объявлена (текущий или родительские уровни)
    pub fn assign_variable(
        &mut self,
        name: impl ToString + std::fmt::Display,
        value: RNWObjectRef,
    ) -> RWResult<()> {
        if self.assign_variable_value(&name, &value)? {
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().assign_variable(name, value)
        } else {
            Err(RuneWayError::new(RuneWayErrorKind::name_error())
                .with_message(format!("Variable '{}' not defined", name)))
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

    pub fn find_similar_strings(&self, string: impl AsRef<str>) -> Vec<String> {
        let library_names: HashSet<String> = self.collect_all_names();

        let mut vec: Vec<(_, _)> = library_names
            .iter()
            .map(|name| (name, strsim::levenshtein(name, string.as_ref())))
            .filter(|(_, dist)| *dist <= 4) // dist <= threshold
            .collect::<Vec<_>>();
        vec.sort_by_key(|(_, dist)| *dist);

        vec.into_iter().map(|(name, _)| name.clone()).collect()
    }

    pub fn get_as_hash_map(&self) -> HashMap<String, RNWObjectRef> {
        let mut hash_map: HashMap<String, RNWObjectRef> = HashMap::new();

        for (name, field) in self.variables.iter() {
            if field.value.is_some() {
                hash_map.insert(name.clone(), field.value.clone().unwrap());
            }
        }

        hash_map
    }
}

#[derive(Clone, Debug)]
struct EnvField {
    value: Option<RNWObjectRef>,
    static_type: Option<RNWTypeId>,
}

impl EnvField {
    pub fn new(value: RNWObjectRef) -> Self {
        let borrow = value.borrow();

        Self {
            value: Some(value.clone()),
            static_type: Some(borrow.rnw_type_id()),
        }
    }

    pub fn new_with_type(value: RNWObjectRef, static_type: Option<RNWTypeId>) -> Self {
        Self {
            value: Some(value),
            static_type,
        }
    }

    pub fn new_uninitiated(static_type: Option<RNWTypeId>) -> Self {
        Self {
            value: None,
            static_type,
        }
    }
}

pub type EnvRef = Rc<RefCell<Environment>>;
