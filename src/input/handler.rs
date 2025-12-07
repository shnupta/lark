use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};

use crate::editor::{Editor, Mode};

pub fn handle_event(editor: &mut Editor, event: Event) {
    match event {
        Event::Key(key) => handle_key(editor, key),
        Event::Resize(_, _) => {
            // Resize is handled by the renderer
        }
        _ => {}
    }
}

fn handle_key(editor: &mut Editor, key: KeyEvent) {
    match editor.mode {
        Mode::Normal => handle_normal_mode(editor, key),
        Mode::Insert => handle_insert_mode(editor, key),
        Mode::Command => handle_command_mode(editor, key),
    }
}

fn handle_normal_mode(editor: &mut Editor, key: KeyEvent) {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => editor.move_left(),
        KeyCode::Char('j') | KeyCode::Down => editor.move_down(),
        KeyCode::Char('k') | KeyCode::Up => editor.move_up(),
        KeyCode::Char('l') | KeyCode::Right => editor.move_right(),

        KeyCode::Char('i') => editor.enter_insert_mode(),
        KeyCode::Char(':') => editor.enter_command_mode(),

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
