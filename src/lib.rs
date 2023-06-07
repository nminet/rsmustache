//! A minimal near-complete Mustache implementation.
//! 
//! This implementation support all core module as well as inheritance
//! and dynamic-partials, passing relevant tests in [`specs`].
//! 
//! It does NOT support features where the data to be rendered contains
//! code that must be executed by the rendering engine (in other words 
//! there is no internal script engine).
//! 
//! However the data source feeding the template - an implementation of
//! the [`crate::Context`] trait - executes code producing input for
//! the template. The stack navigation then provides a feedback from
//! the (fixed) template into the data source.
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
