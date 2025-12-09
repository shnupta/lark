use std::collections::HashMap;

/// Editor settings that can be customized via Rhai config
#[derive(Debug, Clone)]
pub struct Settings {
    // Display
    pub theme: String,
    pub show_line_numbers: bool,
    pub relative_line_numbers: bool,
    pub tab_width: usize,
    pub show_whitespace: bool,

    // Editing
    pub auto_indent: bool,
    pub insert_spaces: bool, // Use spaces instead of tabs

    // File browser
    pub file_browser_width: u16,
    pub show_hidden_files: bool,

    // Custom keybinds: key sequence -> action name
    pub keybinds: HashMap<String, String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "gruvbox-dark".to_string(),
            show_line_numbers: true,
            relative_line_numbers: true,
            tab_width: 4,
            show_whitespace: false,

            auto_indent: true,
            insert_spaces: true,

            file_browser_width: 30,
            show_hidden_files: false,

            keybinds: HashMap::new(),
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}
