use crate::context::{Context, Boxed, into_box};
pub use serde_json::Value as JsonValue;


impl<'a> Context<'a> for &'a JsonValue {
    fn child(&self, name: &str) -> Option<Boxed<'a>> {
        self.get(name)
            .map(into_box)
    }
    
    fn children(&self) -> Vec<Boxed<'a>> {
        match self {
            JsonValue::Array(seq) =>
                seq.iter()
                    .map(into_box)
                    .collect::<_>(),
            _ => vec![]
        }
    }

    fn value(&self) -> Option<String> {
        match self {
            JsonValue::String(s) => Some(s.clone()),
            JsonValue::Number(n) => Some(n.to_string()),
            JsonValue::Bool(b) => Some(b.to_string()),
            JsonValue::Null => Some(String::new()),
            _ => None
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            JsonValue::Null => true,
            JsonValue::Bool(b) => *b,
            JsonValue::Array(seq) => !seq.is_empty(),
            _ => true
        }
    }
}
