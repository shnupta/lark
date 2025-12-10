//! Scripting module - Rhai runtime for configuration and plugins
//!
//! All editor functions are exposed under the `lark` namespace:
//! - `lark::config::*` - settings, themes, keybinds
//! - `lark::editor::*` - buffer operations, cursor, mode (future)
//! - `lark::ui::*` - popups, windows, messages (future)
//! - `lark::fs::*` - file operations (future)
//! - `lark::process::*` - spawn commands (future)

mod api;
mod engine;

pub use engine::ScriptEngine;
