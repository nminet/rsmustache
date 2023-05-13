use std::fmt::Debug;
use std::rc::Rc;


pub trait Context<'a>: Debug {
    fn child(&self, name: &str) -> Option<Boxed<'a>>;
    fn children(&self) -> Vec<Boxed<'a>>;
    fn value(&self) -> Option<String>;
    fn is_truthy(&self) -> bool;
}

pub type Boxed<'a> = Rc<dyn Context<'a> + 'a>;

pub(crate) fn into_box<'a, T>(context: &'a T) -> Boxed<'a>
where &'a T: Context<'a> {
    Rc::new(context)
}


#[derive(Clone, Debug)]
pub(crate) struct Stack<'a> {
    context: Rc<Boxed<'a>>,
    parent: Option<Rc<Stack<'a>>>
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
            parent: Some(Rc::new(self.clone()))
        }
    }

    fn push_from_parent(&self, name: &str, up_ok: bool) -> PushResult<'a> {
        match &self.parent {
            Some(parent) if up_ok => Rc::clone(&parent).push(name, up_ok),
            _ => PushResult::None
        }
    }

    fn push_obj_or_list(&self, context: Boxed<'a>) -> PushResult<'a> {
        let children = context.children();
        if children.is_empty() {
            PushResult::Single(self.frame(context))
        } else {
            PushResult::List(
                children.into_iter()
                    .map(|context| self.frame(context))
                    .collect::<_>()
            )
        }
    }

    pub(crate) fn push(&self, name: &str, up_ok: bool) -> PushResult<'a> {
        if name == "." {
            let children = self.context.children();
            if children.is_empty() {
                PushResult::Single(self.clone())
            } else {
                PushResult::List(
                    children.into_iter()
                        .map(|context| self.frame(context))
                        .collect()
                )
            }
        } else if let Some(idx) = name.find(".") {
            match self.context.child(&name[..idx]) {
                Some(context) => self.frame(context).push(&name[idx + 1..], false),
                _ => self.push_from_parent(name, up_ok)
            }
        } else {
            match self.context.child(name) {
                Some(context) => self.push_obj_or_list(context),
                _ => self.push_from_parent(name, up_ok)
            }
        }
    }

    pub(crate) fn get(&self, name: &str) -> Option<String> {
        if name == "." {
            self.value()
        } else if let PushResult::Single(item) = self.push(name, true) {
            item.value()
        } else {
            None
        }
    }

    pub(crate) fn is_truthy(&self) -> bool {
        self.context.is_truthy()
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
        let phones = root.push("phones", true);
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
