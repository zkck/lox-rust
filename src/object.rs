
#[derive(Debug, Clone, PartialEq)]
pub enum LoxObject {
    Number(f32),
    String(String),
    True,
    False,
    Nil,
}

impl From<f32> for LoxObject {
    fn from(f: f32) -> Self {
        LoxObject::Number(f)
    }
}

impl From<bool> for LoxObject {
    fn from(b: bool) -> Self {
        if b {
            LoxObject::True
        } else {
            LoxObject::False
        }
    }
}

impl From<String> for LoxObject {
    fn from(s: String) -> Self {
        LoxObject::String(s)
    }
}

impl std::fmt::Display for LoxObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoxObject::Number(n) => write!(f, "{}", n),
            LoxObject::String(s) => write!(f, "{}", s),
            LoxObject::True => write!(f, "true"),
            LoxObject::False => write!(f, "false"),
            LoxObject::Nil => write!(f, "nil"),
        }
    }
}

