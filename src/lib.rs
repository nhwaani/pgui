//! pgui library crate.
//!
//! The `src/main.rs` binary is a thin shim around the modules exposed
//! here. Integration tests in `tests/` import items via the `pgui`
//! crate name (`use pgui::services::storage::...`).
//!
//! Anything that needs to be reachable from `tests/` lives in this
//! library crate — that is, everything except the `main()` function
//! and its action bindings.

pub mod assets;
pub mod services;
pub mod state;
pub mod themes;
pub mod window;
pub mod workspace;
