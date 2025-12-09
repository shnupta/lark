// Lark Editor Configuration
// Place this file at: ~/.config/lark/init.rhai

// =============================================================================
// Theme
// =============================================================================
// Available themes: gruvbox-dark, gruvbox-light, nord, dracula, solarized-dark
set_theme("gruvbox-dark");

// =============================================================================
// Editor Settings
// =============================================================================

// Tab width (1-16)
set_tab_width(4);

// Use spaces instead of tabs
set_insert_spaces(true);

// Auto-indent new lines
set_auto_indent(true);

// Line numbers
set_show_line_numbers(true);
set_relative_line_numbers(true);

// =============================================================================
// File Browser
// =============================================================================

// Show hidden files (dotfiles)
set_show_hidden_files(false);

// =============================================================================
// Custom Keybinds
// =============================================================================
// Format: bind("key-sequence", "action")
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
// bind("<leader>w", "save");
// bind("<leader>q", "quit");
// bind("<leader>n", "new-tab");

