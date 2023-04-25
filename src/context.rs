pub enum Context<'a> {
    Value(&'a str),
    Object(Box<dyn Fn(&str) -> Option<Context>>),
    Iterable(&'a Vec<Context<'a>>),
    Factory0(Box<dyn Fn() -> Context<'a>>),
    Factory1(Box<dyn Fn(&str) -> Context>),
    Lambda0(Box<dyn Fn() -> String>),
    Lanbda1(Box<dyn Fn(&str) -> String>),
}

pub trait IntoContext<'a> {
    fn into_context() -> Context<'a>;
}


pub(crate) struct Stack<'a> {
    frames: Vec<Context<'a>>
}


impl<'a> Stack<'a> {
    pub(crate) fn new() -> Self {
        Stack {
            frames: vec! []
        }
    }
}
