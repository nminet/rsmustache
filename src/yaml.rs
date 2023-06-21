use crate::{Context, ContextValue, ContextRef};
pub use serde_yaml::Value as YamlValue;


impl Context for YamlValue {
    fn child(&self, name: &str, _location: Option<(usize, usize)>) -> Option<ContextRef> {
        self.get(name).map(
            |value| value as ContextRef
        )
    }
    
    fn children(&self) -> Option<Vec<ContextRef>> {
        match self {
            YamlValue::Sequence(seq) =>
                Some(
                    seq.iter()
                        .map(|value| value as ContextRef)
                        .collect::<_>()
                ),
            _ => None
        }
    }

    fn value(&self) -> ContextValue {
        let text = match self {
            YamlValue::String(s) => s.clone(),
            YamlValue::Number(n) => n.to_string(),
            YamlValue::Bool(b) => b.to_string(),
            _ => "".to_owned()
        };
        ContextValue::Text(text)
    }

    /// Falsy indicator.
    /// 
    /// Falsy values in this implementation are:
    /// - null
    /// - boolean false
    /// 
    /// All other values are truthy.
    fn is_falsy(&self) -> bool {
        match self {
            YamlValue::Null => true,
            YamlValue::Bool(b) => !*b,
            _ => false
        }
    }
}
