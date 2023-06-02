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

    fn value(&self) -> String {
        match self {
            JsonValue::String(s) => s.clone(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Bool(b) => b.to_string(),
            _ => "".to_owned()
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            JsonValue::Null => false,
            JsonValue::Bool(b) => *b,
            _ => true
        }
    }
}
