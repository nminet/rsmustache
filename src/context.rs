use std::fmt::Debug;
use std::rc::Rc;


pub trait Context<'a>: Debug {
    fn child(&self, name: &str) -> Option<Boxed<'a>>;
    fn children(&self) -> Vec<Boxed<'a>>;
    fn value(&self) -> Option<String>;
}

pub type Boxed<'a> = Box<dyn Context<'a> + 'a>;

pub(crate) fn into_box<'a, T>(context: &'a T) -> Boxed<'a>
where &'a T: Context<'a> {
    Box::new(context)
}


#[derive(Debug)]
pub(crate) struct Stack<'a> {
    context: Rc<Boxed<'a>>,
    parent: Option<Rc<Boxed<'a>>>
}

pub(crate) enum PushResult<'a> {
    None,
    Single(Stack<'a>),
    List(Vec<Stack<'a>>)
}


impl<'a> Stack<'a> {
    pub(crate) fn root<T>(context: &'a T) -> Self
    where &'a T: Context<'a> {
        Stack {
            context: Rc::new(into_box(context)),
            parent: None
        }
    }

    fn frame(&self, context: Boxed<'a>) -> Self {
        Stack {
            context: Rc::new(context),
            parent: Some(Rc::clone(&self.context))
        }
    }

    pub(crate) fn push(&self, name: &str) -> PushResult<'a> {
        match self.context.child(name) {
            Some(context) => {
                let children = context.children();
                if children.is_empty() {
                    PushResult::Single(self.frame(context))
                } else {
                    PushResult::List(
                        children.into_iter()
                            .map(|context| self.frame(context))
                            .collect::<Vec<Self>>()
                    )
                }
            },
            _ => PushResult::None
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

    pub(crate) fn value(&self) -> Option<String> {
        self.context.value()
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
