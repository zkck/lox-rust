use std::collections::HashMap;

use crate::object;

type Variables = HashMap<String, object::LoxObject>;

pub struct Environment {
    globals: Variables,
    locals: Vec<Variables>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            globals: HashMap::new(),
            locals: vec![],
        }
    }

    fn current_scope(&mut self) -> &mut Variables {
        self.locals.last_mut().unwrap_or(&mut self.globals)
    }

    pub fn define(&mut self, name: String, value: object::LoxObject) {
        self.current_scope().insert(name, value);
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut object::LoxObject> {
        self.locals
            .iter_mut()
            .rev()
            .find_map(|variables| variables.get_mut(name))
            .or(self.globals.get_mut(name))
    }

    pub fn get(&self, name: &str) -> Option<object::LoxObject> {
        self.locals
            .iter()
            .rev()
            .find_map(|variables| variables.get(name))
            .or(self.globals.get(name))
            .cloned()
    }

    pub fn assign(&mut self, name: &str, new_value: object::LoxObject) -> bool {
        match self.get_mut(name) {
            Some(value) => {
                *value = new_value;
                true
            }
            None => false,
        }
    }

    pub fn new_scope(&mut self) {
        self.locals.push(HashMap::new())
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }
}
