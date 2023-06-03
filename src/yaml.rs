use crate::{Context, ContextRef};
pub use serde_yaml::Value as YamlValue;


impl<'a> Context<'a> for YamlValue {
    fn child(&'a self, name: &str) -> Option<ContextRef<'a>> {
        self.get(name).map(
            |value| value as ContextRef<'a>
        )
    }
    
    fn children(&'a self) -> Option<Vec<ContextRef<'a>>> {
        match self {
            YamlValue::Sequence(seq) =>
                Some(
                    seq.iter()
                        .map(|value| value as ContextRef<'a>)
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

    fn is_falsy(&self) -> bool {
        match self {
            YamlValue::Null => true,
            YamlValue::Bool(b) => !*b,
            _ => false
        }
    }
}
