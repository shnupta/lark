use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;

use super::keymap::{Action, Key, KeyResult, KeySequenceState};
use crate::editor::{Direction, FinderAction, Mode, PaneKind, Workspace};

pub struct InputState {
    pub key_seq: KeySequenceState,
    pub pending_file_path: Option<PathBuf>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            key_seq: KeySequenceState::new(),
            pending_file_path: None,
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn handle_event(workspace: &mut Workspace, event: Event, input_state: &mut InputState) {
    match event {
        Event::Key(key) => {
            // If there's an error displayed, dismiss it on any keypress
            if workspace.error.is_some() {
                workspace.clear_error();
                return; // Don't process the key, just dismiss the error
            }
            workspace.clear_message();
            handle_key(workspace, key, input_state);
        }
        Event::Resize(_, _) => {}
        _ => {}
    }
}

fn handle_key(workspace: &mut Workspace, key: KeyEvent, input_state: &mut InputState) {
    // Handle pane selection mode
    if workspace.selecting_pane {
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_lowercase() {
                if let Some(path) = input_state.pending_file_path.take() {
                    workspace.open_file_in_pane(path, c);
                }
                workspace.selecting_pane = false;
                return;
            }
        }
        if key.code == KeyCode::Esc {
            workspace.selecting_pane = false;
            input_state.pending_file_path = None;
            return;
        }
        return;
    }

    // Command mode takes priority - check this first
    if workspace.mode() == Mode::Command {
        handle_command_mode(workspace, key);
        return;
    }

    let pane = workspace.focused_pane();
    let kind = pane.kind;

    // File browser - handle navigation and file browser specific keys
    if kind == PaneKind::FileBrowser {
        // Window navigation works from file browser
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('w') {
            // Start Ctrl-W sequence
            let k = Key::new(key.code, key.modifiers);
            match input_state.key_seq.process_key(k, "normal") {
                KeyResult::Pending => {
                    workspace.pending_keys = input_state.key_seq.pending_display();
                }
                _ => {}
            }
            return;
        }

        // Check if we're in the middle of Ctrl-W sequence
        if !input_state.key_seq.pending_display().is_empty() {
            let k = Key::new(key.code, key.modifiers);
            match input_state.key_seq.process_key(k, "normal") {
                KeyResult::Action(action, _) => {
                    workspace.pending_keys.clear();
                    execute_action(workspace, action, 1, input_state);
                    return;
                }
                KeyResult::Pending => {
                    workspace.pending_keys = input_state.key_seq.pending_display();
                    return;
                }
                _ => {
                    workspace.pending_keys.clear();
                }
            }
        }

        // Ctrl-G to toggle off
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('g') {
            workspace.toggle_file_browser();
            return;
        }

        handle_file_browser(workspace, key, input_state);
        return;
    }

    // Insert mode - handle text input directly
    if workspace.focused_pane().mode == Mode::Insert {
        if handle_insert_mode(workspace, key) {
            return;
        }
    }

    // Use key sequence system
    let mode_str = match workspace.focused_pane().mode {
        Mode::Normal => "normal",
        Mode::Insert => "insert",
        _ => "normal",
    };

    let k = Key::new(key.code, key.modifiers);
    match input_state.key_seq.process_key(k, mode_str) {
        KeyResult::Action(action, count) => {
            workspace.pending_keys.clear();
            execute_action(workspace, action, count, input_state);
        }
        KeyResult::Pending => {
            workspace.pending_keys = input_state.key_seq.pending_display();
        }
        KeyResult::Unhandled | KeyResult::Cancelled => {
            workspace.pending_keys.clear();
        }
    }
}

fn handle_file_browser(workspace: &mut Workspace, key: KeyEvent, input_state: &mut InputState) {
    // Ctrl+T to open in new tab
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('t') {
        if let Some(path) = workspace.file_browser_mut().select() {
            workspace.open_file_in_new_tab(path);
        }
        return;
    }

    match key.code {
        KeyCode::Esc => {
            // Escape just clears any message, doesn't close file browser
            // Use Ctrl+G to toggle file browser
            workspace.clear_message();
        }
        KeyCode::Char('j') | KeyCode::Down => workspace.file_browser_mut().move_down(),
        KeyCode::Char('k') | KeyCode::Up => workspace.file_browser_mut().move_up(),
        KeyCode::Char(':') => {
            // Enter command mode even from file browser
            workspace.focused_pane_mut().mode = Mode::Command;
            workspace.command_buffer.clear();
        }
        KeyCode::Enter => {
            if let Some(path) = workspace.try_open_file_from_browser() {
                let editor_panes = workspace.get_editor_panes_with_labels();
                if editor_panes.len() > 1 {
                    workspace.selecting_pane = true;
                    input_state.pending_file_path = Some(path);
                    let labels: String = editor_panes.iter().map(|(c, _)| *c).collect();
                    workspace.set_message(format!("Select pane: {}", labels));
                }
            }
        }
        _ => {}
    }
}

fn handle_insert_mode(workspace: &mut Workspace, key: KeyEvent) -> bool {
    let pane = workspace.focused_pane_mut();

    match key.code {
        KeyCode::Esc => {
            pane.mode = Mode::Normal;
            let line_len = pane.buffer.line_len(pane.cursor.line);
            if pane.cursor.col > 0 && pane.cursor.col >= line_len {
                pane.cursor.col = line_len.saturating_sub(1);
            }
            true
        }
        KeyCode::Char(c) => {
            pane.buffer
                .insert_char(pane.cursor.line, pane.cursor.col, c);
            pane.cursor.col += 1;
            true
        }
        KeyCode::Backspace => {
            if pane.cursor.col > 0 {
                pane.buffer
                    .delete_char_backward(pane.cursor.line, pane.cursor.col);
                pane.cursor.col -= 1;
            } else if pane.cursor.line > 0 {
                let prev_line_len = pane.buffer.line_len(pane.cursor.line - 1);
                pane.buffer
                    .delete_char_backward(pane.cursor.line, pane.cursor.col);
                pane.cursor.line -= 1;
                pane.cursor.col = prev_line_len;
            }
            true
        }
        KeyCode::Enter => {
            pane.buffer
                .insert_newline(pane.cursor.line, pane.cursor.col);
            pane.cursor.line += 1;
            pane.cursor.col = 0;
            true
        }
        _ => false,
    }
}

fn handle_command_mode(workspace: &mut Workspace, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            workspace.command_buffer.clear();
            workspace.focused_pane_mut().mode = Mode::Normal;
        }
        KeyCode::Enter => {
            execute_command(workspace);
        }
        KeyCode::Backspace => {
            workspace.command_buffer.pop();
            if workspace.command_buffer.is_empty() {
                workspace.focused_pane_mut().mode = Mode::Normal;
            }
        }
        KeyCode::Char(c) => {
            workspace.command_buffer.push(c);
        }
        _ => {}
    }
}

fn execute_action(
    workspace: &mut Workspace,
    action: Action,
    count: usize,
    _input_state: &mut InputState,
) {
    for _ in 0..count {
        match action.clone() {
            // Movement
            Action::MoveLeft => {
                workspace.focused_pane_mut().cursor.move_left();
            }
            Action::MoveRight => {
                let pane = workspace.focused_pane_mut();
                let line_len = pane.buffer.line_len(pane.cursor.line);
                pane.cursor.move_right(line_len);
            }
            Action::MoveUp => {
                let pane = workspace.focused_pane_mut();
                pane.cursor.move_up();
                let line_len = pane.buffer.line_len(pane.cursor.line);
                if pane.cursor.col > line_len {
                    pane.cursor.col = line_len;
                }
            }
            Action::MoveDown => {
                let pane = workspace.focused_pane_mut();
                let line_count = pane.buffer.line_count();
                pane.cursor.move_down(line_count);
                let line_len = pane.buffer.line_len(pane.cursor.line);
                if pane.cursor.col > line_len {
                    pane.cursor.col = line_len;
                }
            }
            Action::MoveToLineStart => {
                workspace.focused_pane_mut().cursor.col = 0;
            }
            Action::MoveToLineEnd => {
                let pane = workspace.focused_pane_mut();
                let line_len = pane.buffer.line_len(pane.cursor.line);
                pane.cursor.col = line_len.saturating_sub(1);
            }
            Action::MoveToFirstLine => {
                let pane = workspace.focused_pane_mut();
                pane.cursor.line = 0;
                pane.cursor.col = 0;
            }
            Action::MoveToLastLine => {
                let pane = workspace.focused_pane_mut();
                pane.cursor.line = pane.buffer.line_count().saturating_sub(1);
            }
            Action::MoveWordForward => move_word_forward(workspace.focused_pane_mut()),
            Action::MoveWordBackward => move_word_backward(workspace.focused_pane_mut()),
            Action::MoveWordEnd => move_word_end(workspace.focused_pane_mut()),
            Action::PageDown => {
                let pane = workspace.focused_pane_mut();
                let line_count = pane.buffer.line_count();
                for _ in 0..20 {
                    pane.cursor.move_down(line_count);
                }
                let line_len = pane.buffer.line_len(pane.cursor.line);
                if pane.cursor.col > line_len {
                    pane.cursor.col = line_len;
                }
            }
            Action::PageUp => {
                let pane = workspace.focused_pane_mut();
                for _ in 0..20 {
                    pane.cursor.move_up();
                }
                let line_len = pane.buffer.line_len(pane.cursor.line);
                if pane.cursor.col > line_len {
                    pane.cursor.col = line_len;
                }
            }

            // Mode changes
            Action::EnterInsertMode => {
                workspace.focused_pane_mut().mode = Mode::Insert;
            }
            Action::EnterInsertModeAppend => {
                let pane = workspace.focused_pane_mut();
                let line_len = pane.buffer.line_len(pane.cursor.line);
                if pane.cursor.col < line_len {
                    pane.cursor.col += 1;
                }
                pane.mode = Mode::Insert;
            }
            Action::EnterInsertModeAppendLine => {
                let pane = workspace.focused_pane_mut();
                pane.cursor.col = pane.buffer.line_len(pane.cursor.line);
                pane.mode = Mode::Insert;
            }
            Action::EnterInsertModeOpenBelow => {
                let pane = workspace.focused_pane_mut();
                let line_len = pane.buffer.line_len(pane.cursor.line);
                pane.cursor.col = line_len;
                pane.buffer
                    .insert_newline(pane.cursor.line, pane.cursor.col);
                pane.cursor.line += 1;
                pane.cursor.col = 0;
                pane.mode = Mode::Insert;
            }
            Action::EnterInsertModeOpenAbove => {
                let pane = workspace.focused_pane_mut();
                pane.cursor.col = 0;
                pane.buffer.insert_newline(pane.cursor.line, 0);
                pane.mode = Mode::Insert;
            }
            Action::EnterNormalMode => {
                let pane = workspace.focused_pane_mut();
                pane.mode = Mode::Normal;
                let line_len = pane.buffer.line_len(pane.cursor.line);
                if pane.cursor.col > 0 && pane.cursor.col >= line_len {
                    pane.cursor.col = line_len.saturating_sub(1);
                }
            }
            Action::EnterCommandMode => {
                workspace.focused_pane_mut().mode = Mode::Command;
                workspace.command_buffer.clear();
            }

            // Window management
            Action::SplitVertical => workspace.split_vertical(),
            Action::SplitHorizontal => workspace.split_horizontal(),
            Action::FocusNext => workspace.focus_next(),
            Action::FocusLeft => workspace.focus_direction(Direction::Left),
            Action::FocusRight => workspace.focus_direction(Direction::Right),
            Action::FocusUp => workspace.focus_direction(Direction::Up),
            Action::FocusDown => workspace.focus_direction(Direction::Down),

            // File browser
            Action::ToggleFileBrowser => workspace.toggle_file_browser(),
            Action::FocusFileBrowser => workspace.focus_file_browser(),

            // Finder actions
            Action::FindFile => {
                workspace.pending_finder = Some(FinderAction::FindFile);
            }
            Action::Grep => {
                // For now, grep the word under cursor (or prompt for pattern)
                workspace.pending_finder = Some(FinderAction::Grep(String::new()));
            }

            // Pane selection
            Action::SelectPane(c) => {
                workspace.focus_pane_by_label(c);
            }

            // Tabs
            Action::NewTab => {
                workspace.new_tab();
            }
            Action::NextTab => {
                workspace.next_tab();
            }
            Action::PrevTab => {
                workspace.prev_tab();
            }
            Action::CloseTab => {
                workspace.close_tab();
            }

            // Other
            Action::Quit => workspace.quit(),
        }
    }
}

fn execute_command(workspace: &mut Workspace) {
    let cmd = workspace.command_buffer.trim().to_string();
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let command = parts.first().map(|s| *s).unwrap_or("");
    let args = parts.get(1).map(|s| *s);

    match command {
        "q" | "quit" => {
            // Close current pane, or quit if last pane
            if !workspace.close_focused_pane() {
                workspace.quit();
            }
        }
        "qa" | "quitall" => workspace.quit(),
        "w" | "write" => match workspace.focused_pane_mut().buffer.save() {
            Ok(_) => workspace.set_message("Written"),
            Err(e) => workspace.set_message(format!("Error: {}", e)),
        },
        "wq" => match workspace.focused_pane_mut().buffer.save() {
            Ok(_) => {
                if !workspace.close_focused_pane() {
                    workspace.quit();
                }
            }
            Err(e) => workspace.set_message(format!("Error: {}", e)),
        },
        "vs" | "vsplit" => workspace.split_vertical(),
        "sp" | "split" => workspace.split_horizontal(),
        "close" => {
            workspace.close_focused_pane();
        }
        "theme" => {
            if let Some(name) = args {
                let available = crate::theme::list_builtin_themes();
                if available.contains(&name) {
                    workspace.set_theme(name);
                    workspace.set_message(format!("Theme: {}", name));
                } else {
                    workspace.set_message(format!(
                        "Unknown theme: {}. Available: {}",
                        name,
                        available.join(", ")
                    ));
                }
            } else {
                workspace.set_message(format!("Current theme: {}", workspace.theme_name));
            }
        }
        "themes" => {
            let themes = crate::theme::list_builtin_themes().join(", ");
            workspace.set_message(format!("Available themes: {}", themes));
        }
        "source" => {
            // Reload config file
            let mut script_engine = crate::scripting::ScriptEngine::new();
            match script_engine.load_default() {
                Ok(_) => {
                    let settings = script_engine.settings();
                    workspace.theme_name = settings.theme.clone();
                    workspace.set_message("Config reloaded");
                }
                Err(e) => {
                    workspace.set_message(format!("Config error: {}", e));
                }
            }
        }
        "TSList" => {
            // List installed and available grammars
            let registry = crate::syntax::LanguageRegistry::new();
            let installed = registry.installed();
            let not_installed = registry.not_installed();
            let outdated = registry.outdated_grammars();

            let installed_names: Vec<_> = installed.iter().map(|l| l.name()).collect();
            let not_installed_names: Vec<_> = not_installed.iter().map(|l| l.name()).collect();

            let mut msg = String::new();
            if installed_names.is_empty() {
                msg.push_str(&format!(
                    "No grammars installed. Available: {}",
                    not_installed_names.join(", ")
                ));
            } else {
                msg.push_str(&format!(
                    "Installed: {} | Available: {}",
                    installed_names.join(", "),
                    not_installed_names.join(", ")
                ));
            }

            if !outdated.is_empty() {
                msg.push_str(&format!(" | Outdated: {}", outdated.join(", ")));
            }

            workspace.set_message(msg);
        }
        "TSStatus" => {
            // Show ABI version and status
            let registry = crate::syntax::LanguageRegistry::new();
            let outdated = registry.outdated_grammars();

            let mut lines = vec![format!(
                "Tree-sitter ABI version: {}",
                crate::syntax::TREE_SITTER_ABI_VERSION
            )];

            if outdated.is_empty() {
                lines.push("All grammars are compatible".to_string());
            } else {
                lines.push(format!("Outdated grammars: {}", outdated.join(", ")));
                lines.push("Use :TSUpdate to reinstall".to_string());
            }

            workspace.set_message(lines.join("\n"));
        }
        "TSUpdate" => {
            // Reinstall all outdated grammars
            let mut installer = crate::syntax::GrammarInstaller::new();
            let outdated = installer.outdated_grammars();

            if outdated.is_empty() {
                workspace.set_message("All grammars are up to date");
            } else {
                workspace.set_message(format!("Updating {} grammars...", outdated.len()));
                let results = installer.reinstall_outdated();

                let success_count = results
                    .iter()
                    .filter(|(_, r)| matches!(r, crate::syntax::InstallResult::Reinstalled))
                    .count();
                let fail_count = results
                    .iter()
                    .filter(|(_, r)| matches!(r, crate::syntax::InstallResult::Error(_)))
                    .count();

                if fail_count == 0 {
                    workspace
                        .set_message(format!("Updated {} grammars successfully", success_count));
                } else {
                    workspace
                        .set_error(format!("Updated {}, failed {}", success_count, fail_count));
                }
            }
        }
        _ if cmd.starts_with("TSInstall ") => {
            // Install a grammar
            let lang_name = cmd.strip_prefix("TSInstall ").unwrap().trim();

            // Find the language
            let lang = match lang_name.to_lowercase().as_str() {
                "rust" => Some(crate::syntax::Language::Rust),
                "python" => Some(crate::syntax::Language::Python),
                "javascript" | "js" => Some(crate::syntax::Language::JavaScript),
                "typescript" | "ts" => Some(crate::syntax::Language::TypeScript),
                "tsx" => Some(crate::syntax::Language::Tsx),
                "go" => Some(crate::syntax::Language::Go),
                "c" => Some(crate::syntax::Language::C),
                "cpp" | "c++" => Some(crate::syntax::Language::Cpp),
                "json" => Some(crate::syntax::Language::Json),
                "toml" => Some(crate::syntax::Language::Toml),
                "markdown" | "md" => Some(crate::syntax::Language::Markdown),
                "bash" | "sh" => Some(crate::syntax::Language::Bash),
                "lua" => Some(crate::syntax::Language::Lua),
                "ruby" => Some(crate::syntax::Language::Ruby),
                "html" => Some(crate::syntax::Language::Html),
                "css" => Some(crate::syntax::Language::Css),
                "yaml" | "yml" => Some(crate::syntax::Language::Yaml),
                _ => None,
            };

            match lang {
                Some(lang) => {
                    workspace.set_message(format!("Installing {} grammar...", lang.name()));
                    // Note: This blocks the UI - ideally should be async
                    let mut installer = crate::syntax::GrammarInstaller::new();
                    match installer.install(lang) {
                        crate::syntax::InstallResult::Success => {
                            workspace.set_message(format!(
                                "{} grammar installed successfully!",
                                lang.name()
                            ));
                        }
                        crate::syntax::InstallResult::AlreadyInstalled => {
                            workspace.set_message(format!(
                                "{} grammar is already installed",
                                lang.name()
                            ));
                        }
                        crate::syntax::InstallResult::Reinstalled => {
                            workspace.set_message(format!(
                                "{} grammar reinstalled (ABI updated)",
                                lang.name()
                            ));
                        }
                        crate::syntax::InstallResult::Error(e) => {
                            workspace.set_error(format!(
                                "Failed to install {} grammar:\n{}",
                                lang.name(),
                                e
                            ));
                        }
                    }
                }
                None => {
                    let available: Vec<_> = crate::syntax::Language::all_installable()
                        .iter()
                        .map(|l| l.name())
                        .collect();
                    workspace.set_message(format!(
                        "Unknown language: {}. Available: {}",
                        lang_name,
                        available.join(", ")
                    ));
                }
            }
        }
        _ if cmd.starts_with("TSUninstall ") => {
            // Uninstall a grammar
            let lang_name = cmd.strip_prefix("TSUninstall ").unwrap().trim();

            let lang = match lang_name.to_lowercase().as_str() {
                "rust" => Some(crate::syntax::Language::Rust),
                "python" => Some(crate::syntax::Language::Python),
                "javascript" | "js" => Some(crate::syntax::Language::JavaScript),
                "typescript" | "ts" => Some(crate::syntax::Language::TypeScript),
                "go" => Some(crate::syntax::Language::Go),
                "c" => Some(crate::syntax::Language::C),
                "cpp" | "c++" => Some(crate::syntax::Language::Cpp),
                "json" => Some(crate::syntax::Language::Json),
                "toml" => Some(crate::syntax::Language::Toml),
                "markdown" | "md" => Some(crate::syntax::Language::Markdown),
                _ => None,
            };

            match lang {
                Some(lang) => {
                    let mut installer = crate::syntax::GrammarInstaller::new();
                    match installer.uninstall(lang) {
                        Ok(_) => {
                            workspace.set_message(format!("{} grammar uninstalled", lang.name()));
                        }
                        Err(e) => {
                            workspace.set_error(format!("Failed to uninstall: {}", e));
                        }
                    }
                }
                None => {
                    workspace.set_message(format!("Unknown language: {}", lang_name));
                }
            }
        }
        "log" => {
            // Show the editor log
            let log = workspace.get_log();
            if log.is_empty() {
                workspace.set_message("Log is empty");
            } else {
                // Show log in message - truncate if too long
                let lines: Vec<&str> = log.lines().collect();
                let display = if lines.len() > 5 {
                    format!(
                        "... ({} more)\n{}",
                        lines.len() - 5,
                        lines[lines.len() - 5..].join("\n")
                    )
                } else {
                    log
                };
                workspace.set_message(display);
            }
        }
        "syntax" => {
            // Show syntax highlighting status for the focused editor pane
            let pane = workspace.focused_pane();
            let file_info = pane
                .buffer
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "[No file]".to_string());
            let pane_kind = match pane.kind {
                crate::editor::PaneKind::Editor => "Editor",
                crate::editor::PaneKind::FileBrowser => "FileBrowser",
            };
            let status = pane.highlighter.status();
            workspace.set_message(format!("{} | {} | {}", pane_kind, file_info, status));
        }
        "verbose" => {
            // Toggle verbose mode
            workspace.verbose = !workspace.verbose;
            workspace.set_message(format!(
                "Verbose mode: {}",
                if workspace.verbose { "on" } else { "off" }
            ));
        }
        _ if cmd.starts_with("e ") || cmd.starts_with("edit ") => {
            // Open a file
            let path_str = if cmd.starts_with("e ") {
                cmd.strip_prefix("e ").unwrap().trim()
            } else {
                cmd.strip_prefix("edit ").unwrap().trim()
            };

            let path = std::path::PathBuf::from(path_str);
            if path.exists() {
                workspace.open_file_in_focused_pane(path);
                workspace.set_message(format!("Opened: {}", path_str));
            } else {
                workspace.set_message(format!("File not found: {}", path_str));
            }
        }
        "" => {}
        _ => {
            workspace.set_message(format!("Unknown command: {}", cmd));
        }
    }
    workspace.command_buffer.clear();
    workspace.focused_pane_mut().mode = Mode::Normal;
}

// Word motion helpers
fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn move_word_forward(pane: &mut crate::editor::Pane) {
    let line_count = pane.buffer.line_count();

    while let Some(c) = pane.buffer.char_at(pane.cursor.line, pane.cursor.col) {
        if !is_word_char(c) {
            break;
        }
        pane.cursor.col += 1;
    }

    loop {
        match pane.buffer.char_at(pane.cursor.line, pane.cursor.col) {
            Some(c) if is_word_char(c) => break,
            Some(_) => pane.cursor.col += 1,
            None => {
                if pane.cursor.line + 1 < line_count {
                    pane.cursor.line += 1;
                    pane.cursor.col = 0;
                } else {
                    break;
                }
            }
        }
    }
}

fn move_word_backward(pane: &mut crate::editor::Pane) {
    if pane.cursor.col > 0 {
        pane.cursor.col -= 1;
    } else if pane.cursor.line > 0 {
        pane.cursor.line -= 1;
        pane.cursor.col = pane.buffer.line_len(pane.cursor.line).saturating_sub(1);
    }

    loop {
        match pane.buffer.char_at(pane.cursor.line, pane.cursor.col) {
            Some(c) if is_word_char(c) => break,
            Some(_) if pane.cursor.col > 0 => pane.cursor.col -= 1,
            _ if pane.cursor.line > 0 => {
                pane.cursor.line -= 1;
                pane.cursor.col = pane.buffer.line_len(pane.cursor.line).saturating_sub(1);
            }
            _ => return,
        }
    }

    while pane.cursor.col > 0 {
        if let Some(c) = pane.buffer.char_at(pane.cursor.line, pane.cursor.col - 1) {
            if is_word_char(c) {
                pane.cursor.col -= 1;
            } else {
                break;
            }
        } else {
            break;
        }
    }
}

fn move_word_end(pane: &mut crate::editor::Pane) {
    let line_count = pane.buffer.line_count();
    pane.cursor.col += 1;

    loop {
        match pane.buffer.char_at(pane.cursor.line, pane.cursor.col) {
            Some(c) if is_word_char(c) => break,
            Some(_) => pane.cursor.col += 1,
            None => {
                if pane.cursor.line + 1 < line_count {
                    pane.cursor.line += 1;
                    pane.cursor.col = 0;
                } else {
                    let line_len = pane.buffer.line_len(pane.cursor.line);
                    if pane.cursor.col > line_len {
                        pane.cursor.col = line_len;
                    }
                    return;
                }
            }
        }
    }

    while let Some(c) = pane.buffer.char_at(pane.cursor.line, pane.cursor.col + 1) {
        if is_word_char(c) {
            pane.cursor.col += 1;
        } else {
            break;
        }
    }
}
