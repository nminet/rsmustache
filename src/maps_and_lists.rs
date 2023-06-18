use std::{collections::HashMap, cell::RefCell, rc::Rc};
use crate::context::{Context, ContextRef, ContextValue};


enum Value {
    Null,
    Bool(bool),
    Text(String),
    Mapping(HashMap<String, MapsAndLists>),
    Sequence(Vec<MapsAndLists>),
    Lambda0(Box<dyn Fn() -> String>, RefCell<String>),
    Lambda1(Box<dyn Fn(&str) -> String>, Rc<str>, RefCell<String>),
}

pub struct MapsAndLists(Value);

impl MapsAndLists {
    pub fn null() -> MapsAndLists {
        MapsAndLists(Value::Null)
    }

    pub fn bool(b: bool) -> MapsAndLists {
        MapsAndLists(Value::Bool(b))
    }

    pub fn text(t: &str) -> MapsAndLists {
        MapsAndLists(Value::Text(t.to_owned()))
    }

    pub fn mapping(mapping: HashMap<String, MapsAndLists>) -> MapsAndLists {
        MapsAndLists(Value::Mapping(mapping))
    }

    pub fn sequence(sequence: Vec<MapsAndLists>) -> MapsAndLists {
        MapsAndLists(Value::Sequence(sequence))
    }

    pub fn lambda0<T>(fun: T) -> MapsAndLists
    where T: Fn() -> String + 'static {
        MapsAndLists(Value::Lambda0(
            Box::new(fun),
            RefCell::new("".to_owned())
        ))
    }

    pub fn lambda1<T>(fun: T, template: &Rc<str>) -> MapsAndLists
    where T: Fn(&str) -> String + 'static {
        MapsAndLists(Value::Lambda1(
            Box::new(fun),
            Rc::clone(&template),
            RefCell::new("".to_owned())
        ))
    }

    fn process_lambda(&self, section: &Option<(usize, usize)>) {
       match self {
            MapsAndLists(Value::Lambda0(lambda, result)) => {
                result.replace(lambda());
            },
            MapsAndLists(Value::Lambda1(lambda, template, result)) => {
                let (start, end) = section.unwrap();
                result.replace(lambda(&template[start..end]));
            },
            _ => {}
        };
    }
}

impl<'a> Context<'a> for MapsAndLists{
    fn child<'b>(&'a self, name: &str, section: Option<(usize, usize)>) -> Option<ContextRef<'b>>
    where 'a: 'b {
        match self {
            MapsAndLists(Value::Mapping(obj)) =>
                obj.get(name).map(
                    |it| {
                        it.process_lambda(&section);
                        it as ContextRef<'b>
                    }
                ),
            _ => None
        }
    }

    fn children<'b>(&'a self) -> Option<Vec<ContextRef<'b>>>
    where 'a: 'b {
        match self {
            MapsAndLists(Value::Sequence(seq)) =>
                Some(
                    seq.iter().map(
                        |it| it as ContextRef<'b>
                    ).collect::<Vec<_>>()
                ),
            _ => None
        }
    }

    fn value(&self) -> ContextValue {
        match self {
            MapsAndLists(Value::Bool(b)) => ContextValue::Text(b.to_string()),
            MapsAndLists(Value::Text(text)) => ContextValue::Text(text.to_owned()),
            MapsAndLists(Value::Lambda0(_, result)) => ContextValue::Lambda(
                result.borrow().clone()
            ),
            MapsAndLists(Value::Lambda1(_, _, result)) => ContextValue::Lambda(
                result.borrow().clone()
            ),
            _ => ContextValue::Text("".to_owned())
        }
    }

    fn is_falsy(&self) -> bool {
        match self {
            MapsAndLists(Value::Null) => true,
            MapsAndLists(Value::Bool(b)) => !b,
            _ => false
        }
    }
}
