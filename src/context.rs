use std::fmt::Debug;
use std::rc::Rc;


pub trait Context<'a>: Debug {
    fn child(&self, name: &str) -> Option<RcContext<'a>>;
    fn children(&self) -> Vec<RcContext<'a>>;
    fn value(&self) -> Option<String>;
    fn is_truthy(&self) -> bool;
}

// Use an RC to ref as dotted names require the same data available
// in multiple stack frames. Since the actual Context implementation
// may be defined in an external crate, cloning may not be desirable.
pub type RcContext<'a> = Rc<dyn Context<'a> + 'a>;

pub fn into_rc<'a, T>(context: &'a T) -> RcContext<'a>
where &'a T: Context<'a> {
    Rc::new(context)
}


#[derive(Clone, Debug)]
struct Frame<'a> {
    context: RcContext<'a>,
    parent_ok: bool
}

#[derive(Clone, Debug)]
// Use a vector to keep implementation simple. The alternative would be
// a variation on linked list. The tradeof is copies of stack states.
// As mustache stacks are not very deep this seems acceptabe for now.
pub(crate) struct Stack<'a> {
    frames: Vec<Frame<'a>>
}

#[derive(Debug)]
pub(crate) enum PushResult<'a> {
    None,
    Single(Stack<'a>),
    List(Vec<Stack<'a>>)
}


impl<'a> Stack<'a> {

    pub(crate) fn root<T>(context: &'a T) -> Self
    where &'a T: Context<'a> {
        let root = Frame {
            context: into_rc(context),
            parent_ok: false
        };
        let mut frames: Vec<Frame<'a>> = Vec::new();
        frames.push(root);
        Stack { frames }
    }

    fn extend(&self, context: RcContext<'a>, parent_ok: bool) -> Self {
        let top = Frame {
            context: Rc::clone(&context),
            parent_ok
        };
        let mut frames = self.frames.clone();
        frames.push(top);
        Stack { frames }
    }

    fn merge(&self, stack: Stack<'a>) -> Self {
        let first = stack.frames.first().unwrap();
        let rest = &stack.frames[1..];
        let bridge = Frame {
            context: Rc::clone(&first.context),
            parent_ok: true
        };
        let mut frames = self.frames.clone();
        frames.push(bridge);
        frames.extend_from_slice(rest);
        Stack { frames }
    }

    fn top(&self) -> &Frame<'a> {
        &self.frames.last().unwrap()
    }

    fn context(&self) -> &RcContext<'a> {
        &self.top().context
    }

    fn parent(&self) -> Option<Self> {
        if self.top().parent_ok {
            let frames = Vec::from(
                &self.frames[..self.frames.len() - 1]
            );
            Some(Stack { frames })
        } else {
            None
        }
    }

    fn push_from_parent(&self, name: &str, onto: &Stack<'a>) -> PushResult<'a> {
        if let Some(parent) = self.parent() {
            match parent.push(name) {
                PushResult::Single(stack) =>
                    PushResult::Single(
                        onto.merge(stack)
                    ),
                PushResult::List(stacks) =>
                    PushResult::List(
                        stacks.into_iter()
                            .map(|stack| onto.merge(stack))
                            .collect::<_>()
                    ),
                PushResult::None =>
                    parent.push_from_parent(name, onto)
            }
        } else {
            PushResult::None
        }
    }

    fn push_obj_or_list(&self, context: RcContext<'a>) -> PushResult<'a> {
        let children = context.children();
        if children.is_empty() {
            PushResult::Single(self.extend(context, true))
        } else {
            PushResult::List(
                children.into_iter()
                    .map(|context| self.extend(context, true))
                    .collect::<_>()
            )
        }
    }

    pub(crate) fn push(&self, name: &str) -> PushResult<'a> {
        if name == "." {
            let children = self.context().children();
            if children.is_empty() {
                PushResult::Single(self.clone())
            } else {
                PushResult::List(
                    children.into_iter()
                        .map(|context| self.extend(context, true))
                        .collect()
                )
            }
        } else if let Some(idx) = name.find(".") {
            match self.context().child(&name[..idx]) {
                Some(context) => self.extend(context, false).push(&name[idx + 1..]),
                _ => self.push_from_parent(name, self)
            }
        } else {
            match self.context().child(name) {
                Some(context) => self.push_obj_or_list(context),
                _ => self.push_from_parent(name, self)
            }
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        if name == "." {
            self.value()
        } else if let PushResult::Single(item) = self.push(name) {
            item.value()
        } else {
            None
        }
    }

    pub(crate) fn is_truthy(&self) -> bool {
        self.context().is_truthy()
    }

    fn value(&self) -> Option<String> {
        self.context().value()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsonValue};

    #[test]
    fn single_value() {
        let json = json1();
        let root = Stack::root(&json);

        assert_eq!(
            root.get("name"),
            Some(String::from("John Doe"))
        );
        assert_eq!(
            root.get("age"),
            Some(String::from("43"))
        );
        assert_eq!(
            root.get("phones"),
            None
        );
        let phones = root.push("phones");
        match phones {
            PushResult::List(seq) =>
                assert_eq!(
                    seq.into_iter()
                        .map(|s| s.value().unwrap())
                        .collect::<Vec<String>>(),
                    vec![
                        String::from("+44 1234567"),
                        String::from("+44 2345678")
                    ]
                ),
            _ => assert!(false)
        }
    }

    fn json1() -> JsonValue {
        let data = r#"
        {
            "name": "John Doe",
            "age": 43,
            "phones": [
                "+44 1234567",
                "+44 2345678"
            ]
        }"#;
        serde_json::from_str::<JsonValue>(data).unwrap()
    }
}
