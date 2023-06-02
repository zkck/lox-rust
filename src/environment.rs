use std::collections::HashMap;

use crate::object;

pub struct Environment {
    values: HashMap<String, object::LoxObject>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: object::LoxObject) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<object::LoxObject> {
        self.values.get(name).cloned()
    }
}
