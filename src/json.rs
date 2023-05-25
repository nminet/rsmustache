use crate::{Context, RcContext, into_rc};
pub use serde_json::Value as JsonValue;


impl<'a> Context<'a> for &'a JsonValue {
    fn child(&self, name: &str) -> Option<RcContext<'a>> {
        self.get(name)
            .map(into_rc)
    }
    
    fn children(&self) -> Option<Vec<RcContext<'a>>> {
        match self {
            JsonValue::Array(seq) =>
                Some(
                    seq.iter()
                        .map(into_rc)
                        .collect::<_>()
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
