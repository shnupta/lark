use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use rhai::{AST, Engine, Scope};

use super::Settings;

/// The Rhai scripting engine for configuration
pub struct ConfigEngine {
    engine: Engine,
    settings: Arc<RwLock<Settings>>,
    ast: Option<AST>,
}

impl ConfigEngine {
    pub fn new() -> Self {
        let settings = Arc::new(RwLock::new(Settings::default()));
        let engine = Self::create_engine(Arc::clone(&settings));

        Self {
            engine,
            settings,
            ast: None,
        }
    }

    fn create_engine(settings: Arc<RwLock<Settings>>) -> Engine {
        let mut engine = Engine::new();

        // Limit script execution for safety
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(100_000);

        // Register settings functions
        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_theme", move |name: &str| {
                if let Ok(mut settings) = s.write() {
                    settings.theme = name.to_string();
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("get_theme", move || -> String {
                s.read().map(|s| s.theme.clone()).unwrap_or_default()
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_tab_width", move |width: i64| {
                if let Ok(mut settings) = s.write() {
                    settings.tab_width = width.max(1).min(16) as usize;
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_relative_line_numbers", move |enabled: bool| {
                if let Ok(mut settings) = s.write() {
                    settings.relative_line_numbers = enabled;
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_show_line_numbers", move |enabled: bool| {
                if let Ok(mut settings) = s.write() {
                    settings.show_line_numbers = enabled;
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_auto_indent", move |enabled: bool| {
                if let Ok(mut settings) = s.write() {
                    settings.auto_indent = enabled;
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_insert_spaces", move |enabled: bool| {
                if let Ok(mut settings) = s.write() {
                    settings.insert_spaces = enabled;
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("set_show_hidden_files", move |enabled: bool| {
                if let Ok(mut settings) = s.write() {
                    settings.show_hidden_files = enabled;
                }
            });
        }

        {
            let s = Arc::clone(&settings);
            engine.register_fn("bind", move |key: &str, action: &str| {
                if let Ok(mut settings) = s.write() {
                    settings
                        .keybinds
                        .insert(key.to_string(), action.to_string());
                }
            });
        }

        // Utility functions
        engine.register_fn("print", |msg: &str| {
            // For now, just ignore print statements
            // Later we could log them or show in a message
            let _ = msg;
        });

        engine
    }

    /// Load and execute a config file
    pub fn load_file(&mut self, path: &PathBuf) -> Result<(), String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        self.eval(&content)
    }

    /// Evaluate a Rhai script string
    pub fn eval(&mut self, script: &str) -> Result<(), String> {
        let ast = self
            .engine
            .compile(script)
            .map_err(|e| format!("Config parse error: {}", e))?;

        let mut scope = Scope::new();
        self.engine
            .run_ast_with_scope(&mut scope, &ast)
            .map_err(|e| format!("Config error: {}", e))?;

        self.ast = Some(ast);
        Ok(())
    }

    /// Get the current settings (cloned)
    pub fn settings(&self) -> Settings {
        self.settings.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get the config directory path
    pub fn config_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("lark"))
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

impl Default for ConfigEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_theme() {
        let mut engine = ConfigEngine::new();
        engine.eval(r#"set_theme("nord");"#).unwrap();
        assert_eq!(engine.settings().theme, "nord");
    }

    #[test]
    fn test_set_tab_width() {
        let mut engine = ConfigEngine::new();
        engine.eval("set_tab_width(2);").unwrap();
        assert_eq!(engine.settings().tab_width, 2);
    }

    #[test]
    fn test_set_tab_width_clamped() {
        let mut engine = ConfigEngine::new();
        engine.eval("set_tab_width(100);").unwrap();
        assert_eq!(engine.settings().tab_width, 16); // Clamped to max
    }

    #[test]
    fn test_bind_key() {
        let mut engine = ConfigEngine::new();
        engine.eval(r#"bind("<leader>w", "save");"#).unwrap();
        let settings = engine.settings();
        assert_eq!(
            settings.keybinds.get("<leader>w"),
            Some(&"save".to_string())
        );
    }

    #[test]
    fn test_multiple_settings() {
        let mut engine = ConfigEngine::new();
        engine
            .eval(
                r#"
                set_theme("dracula");
                set_tab_width(4);
                set_relative_line_numbers(false);
                set_auto_indent(true);
            "#,
            )
            .unwrap();

        let settings = engine.settings();
        assert_eq!(settings.theme, "dracula");
        assert_eq!(settings.tab_width, 4);
        assert!(!settings.relative_line_numbers);
        assert!(settings.auto_indent);
    }
}
