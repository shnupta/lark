use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::editor::{Editor, Mode};

pub fn handle_event(editor: &mut Editor, event: Event) {
    match event {
        Event::Key(key) => {
            // Clear any message on keypress
            editor.clear_message();
            handle_key(editor, key);
        }
        Event::Resize(_, _) => {
            // Resize is handled by the renderer
        }
        _ => {}
    }
}

fn handle_key(editor: &mut Editor, key: KeyEvent) {
    // Ctrl-G toggles file browser from any mode
    if key.code == KeyCode::Char('g') && key.modifiers.contains(KeyModifiers::CONTROL) {
        editor.toggle_file_browser();
        return;
    }

    match editor.mode {
        Mode::Normal => handle_normal_mode(editor, key),
        Mode::Insert => handle_insert_mode(editor, key),
        Mode::Command => handle_command_mode(editor, key),
        Mode::FileBrowser => handle_file_browser_mode(editor, key),
    }
}

fn handle_file_browser_mode(editor: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => editor.mode = Mode::Normal,
        KeyCode::Char('j') | KeyCode::Down => editor.file_browser.move_down(),
        KeyCode::Char('k') | KeyCode::Up => editor.file_browser.move_up(),
        KeyCode::Enter => editor.open_selected_file(),
        _ => {}
    }
}

fn handle_normal_mode(editor: &mut Editor, key: KeyEvent) {
    match key.code {
        // Basic movement
        KeyCode::Char('h') | KeyCode::Left => editor.move_left(),
        KeyCode::Char('j') | KeyCode::Down => editor.move_down(),
        KeyCode::Char('k') | KeyCode::Up => editor.move_up(),
        KeyCode::Char('l') | KeyCode::Right => editor.move_right(),

        // Line motions
        KeyCode::Char('0') => editor.move_to_line_start(),
        KeyCode::Char('$') => editor.move_to_line_end(),
        KeyCode::Char('g') => editor.move_to_first_line(), // TODO: proper gg
        KeyCode::Char('G') => editor.move_to_last_line(),

        // Word motions
        KeyCode::Char('w') => editor.move_word_forward(),
        KeyCode::Char('b') => editor.move_word_backward(),
        KeyCode::Char('e') => editor.move_word_end(),

        // Insert mode entry
        KeyCode::Char('i') => editor.enter_insert_mode(),
        KeyCode::Char('a') => editor.append(),
        KeyCode::Char('A') => editor.append_end_of_line(),
        KeyCode::Char('o') => editor.open_line_below(),
        KeyCode::Char('O') => editor.open_line_above(),

        // Command mode
        KeyCode::Char(':') => editor.enter_command_mode(),

        // Quick quit with Ctrl-C
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            editor.quit();
        }

        _ => {}
    }
}

fn handle_insert_mode(editor: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => editor.enter_normal_mode(),

        KeyCode::Char(c) => {
            editor.insert_char(c);
        }

        KeyCode::Backspace => {
            editor.delete_char_backward();
        }

        KeyCode::Enter => {
            editor.insert_newline();
        }

        KeyCode::Left => editor.move_left(),
        KeyCode::Right => editor.move_right(),
        KeyCode::Up => editor.move_up(),
        KeyCode::Down => editor.move_down(),

        _ => {}
    }
}

fn handle_command_mode(editor: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            editor.command_buffer.clear();
            editor.enter_normal_mode();
        }

        KeyCode::Enter => {
            editor.execute_command();
        }

        KeyCode::Backspace => {
            editor.command_buffer.pop();
            if editor.command_buffer.is_empty() {
                editor.enter_normal_mode();
            }
        }

        KeyCode::Char(c) => {
            editor.command_buffer.push(c);
        }

        _ => {}
    }
}
