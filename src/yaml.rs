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

    fn value(&self) -> Option<String> {
        match self {
            YamlValue::String(s) => Some(s.clone()),
            YamlValue::Number(n) => Some(n.to_string()),
            YamlValue::Bool(b) => Some(b.to_string()),
            YamlValue::Null => Some(String::new()),
            _ => None
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            YamlValue::Null => false,
            YamlValue::Bool(b) => *b,
            YamlValue::Sequence(seq) => !seq.is_empty(),
            _ => true
        }
    }
}
