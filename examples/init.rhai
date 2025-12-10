// Lark Editor Configuration
// Place this file at: ~/.config/lark/init.rhai

// =============================================================================
// Theme
// =============================================================================
// Available themes: gruvbox-dark, gruvbox-light, nord, dracula, solarized-dark
lark::config::set_theme("gruvbox-dark");

// List available themes:
// let themes = lark::config::list_themes();
// print(`Available themes: ${themes}`);

// =============================================================================
// Editor Settings
// =============================================================================

// Tab width (1-16)
lark::config::set_tab_width(4);

// Use spaces instead of tabs
lark::config::set_insert_spaces(true);

// Auto-indent new lines
lark::config::set_auto_indent(true);

// Line numbers
lark::config::set_show_line_numbers(true);
lark::config::set_relative_line_numbers(true);

// =============================================================================
// File Browser
// =============================================================================

// Show hidden files (dotfiles)
lark::config::set_show_hidden_files(false);

// =============================================================================
// Custom Keybinds
// =============================================================================
// Format: lark::config::bind("key-sequence", "action")
// 
// Available actions (more coming soon):
//   save, quit, split-vertical, split-horizontal, next-tab, prev-tab,
//   new-tab, close-tab, find-file, grep
//
// Key format examples:
//   "<leader>w"   - Space followed by w
//   "<C-s>"       - Ctrl+S
//   "gd"          - g followed by d
//
// Example keybinds:
// lark::config::bind("<leader>w", "save");
// lark::config::bind("<leader>q", "quit");
// lark::config::bind("<leader>n", "new-tab");
