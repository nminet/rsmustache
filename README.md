# rsmustache

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust implementation of [Mustache](https://mustache.github.io/mustache.5.html) templates.


## Reference

The reference specification for the Mustache template system is in [Mustache Specification](https://github.com/mustache/spec).
It defines required core modules as well as optional modules.

This implementation passes all standard tests for core modules as well as the *inheritance* and *dynamic-names* optional modules.


## Limitations.

The implementation does not directly support callables producing contexts. It can be implemented outside the crate as a specialization of the **Context** trait.

Support for the *lambda* module is not expected.


## TODO

- Add API documentation
- .../...

## Dependencies

The implementation depends on the standard library and serde for json and YAML.

Noel MINET

2023-05-30
