# rsmustache

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust implementation of [Mustache](https://mustache.github.io/mustache.5.html) templates.


## Reference

The reference specification for the Mustache template system is in [Mustache Specification](https://github.com/mustache/spec).
It defines required core modules as well as optional modules.

This implementation passes all standard tests for core modules except *partials*.


## Limitations.

The implementation does not directly support callable in rendered data (sections, values).

Support for *partials*, *~inheritance* and *~dynamic-names* modules is missing - expected later.
Support for the *lambda* module is not expected.


## TODO

Add missing modules
Add API documentation
.../...

## Dependencies

The implementation depends on the standard library and serde for json and YAML.

Noel MINET

2023-05-25
