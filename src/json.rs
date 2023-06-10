use crate::{Context, ContextRef};
pub use serde_json::Value as JsonValue;


impl<'a> Context<'a> for JsonValue {
    fn child<'b>(&'a self, name: &str, _location: Option<(usize, usize)>) -> Option<ContextRef<'b>>
    where 'a: 'b {
        self.get(name).map(
            |value| value as ContextRef<'b>
        )
    }
    
    fn children<'b>(&'a self) -> Option<Vec<ContextRef<'b>>>
    where 'a: 'b {
        match self {
            JsonValue::Array(seq) =>
                Some(
                    seq.iter()
                        .map(|value| value as ContextRef<'b>)
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
            JsonValue::Number(n) =>
                n.is_u64() && n.as_u64().unwrap() == 0 ||
                n.is_f64() && n.as_f64().unwrap() == 0f64,
            JsonValue::Bool(b) => !*b,
            _ => false
        }
    }
}
