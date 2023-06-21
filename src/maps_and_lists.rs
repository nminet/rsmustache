use std::{collections::HashMap, cell::RefCell, rc::Rc};
use crate::context::{Context, ContextRef, ContextValue};


/// Minimun [Context] implementation.
/// 
/// The [MapsAndLists] context is intended as a bridge between application
/// data and rendering, when access to data can be anticipated thus making
/// eager copy of source data suitable.
/// Its also supports mustache lambda wherein the context can emit a template
/// that must be processed with the current stack.
/// 
/// # Sample
/// 
/// ```
/// use mustache::{Template, MapsAndLists};
/// use std::{collections::HashMap, cell::RefCell, rc::Rc};
/// 
/// let source = Rc::from(
///     "{{#wrapped}}hello {{#names}}{{.}}{{sep}}{{/names}}{{/wrapped}}"
/// );
/// let template = Template::from(&source).unwrap();
/// 
/// let names = vec![ "john", "paul", "jacques"];
/// 
/// // in real life code below would be some form of adapter function
/// let counter = RefCell::new(names.len() - 1);
/// let context = MapsAndLists::mapping(
///   vec![
///     (String::from("names"), MapsAndLists::sequence(
///         names.into_iter().map(
///             |n| MapsAndLists::text(n)
///         ).collect::<Vec<_>>()
///     )),
///     (String::from("sep"), MapsAndLists::lambda0(
///         move || {
///             let current = { *counter.borrow() };
///             if current > 0 {
///                 counter.replace(current - 1);
///                 ", ".to_owned()
///             } else {
///                 "".to_owned()
///             }
///         }
///     )),
///     (String::from("wrapped"), MapsAndLists::lambda1(
///         |s| format!("[{}]", s),
///         &source
///     )),
///   ].into_iter().collect::<HashMap<_, _>>()
/// );
/// let result = template.render(&context);
/// 
/// assert_eq!(result, "[hello john, paul, jacques]")
/// ```

pub struct MapsAndLists(Value);

enum Value {
    Null,
    Bool(bool),
    Text(String),
    Mapping(HashMap<String, MapsAndLists>),
    Sequence(Vec<MapsAndLists>),
    Lambda0(Box<dyn Fn() -> String>, RefCell<String>),
    Lambda1(Box<dyn Fn(&str) -> String>, Rc<str>, RefCell<String>),
}

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

    /// Create a mustache lambda.
    /// 
    /// When rendering, `fun` is called each time it is referenced.
    /// 
    /// As per mustache specification, a lambda is considered truthy even
    /// when its result is falsy.
    pub fn lambda0<T>(fun: T) -> MapsAndLists
    where T: Fn() -> String + 'static {
        MapsAndLists(Value::Lambda0(
            Box::new(fun),
            RefCell::new("".to_owned())
        ))
    }

    /// Create a mustache lambda processing a section.
    /// 
    /// When rendering, `fun` is called each time it is referenced.
    /// 
    /// `fun` receives a slice of `template` and must return a new mustache
    /// text that will be compiled and rendered.
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

impl Context for MapsAndLists{
    fn child(&self, name: &str, section: Option<(usize, usize)>) -> Option<ContextRef> {
        match self {
            MapsAndLists(Value::Mapping(obj)) =>
                obj.get(name).map(
                    |it| {
                        it.process_lambda(&section);
                        it as ContextRef
                    }
                ),
            _ => None
        }
    }

    fn children(&self) -> Option<Vec<ContextRef>> {
        match self {
            MapsAndLists(Value::Sequence(seq)) =>
                Some(
                    seq.iter().map(
                        |it| it as ContextRef
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
            MapsAndLists(Value::Text(t)) => t.is_empty(),
            _ => false
        }
    }
}
