//! A minimal near-complete Mustache implementation.
//! 
//! A mustache [Template] compiled from source is rendered by navigating
//! in a [Context], getting partials from a [TemplateStore].
//! 
//! This implementation support all core module as well as the optional
//! inheritance and dynamic-partials modules, passing relevant tests
//! in [`specs`].
//! 
//! By design there is no mechanism to submit in-data callables.
//! As the [Context] is a trait it is in principle possible to refer to
//! pre-defined functions (possibly even use a section to feed a script engine)
//! but this is NOT part of the crate and would have be be provided by an
//! ad-hoc implementation. To support cases where the function needs it,
//! the [Context] interface receives indices to locate the code of a section
//! being rendered.
//! 
//! 
//! # Samples
//! 
//! ## Hello world
//! 
//! ```
//! use mustache::{Template, JsonValue};
//! 
//! let text = "hello, {{you}}!";
//! let data = r#"{
//!     "you": "world"
//! }"#;
//! 
//! let template = Template::from(text).unwrap();
//! let context = serde_json::from_str::<JsonValue>(data).unwrap();
//! 
//! let result = template.render(&context);
//! 
//! assert_eq!(result, "hello, world!")
//! ```
//! 
//! ## Hello team
//! 
//! ```
//! use mustache::{Template, YamlValue};
//! let text = r#"
//!   {{#team}}
//!   hello, {{address}} {{name}}!
//!   {{/team}}
//! "#;
//! let data = r#"
//!   team:
//!     - name: john
//!       address: little
//!     - name: 42
//!       address: citizen
//! "#;
//! 
//! let template = Template::from(text).unwrap();
//! let context = serde_yaml::from_str::<YamlValue>(data).unwrap();
//! 
//! let result = template.render(&context);
//! assert_eq!(result, r#"
//!   hello, little john!
//!   hello, citizen 42!
//! "#);
//! ```
//! 
//! 
//! [`specs`]: https://github.com/mustache/spec
//! [`Context`]: crate::Context
mod template;
mod reader;
mod context;
mod json;
mod yaml;
mod maps_and_lists;

pub use self::template::{Template, TemplateStore, TemplateMap};
pub use self::context::{Context, ContextValue, ContextRef};
pub use self::json::JsonValue;
pub use self::yaml::YamlValue;
pub use self::maps_and_lists::MapsAndLists;
