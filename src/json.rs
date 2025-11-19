use crate::{Context, ContextValue, ContextRef, ContextRefIterator};
pub use serde_json::Value as JsonValue;


impl Context for JsonValue {
    fn child(&self, name: &str, _location: Option<(usize, usize)>) -> Option<ContextRef<'_>> {
        self.get(name).map(
            |value| value as ContextRef
        )
    }

    fn children(&self) -> Option<ContextRefIterator<'_>> {
        match self {
            JsonValue::Array(seq) => 
                Some(Box::new(seq.iter().map(|value| value as ContextRef))),
            _ => None
        }
    }

    fn value(&self) -> ContextValue {
        let text = match self {
            JsonValue::String(s) => s.clone(),
            JsonValue::Number(n) => n.to_string(),
            JsonValue::Bool(b) => b.to_string(),
            _ => "".to_owned()
        };
        ContextValue::Text(text)
    }

    /// Falsy indicator.
    /// 
    /// Falsy values in this implementation are:
    /// - null
    /// - boolean false
    /// - empty string
    /// - number 0 (int or float)
    /// 
    /// All other values are truthy.
    fn is_falsy(&self) -> bool {
        match self {
            JsonValue::Null => true,
            JsonValue::String(s) => s.is_empty(),
            JsonValue::Number(n) => {
                if let Some(u) = n.as_u64() {
                    u == 0
                } else if let Some(f) = n.as_f64() {
                    f == 0.0
                } else if let Some(i) = n.as_i64() {
                    i == 0
                } else {
                    false
                }
            },
            JsonValue::Bool(b) => !*b,
            _ => false
        }
    }
}
