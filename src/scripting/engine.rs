//! The main Rhai scripting engine for Lark
//!
//! Provides the `lark` namespace with all editor APIs:
//! - `lark::config::*` - configuration and settings
//! - `lark::editor::*` - buffer/cursor operations (future)
//! - `lark::ui::*` - UI elements like popups (future)

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use rhai::{AST, Engine, Scope};

use super::api;
use crate::config::Settings;

/// The main scripting engine for Lark
pub struct ScriptEngine {
    engine: Engine,
    settings: Arc<RwLock<Settings>>,
    ast: Option<AST>,
}

impl ScriptEngine {
    /// Create a new script engine with fresh settings
    pub fn new() -> Self {
        let settings = Arc::new(RwLock::new(Settings::default()));
        let engine = Self::create_engine(Arc::clone(&settings));

        Self {
            engine,
            settings,
            ast: None,
        }
    }

    /// Create the Rhai engine with the `lark` namespace
    fn create_engine(settings: Arc<RwLock<Settings>>) -> Engine {
        let mut engine = Engine::new();

        // Safety limits
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(100_000);

        // Create the `lark` module as a static namespace
        let mut lark_module = rhai::Module::new();

        // Register lark::config submodule
        let config_module = api::config::create_module(Arc::clone(&settings));
        lark_module.set_sub_module("config", config_module);

        // Future: Register other submodules
        // lark_module.set_sub_module("editor", api::editor::create_module(...));
        // lark_module.set_sub_module("ui", api::ui::create_module(...));
        // lark_module.set_sub_module("fs", api::fs::create_module(...));

        // Register `lark` as a static module (accessible as lark::*)
        engine.register_static_module("lark", lark_module.into());

        // Utility function for debugging
        engine.register_fn("print", |msg: &str| {
            // TODO: Log to status line or file
            eprintln!("[rhai] {}", msg);
        });

        engine
    }

    /// Load and execute a config file
    pub fn load_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read script file: {}", e))?;

        self.eval(&content)
    }

    /// Evaluate a Rhai script string
    pub fn eval(&mut self, script: &str) -> Result<(), String> {
        let ast = self
            .engine
            .compile(script)
            .map_err(|e| format!("Script parse error: {}", e))?;

        let mut scope = Scope::new();
        self.engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| format!("Script error: {}", e))?;

        self.ast = Some(ast);
        Ok(())
    }

    /// Get the current settings (cloned)
    pub fn settings(&self) -> Settings {
        self.settings.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get a reference to the settings for sharing
    pub fn settings_ref(&self) -> Arc<RwLock<Settings>> {
        Arc::clone(&self.settings)
    }

    /// Get the config directory path
    /// Uses ~/.config/lark/ on all platforms for consistency
    pub fn config_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|p| p.join(".config").join("lark"))
    }

    /// Get the default config file path
    pub fn config_file() -> Option<PathBuf> {
        Self::config_dir().map(|p| p.join("init.rhai"))
    }

    /// Load the default config file if it exists
    pub fn load_default(&mut self) -> Result<(), String> {
        if let Some(config_file) = Self::config_file() {
            if config_file.exists() {
                return self.load_file(&config_file);
            }
        }
        Ok(()) // No config file is fine
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lark_config_set_theme() {
        let mut engine = ScriptEngine::new();
        engine.eval(r#"lark::config::set_theme("nord");"#).unwrap();
        assert_eq!(engine.settings().theme, "nord");
    }

    #[test]
    fn test_lark_config_set_tab_width() {
        let mut engine = ScriptEngine::new();
        engine.eval("lark::config::set_tab_width(2);").unwrap();
        assert_eq!(engine.settings().tab_width, 2);
    }

    #[test]
    fn test_lark_config_bind() {
        let mut engine = ScriptEngine::new();
        engine
            .eval(r#"lark::config::bind("<leader>w", "save");"#)
            .unwrap();
        let settings = engine.settings();
        assert_eq!(
            settings.keybinds.get("<leader>w"),
            Some(&"save".to_string())
        );
    }

    #[test]
    fn test_lark_config_multiple() {
        let mut engine = ScriptEngine::new();
        engine
            .eval(
                r#"
                lark::config::set_theme("dracula");
                lark::config::set_tab_width(4);
                lark::config::set_relative_line_numbers(false);
                lark::config::set_auto_indent(true);
            "#,
            )
            .unwrap();

        let settings = engine.settings();
        assert_eq!(settings.theme, "dracula");
        assert_eq!(settings.tab_width, 4);
        assert!(!settings.relative_line_numbers);
        assert!(settings.auto_indent);
    }

    #[test]
    fn test_lark_config_list_themes() {
        let mut engine = ScriptEngine::new();
        // Verify list_themes returns an array with expected themes
        engine
            .eval(
                r#"
                let themes = lark::config::list_themes();
                if themes.len() == 0 {
                    throw "No themes returned";
                }
            "#,
            )
            .unwrap();
    }
}
