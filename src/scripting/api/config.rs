//! lark::config - Settings, themes, and keybinds
//!
//! Usage in Rhai:
//! ```rhai
//! lark::config::set_theme("nord");
//! lark::config::set_tab_width(4);
//! lark::config::bind("<leader>w", "save");
//! ```

use rhai::plugin::*;
use std::sync::{Arc, RwLock};

use crate::config::Settings;

/// Create the config module with access to settings
pub fn create_module(settings: Arc<RwLock<Settings>>) -> rhai::Module {
    let mut module = rhai::Module::new();

    // set_theme(name: &str)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_theme", move |name: &str| {
            if let Ok(mut settings) = s.write() {
                settings.theme = name.to_string();
            }
            Ok(())
        });
    }

    // get_theme() -> String
    {
        let s = Arc::clone(&settings);
        module.set_native_fn(
            "get_theme",
            move || -> Result<String, Box<EvalAltResult>> {
                Ok(s.read().map(|s| s.theme.clone()).unwrap_or_default())
            },
        );
    }

    // set_tab_width(width: i64)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_tab_width", move |width: i64| {
            if let Ok(mut settings) = s.write() {
                settings.tab_width = width.max(1).min(16) as usize;
            }
            Ok(())
        });
    }

    // get_tab_width() -> i64
    {
        let s = Arc::clone(&settings);
        module.set_native_fn(
            "get_tab_width",
            move || -> Result<i64, Box<EvalAltResult>> {
                Ok(s.read().map(|s| s.tab_width as i64).unwrap_or(4))
            },
        );
    }

    // set_relative_line_numbers(enabled: bool)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_relative_line_numbers", move |enabled: bool| {
            if let Ok(mut settings) = s.write() {
                settings.relative_line_numbers = enabled;
            }
            Ok(())
        });
    }

    // set_show_line_numbers(enabled: bool)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_show_line_numbers", move |enabled: bool| {
            if let Ok(mut settings) = s.write() {
                settings.show_line_numbers = enabled;
            }
            Ok(())
        });
    }

    // set_auto_indent(enabled: bool)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_auto_indent", move |enabled: bool| {
            if let Ok(mut settings) = s.write() {
                settings.auto_indent = enabled;
            }
            Ok(())
        });
    }

    // set_insert_spaces(enabled: bool)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_insert_spaces", move |enabled: bool| {
            if let Ok(mut settings) = s.write() {
                settings.insert_spaces = enabled;
            }
            Ok(())
        });
    }

    // set_show_hidden_files(enabled: bool)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("set_show_hidden_files", move |enabled: bool| {
            if let Ok(mut settings) = s.write() {
                settings.show_hidden_files = enabled;
            }
            Ok(())
        });
    }

    // bind(key: &str, action: &str)
    {
        let s = Arc::clone(&settings);
        module.set_native_fn("bind", move |key: &str, action: &str| {
            if let Ok(mut settings) = s.write() {
                settings
                    .keybinds
                    .insert(key.to_string(), action.to_string());
            }
            Ok(())
        });
    }

    // list_themes() -> Array
    module.set_native_fn(
        "list_themes",
        || -> Result<rhai::Array, Box<EvalAltResult>> {
            let themes = crate::theme::list_builtin_themes();
            Ok(themes
                .into_iter()
                .map(|s| rhai::Dynamic::from(s.to_string()))
                .collect())
        },
    );

    module
}
