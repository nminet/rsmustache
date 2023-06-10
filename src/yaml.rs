use crate::{Context, ContextRef};
pub use serde_yaml::Value as YamlValue;


impl<'a> Context<'a> for YamlValue {
    fn child<'b>(&'a self, name: &str, _location: Option<(usize, usize)>) -> Option<ContextRef<'b>>
    where 'a: 'b {
        self.get(name).map(
            |value| value as ContextRef<'b>
        )
    }
    
    fn children<'b>(&'a self) -> Option<Vec<ContextRef<'b>>>
    where 'a: 'b {
        match self {
            YamlValue::Sequence(seq) =>
                Some(
                    seq.iter()
                        .map(|value| value as ContextRef<'b>)
                        .collect::<_>()
                ),
            _ => None
        }
    }

    fn value(&self) -> String {
        match self {
            YamlValue::String(s) => s.clone(),
            YamlValue::Number(n) => n.to_string(),
            YamlValue::Bool(b) => b.to_string(),
            _ => "".to_owned()
        }
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
