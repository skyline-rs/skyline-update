# skyline-update

A server/library combo for updating skyline plugins written in Rust.

Basic library usage:

```rust
// Get a stable update of plugin "plugin_name" from IP `127.0.0.1` using the version in `Cargo.toml` to display the version of the current plugin
skyline_update::check_update("127.0.0.1".parse().unwrap(), "plugin_name", env!("CARGO_PKG_VERSION"), false);
```
