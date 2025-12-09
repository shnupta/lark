use super::Color;

/// Style for a UI element (color + optional attributes)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub fg: Color,
    pub bg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub const fn new(fg: Color) -> Self {
        Self {
            fg,
            bg: None,
            bold: false,
            italic: false,
        }
    }

    pub const fn with_bg(mut self, bg: Color) -> Self {
        self.bg = Some(bg);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }

    pub const fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
}

/// Complete theme definition
/// All fields are public for easy Rhai access later
#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,

    // Editor chrome
    pub background: Color,
    pub foreground: Color,
    pub cursor: Color,
    pub selection: Color,

    // UI elements
    pub line_number: Color,
    pub line_number_active: Color,
    pub status_bar_bg: Color,
    pub status_bar_fg: Color,
    pub tab_bar_bg: Color,
    pub tab_bar_fg: Color,
    pub tab_active_bg: Color,
    pub tab_active_fg: Color,

    // File browser
    pub file_browser_bg: Color,
    pub file_browser_dir: Color,
    pub file_browser_file: Color,
    pub file_browser_selected: Color,

    // Pane borders
    pub pane_border: Color,
    pub pane_border_active: Color,

    // Syntax highlighting (for later tree-sitter integration)
    pub syntax_keyword: Style,
    pub syntax_string: Style,
    pub syntax_number: Style,
    pub syntax_comment: Style,
    pub syntax_function: Style,
    pub syntax_type: Style,
    pub syntax_variable: Style,
    pub syntax_operator: Style,
    pub syntax_punctuation: Style,

    // Diagnostics
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub hint: Color,
}

impl Theme {
    /// Gruvbox Dark - warm retro theme
    pub fn gruvbox_dark() -> Self {
        Self {
            name: "gruvbox-dark".to_string(),
            background: Color::from_hex("#282828").unwrap(),
            foreground: Color::from_hex("#ebdbb2").unwrap(),
            cursor: Color::from_hex("#fe8019").unwrap(),
            selection: Color::from_hex("#504945").unwrap(),

            line_number: Color::from_hex("#665c54").unwrap(),
            line_number_active: Color::from_hex("#fabd2f").unwrap(),
            status_bar_bg: Color::from_hex("#3c3836").unwrap(),
            status_bar_fg: Color::from_hex("#ebdbb2").unwrap(),
            tab_bar_bg: Color::from_hex("#1d2021").unwrap(),
            tab_bar_fg: Color::from_hex("#a89984").unwrap(),
            tab_active_bg: Color::from_hex("#3c3836").unwrap(),
            tab_active_fg: Color::from_hex("#ebdbb2").unwrap(),

            file_browser_bg: Color::from_hex("#1d2021").unwrap(),
            file_browser_dir: Color::from_hex("#83a598").unwrap(),
            file_browser_file: Color::from_hex("#ebdbb2").unwrap(),
            file_browser_selected: Color::from_hex("#fe8019").unwrap(),

            pane_border: Color::from_hex("#504945").unwrap(),
            pane_border_active: Color::from_hex("#fe8019").unwrap(),

            syntax_keyword: Style::new(Color::from_hex("#fb4934").unwrap()).bold(),
            syntax_string: Style::new(Color::from_hex("#b8bb26").unwrap()),
            syntax_number: Style::new(Color::from_hex("#d3869b").unwrap()),
            syntax_comment: Style::new(Color::from_hex("#928374").unwrap()).italic(),
            syntax_function: Style::new(Color::from_hex("#fabd2f").unwrap()),
            syntax_type: Style::new(Color::from_hex("#83a598").unwrap()),
            syntax_variable: Color::from_hex("#ebdbb2").unwrap().into(),
            syntax_operator: Color::from_hex("#fe8019").unwrap().into(),
            syntax_punctuation: Color::from_hex("#ebdbb2").unwrap().into(),

            error: Color::from_hex("#fb4934").unwrap(),
            warning: Color::from_hex("#fabd2f").unwrap(),
            info: Color::from_hex("#83a598").unwrap(),
            hint: Color::from_hex("#8ec07c").unwrap(),
        }
    }

    /// Gruvbox Light
    pub fn gruvbox_light() -> Self {
        Self {
            name: "gruvbox-light".to_string(),
            background: Color::from_hex("#fbf1c7").unwrap(),
            foreground: Color::from_hex("#3c3836").unwrap(),
            cursor: Color::from_hex("#d65d0e").unwrap(),
            selection: Color::from_hex("#ebdbb2").unwrap(),

            line_number: Color::from_hex("#a89984").unwrap(),
            line_number_active: Color::from_hex("#b57614").unwrap(),
            status_bar_bg: Color::from_hex("#ebdbb2").unwrap(),
            status_bar_fg: Color::from_hex("#3c3836").unwrap(),
            tab_bar_bg: Color::from_hex("#f2e5bc").unwrap(),
            tab_bar_fg: Color::from_hex("#7c6f64").unwrap(),
            tab_active_bg: Color::from_hex("#ebdbb2").unwrap(),
            tab_active_fg: Color::from_hex("#3c3836").unwrap(),

            file_browser_bg: Color::from_hex("#f2e5bc").unwrap(),
            file_browser_dir: Color::from_hex("#076678").unwrap(),
            file_browser_file: Color::from_hex("#3c3836").unwrap(),
            file_browser_selected: Color::from_hex("#d65d0e").unwrap(),

            pane_border: Color::from_hex("#d5c4a1").unwrap(),
            pane_border_active: Color::from_hex("#d65d0e").unwrap(),

            syntax_keyword: Style::new(Color::from_hex("#9d0006").unwrap()).bold(),
            syntax_string: Style::new(Color::from_hex("#79740e").unwrap()),
            syntax_number: Style::new(Color::from_hex("#8f3f71").unwrap()),
            syntax_comment: Style::new(Color::from_hex("#928374").unwrap()).italic(),
            syntax_function: Style::new(Color::from_hex("#b57614").unwrap()),
            syntax_type: Style::new(Color::from_hex("#076678").unwrap()),
            syntax_variable: Color::from_hex("#3c3836").unwrap().into(),
            syntax_operator: Color::from_hex("#d65d0e").unwrap().into(),
            syntax_punctuation: Color::from_hex("#3c3836").unwrap().into(),

            error: Color::from_hex("#9d0006").unwrap(),
            warning: Color::from_hex("#b57614").unwrap(),
            info: Color::from_hex("#076678").unwrap(),
            hint: Color::from_hex("#427b58").unwrap(),
        }
    }

    /// Nord - arctic, north-bluish color palette
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            background: Color::from_hex("#2e3440").unwrap(),
            foreground: Color::from_hex("#d8dee9").unwrap(),
            cursor: Color::from_hex("#88c0d0").unwrap(),
            selection: Color::from_hex("#434c5e").unwrap(),

            line_number: Color::from_hex("#4c566a").unwrap(),
            line_number_active: Color::from_hex("#d8dee9").unwrap(),
            status_bar_bg: Color::from_hex("#3b4252").unwrap(),
            status_bar_fg: Color::from_hex("#d8dee9").unwrap(),
            tab_bar_bg: Color::from_hex("#2e3440").unwrap(),
            tab_bar_fg: Color::from_hex("#4c566a").unwrap(),
            tab_active_bg: Color::from_hex("#3b4252").unwrap(),
            tab_active_fg: Color::from_hex("#88c0d0").unwrap(),

            file_browser_bg: Color::from_hex("#2e3440").unwrap(),
            file_browser_dir: Color::from_hex("#81a1c1").unwrap(),
            file_browser_file: Color::from_hex("#d8dee9").unwrap(),
            file_browser_selected: Color::from_hex("#88c0d0").unwrap(),

            pane_border: Color::from_hex("#4c566a").unwrap(),
            pane_border_active: Color::from_hex("#88c0d0").unwrap(),

            syntax_keyword: Style::new(Color::from_hex("#81a1c1").unwrap()).bold(),
            syntax_string: Style::new(Color::from_hex("#a3be8c").unwrap()),
            syntax_number: Style::new(Color::from_hex("#b48ead").unwrap()),
            syntax_comment: Style::new(Color::from_hex("#616e88").unwrap()).italic(),
            syntax_function: Style::new(Color::from_hex("#88c0d0").unwrap()),
            syntax_type: Style::new(Color::from_hex("#8fbcbb").unwrap()),
            syntax_variable: Color::from_hex("#d8dee9").unwrap().into(),
            syntax_operator: Color::from_hex("#81a1c1").unwrap().into(),
            syntax_punctuation: Color::from_hex("#eceff4").unwrap().into(),

            error: Color::from_hex("#bf616a").unwrap(),
            warning: Color::from_hex("#ebcb8b").unwrap(),
            info: Color::from_hex("#81a1c1").unwrap(),
            hint: Color::from_hex("#a3be8c").unwrap(),
        }
    }

    /// Dracula - dark theme with vibrant colors
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            background: Color::from_hex("#282a36").unwrap(),
            foreground: Color::from_hex("#f8f8f2").unwrap(),
            cursor: Color::from_hex("#f8f8f2").unwrap(),
            selection: Color::from_hex("#44475a").unwrap(),

            line_number: Color::from_hex("#6272a4").unwrap(),
            line_number_active: Color::from_hex("#f8f8f2").unwrap(),
            status_bar_bg: Color::from_hex("#44475a").unwrap(),
            status_bar_fg: Color::from_hex("#f8f8f2").unwrap(),
            tab_bar_bg: Color::from_hex("#21222c").unwrap(),
            tab_bar_fg: Color::from_hex("#6272a4").unwrap(),
            tab_active_bg: Color::from_hex("#44475a").unwrap(),
            tab_active_fg: Color::from_hex("#bd93f9").unwrap(),

            file_browser_bg: Color::from_hex("#21222c").unwrap(),
            file_browser_dir: Color::from_hex("#bd93f9").unwrap(),
            file_browser_file: Color::from_hex("#f8f8f2").unwrap(),
            file_browser_selected: Color::from_hex("#ff79c6").unwrap(),

            pane_border: Color::from_hex("#44475a").unwrap(),
            pane_border_active: Color::from_hex("#bd93f9").unwrap(),

            syntax_keyword: Style::new(Color::from_hex("#ff79c6").unwrap()).bold(),
            syntax_string: Style::new(Color::from_hex("#f1fa8c").unwrap()),
            syntax_number: Style::new(Color::from_hex("#bd93f9").unwrap()),
            syntax_comment: Style::new(Color::from_hex("#6272a4").unwrap()).italic(),
            syntax_function: Style::new(Color::from_hex("#50fa7b").unwrap()),
            syntax_type: Style::new(Color::from_hex("#8be9fd").unwrap()).italic(),
            syntax_variable: Color::from_hex("#f8f8f2").unwrap().into(),
            syntax_operator: Color::from_hex("#ff79c6").unwrap().into(),
            syntax_punctuation: Color::from_hex("#f8f8f2").unwrap().into(),

            error: Color::from_hex("#ff5555").unwrap(),
            warning: Color::from_hex("#ffb86c").unwrap(),
            info: Color::from_hex("#8be9fd").unwrap(),
            hint: Color::from_hex("#50fa7b").unwrap(),
        }
    }

    /// Solarized Dark
    pub fn solarized_dark() -> Self {
        Self {
            name: "solarized-dark".to_string(),
            background: Color::from_hex("#002b36").unwrap(),
            foreground: Color::from_hex("#839496").unwrap(),
            cursor: Color::from_hex("#268bd2").unwrap(),
            selection: Color::from_hex("#073642").unwrap(),

            line_number: Color::from_hex("#586e75").unwrap(),
            line_number_active: Color::from_hex("#93a1a1").unwrap(),
            status_bar_bg: Color::from_hex("#073642").unwrap(),
            status_bar_fg: Color::from_hex("#839496").unwrap(),
            tab_bar_bg: Color::from_hex("#002b36").unwrap(),
            tab_bar_fg: Color::from_hex("#586e75").unwrap(),
            tab_active_bg: Color::from_hex("#073642").unwrap(),
            tab_active_fg: Color::from_hex("#268bd2").unwrap(),

            file_browser_bg: Color::from_hex("#002b36").unwrap(),
            file_browser_dir: Color::from_hex("#268bd2").unwrap(),
            file_browser_file: Color::from_hex("#839496").unwrap(),
            file_browser_selected: Color::from_hex("#cb4b16").unwrap(),

            pane_border: Color::from_hex("#586e75").unwrap(),
            pane_border_active: Color::from_hex("#268bd2").unwrap(),

            syntax_keyword: Style::new(Color::from_hex("#859900").unwrap()).bold(),
            syntax_string: Style::new(Color::from_hex("#2aa198").unwrap()),
            syntax_number: Style::new(Color::from_hex("#d33682").unwrap()),
            syntax_comment: Style::new(Color::from_hex("#586e75").unwrap()).italic(),
            syntax_function: Style::new(Color::from_hex("#268bd2").unwrap()),
            syntax_type: Style::new(Color::from_hex("#b58900").unwrap()),
            syntax_variable: Color::from_hex("#839496").unwrap().into(),
            syntax_operator: Color::from_hex("#859900").unwrap().into(),
            syntax_punctuation: Color::from_hex("#839496").unwrap().into(),

            error: Color::from_hex("#dc322f").unwrap(),
            warning: Color::from_hex("#cb4b16").unwrap(),
            info: Color::from_hex("#268bd2").unwrap(),
            hint: Color::from_hex("#2aa198").unwrap(),
        }
    }
}

impl From<Color> for Style {
    fn from(color: Color) -> Self {
        Style::new(color)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::gruvbox_dark()
    }
}
