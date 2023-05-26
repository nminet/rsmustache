use crate::{Context, ContextRef};
pub use serde_json::Value as JsonValue;


impl<'a> Context<'a> for JsonValue {
    fn child(&'a self, name: &str) -> Option<ContextRef<'a>> {
        self.get(name).map(
            |value| value as ContextRef<'a>
        )
    }
    
    fn children(&'a self) -> Option<Vec<ContextRef<'a>>> {
        match self {
            JsonValue::Array(seq) =>
                Some(
                    seq.iter()
                        .map(|value| value as ContextRef<'a>)
                        .collect::<Vec<_>>()
                ),
            _ => None
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
