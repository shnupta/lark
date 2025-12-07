use std::{io, path::PathBuf};

use super::{Buffer, Cursor, Mode};

pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub mode: Mode,
    pub command_buffer: String,
    pub running: bool,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            mode: Mode::default(),
            command_buffer: String::new(),
            running: true,
        }
    }

    pub fn open(path: PathBuf) -> Self {
        Self {
            buffer: Buffer::from_file(path),
            cursor: Cursor::new(),
            mode: Mode::default(),
            command_buffer: String::new(),
            running: true,
        }
    }

    pub fn move_left(&mut self) {
        self.cursor.move_left();
    }

    pub fn move_right(&mut self) {
        let line_len = self.buffer.line_len(self.cursor.line);
        self.cursor.move_right(line_len);
    }

    pub fn move_up(&mut self) {
        self.cursor.move_up();
        self.clamp_cursor_col();
    }

    pub fn move_down(&mut self) {
        let line_count = self.buffer.line_count();
        self.cursor.move_down(line_count);
        self.clamp_cursor_col();
    }

    fn clamp_cursor_col(&mut self) {
        let line_len = self.buffer.line_len(self.cursor.line);
        if self.cursor.col > line_len {
            self.cursor.col = line_len;
        }
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_normal_mode(&mut self) {
        self.mode = Mode::Normal;
        self.command_buffer.clear();
        self.clamp_cursor_col();
    }

    pub fn enter_command_mode(&mut self) {
        self.mode = Mode::Command;
        self.command_buffer.clear();
    }

    pub fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim();
        match cmd {
            "q" | "quit" => self.quit(),
            "w" | "write" => {
                if let Err(e) = self.save() {
                    // TODO: show error in status line
                    eprintln!("Error saving: {}", e);
                }
            }
            "wq" => {
                let _ = self.save();
                self.quit();
            }
            _ => {
                // Unknown command - TODO: show error
            }
        }
        self.command_buffer.clear();
        self.mode = Mode::Normal;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn save(&mut self) -> io::Result<()> {
        self.buffer.save()
    }

    // Text editing
    pub fn insert_char(&mut self, ch: char) {
        self.buffer
            .insert_char(self.cursor.line, self.cursor.col, ch);
        self.cursor.col += 1;
    }

    pub fn delete_char_backward(&mut self) {
        if self.cursor.col > 0 {
            self.buffer
                .delete_char_backward(self.cursor.line, self.cursor.col);
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            // Join with previous line
            let prev_line_len = self.buffer.line_len(self.cursor.line - 1);
            self.buffer
                .delete_char_backward(self.cursor.line, self.cursor.col);
            self.cursor.line -= 1;
            self.cursor.col = prev_line_len;
        }
    }

    pub fn insert_newline(&mut self) {
        self.buffer
            .insert_newline(self.cursor.line, self.cursor.col);
        self.cursor.line += 1;
        self.cursor.col = 0;
    }
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor_with_text(s: &str) -> Editor {
        Editor {
            buffer: Buffer::from_text(s),
            cursor: Cursor::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
            running: true,
        }
    }

    #[test]
    fn new_editor_starts_in_normal_mode() {
        let editor = Editor::new();
        assert_eq!(editor.mode, Mode::Normal);
        assert!(editor.running);
    }

    #[test]
    fn enter_insert_mode_switches_mode() {
        let mut editor = Editor::new();
        editor.enter_insert_mode();
        assert_eq!(editor.mode, Mode::Insert);
    }

    #[test]
    fn enter_normal_mode_switches_mode() {
        let mut editor = Editor::new();
        editor.enter_insert_mode();
        editor.enter_normal_mode();
        assert_eq!(editor.mode, Mode::Normal);
    }

    #[test]
    fn enter_command_mode_clears_command_buffer() {
        let mut editor = Editor::new();
        editor.command_buffer = "leftover".to_string();
        editor.enter_command_mode();
        assert_eq!(editor.mode, Mode::Command);
        assert!(editor.command_buffer.is_empty());
    }

    #[test]
    fn move_right_respects_line_length() {
        let mut editor = editor_with_text("hello\nworld");
        // Move right 10 times on a 5-char line
        for _ in 0..10 {
            editor.move_right();
        }
        assert_eq!(editor.cursor.col, 5); // clamped to "hello" length
    }

    #[test]
    fn move_down_clamps_col_to_shorter_line() {
        let mut editor = editor_with_text("hello world\nhi");
        editor.cursor.col = 10; // end of first line
        editor.move_down();
        assert_eq!(editor.cursor.line, 1);
        assert_eq!(editor.cursor.col, 2); // clamped to "hi" length
    }

    #[test]
    fn execute_command_q_quits() {
        let mut editor = Editor::new();
        editor.command_buffer = "q".to_string();
        editor.execute_command();
        assert!(!editor.running);
    }

    #[test]
    fn execute_command_quit_quits() {
        let mut editor = Editor::new();
        editor.command_buffer = "quit".to_string();
        editor.execute_command();
        assert!(!editor.running);
    }

    #[test]
    fn execute_command_returns_to_normal_mode() {
        let mut editor = Editor::new();
        editor.mode = Mode::Command;
        editor.command_buffer = "unknown".to_string();
        editor.execute_command();
        assert_eq!(editor.mode, Mode::Normal);
        assert!(editor.command_buffer.is_empty());
    }
}
