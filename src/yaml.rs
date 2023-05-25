use crate::{Context, RcContext, into_rc};
pub use serde_yaml::Value as YamlValue;


impl<'a> Context<'a> for &'a YamlValue {
    fn child(&self, name: &str) -> Option<RcContext<'a>> {
        self.get(name)
            .map(into_rc)
    }
    
    fn children(&self) -> Option<Vec<RcContext<'a>>> {
        match self {
            YamlValue::Sequence(seq) =>
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
