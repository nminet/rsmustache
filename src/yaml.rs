use crate::context::{Context, IntoContext};
pub use serde_yaml::Value as YamlValue;


fn get_context<'a, 'b>(obj: &'a YamlValue, k: &'b str) -> Option<Context<'a>> {
    obj.get(&k).map(|v| v.into_context())
}


impl<'a> IntoContext for YamlValue {
    fn into_context(&self) -> Context {
        match self {
            YamlValue::Null => Context::Value(String::new()),
            YamlValue::Bool(b) => Context::Value(b.to_string()),
            YamlValue::Number(n) => Context::Value(n.to_string()),
            YamlValue::String(s) => Context::Value(s.to_string()),
            YamlValue::Sequence(seq) => Context::Iterable(
                seq.iter()
                    .map(|v| v.into_context())
                    .collect::<Vec<Context>>()
            ),
            YamlValue::Mapping(_) => Context::Object(
                Box::new(|k| get_context(self, k))
            ),
            YamlValue::Tagged(b) => b.value.into_context()
        }
    }
}
