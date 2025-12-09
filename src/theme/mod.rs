mod colors;
mod theme;

pub use colors::Color;
pub use theme::Theme;

/// Built-in themes
pub fn default_theme() -> Theme {
    Theme::gruvbox_dark()
}

pub fn list_builtin_themes() -> Vec<&'static str> {
    vec![
        "gruvbox-dark",
        "gruvbox-light",
        "nord",
        "dracula",
        "solarized-dark",
    ]
}

pub fn get_builtin_theme(name: &str) -> Option<Theme> {
    match name {
        "gruvbox-dark" => Some(Theme::gruvbox_dark()),
        "gruvbox-light" => Some(Theme::gruvbox_light()),
        "nord" => Some(Theme::nord()),
        "dracula" => Some(Theme::dracula()),
        "solarized-dark" => Some(Theme::solarized_dark()),
        _ => None,
    }
}
