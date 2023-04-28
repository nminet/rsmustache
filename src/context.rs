use std::fmt;

pub enum Context<'a> {
    Value(String),
    Object(Box<dyn Fn(&str) -> Option<Context<'a>> + 'a>),
    Iterable(Vec<Context<'a>>),
    Factory0(Box<dyn Fn() -> Context<'a> + 'a>),
    Factory1(Box<dyn Fn(&str) -> Context<'a> + 'a>),
    Lambda0(Box<dyn Fn() -> String + 'a>),
    Lanbda1(Box<dyn Fn(&str) -> String + 'a>),
}

impl<'a> fmt::Debug for Context<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", "Context {...}")
    }
}


pub trait IntoContext {
    fn into_context(&self) -> Context;
}


pub(crate) struct Stack<'a> {
    frames: Vec<Context<'a>>
}


impl<'a> Stack<'a> {
    pub(crate) fn from(root: Context<'a>) -> Self {
        Stack {
            frames: vec! [
                root
            ]
        }
    }

    pub(crate) fn resolve(&self, name: &str) -> String {
        match self.frames.last() {
            Some(Context::Object(getter)) => {
                if let Some(Context::Value(value)) = getter(name) {
                    value
                } else {
                    String::new()
                }
            },
            Some(Context::Value(value)) => {
                if name == "." {
                    value.clone()
                } else {
                    String::new()
                }
            },
            _ => String::new()
        }
    }
}
