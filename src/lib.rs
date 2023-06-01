//! A complete Mustache implementation with no added features.
//! 
//! This implementation support all core module as well as inheritance
//! and dynamic-partials, passing relevant tests in [`specs`].
//! 
//! 
//! 
//! [`specs`]: https://github.com/mustache/spec
mod template;
mod reader;
mod context;
mod json;
mod yaml;

pub use self::template::{Template, TemplateStore, TemplateMap};
pub use self::context::{Context, ContextRef};
pub use self::json::JsonValue;
pub use self::yaml::YamlValue;
