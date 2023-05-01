# Changelog

Merlon adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html). This means that if a change is made that
breaks backward-compatibility, the major version number will be incremented. This applies for both the Merlon
application and the Rust / Python APIs.

## 2.0.0

- Package IDs have been changed to be kebab-case strings rather than UUIDs. Package IDs must now be in kebab-case, no
less than 3 characters, and no more than 64 characters. This is to allow for more human-readable package IDs, especially
as Git branch names and directory names. **Rust API breaking change.**
