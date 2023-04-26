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


impl Stack<'_> {
    pub(crate) fn new() -> Self {
        Stack {
            frames: vec! []
        }
    }
}
