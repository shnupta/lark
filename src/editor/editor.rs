use std::{io, path::PathBuf};

use super::{Buffer, Cursor, FileBrowser, Mode};

pub struct Editor {
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub mode: Mode,
    pub command_buffer: String,
    pub running: bool,
    pub message: Option<String>,
    pub file_browser: FileBrowser,
    pub scroll_offset: usize,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            mode: Mode::default(),
            command_buffer: String::new(),
            running: true,
            message: None,
            file_browser: FileBrowser::new(),
            scroll_offset: 0,
        }
    }

    pub fn open(path: PathBuf) -> Self {
        Self {
            buffer: Buffer::from_file(path),
            cursor: Cursor::new(),
            mode: Mode::default(),
            command_buffer: String::new(),
            running: true,
            message: None,
            file_browser: FileBrowser::new(),
            scroll_offset: 0,
        }
    }

    /// Adjust scroll offset to keep cursor visible within viewport
    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        // Cursor above viewport - scroll up
        if self.cursor.line < self.scroll_offset {
            self.scroll_offset = self.cursor.line;
        }
        // Cursor below viewport - scroll down
        if self.cursor.line >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor.line - viewport_height + 1;
        }
    }

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn clear_message(&mut self) {
        self.message = None;
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

    // Line motions

    /// Move to start of line (0)
    pub fn move_to_line_start(&mut self) {
        self.cursor.col = 0;
    }

    /// Move to end of line ($)
    pub fn move_to_line_end(&mut self) {
        let line_len = self.buffer.line_len(self.cursor.line);
        self.cursor.col = line_len.saturating_sub(1).max(0);
    }

    /// Move to first line (gg)
    pub fn move_to_first_line(&mut self) {
        self.cursor.line = 0;
        self.clamp_cursor_col();
    }

    /// Move to last line (G)
    pub fn move_to_last_line(&mut self) {
        self.cursor.line = self.buffer.line_count().saturating_sub(1);
        self.clamp_cursor_col();
    }

    // Word motions

    fn is_word_char(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }

    /// Move to start of next word (w)
    pub fn move_word_forward(&mut self) {
        let line_count = self.buffer.line_count();

        // Skip current word
        while let Some(c) = self.buffer.char_at(self.cursor.line, self.cursor.col) {
            if !Self::is_word_char(c) {
                break;
            }
            self.cursor.col += 1;
        }

        // Skip whitespace/punctuation
        loop {
            match self.buffer.char_at(self.cursor.line, self.cursor.col) {
                Some(c) if Self::is_word_char(c) => break,
                Some(_) => self.cursor.col += 1,
                None => {
                    // End of line, try next line
                    if self.cursor.line + 1 < line_count {
                        self.cursor.line += 1;
                        self.cursor.col = 0;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    /// Move to start of previous word (b)
    pub fn move_word_backward(&mut self) {
        // Move left at least once
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        } else if self.cursor.line > 0 {
            self.cursor.line -= 1;
            self.cursor.col = self.buffer.line_len(self.cursor.line);
            if self.cursor.col > 0 {
                self.cursor.col -= 1;
            }
        }

        // Skip whitespace/punctuation going backward
        loop {
            match self.buffer.char_at(self.cursor.line, self.cursor.col) {
                Some(c) if Self::is_word_char(c) => break,
                Some(_) if self.cursor.col > 0 => self.cursor.col -= 1,
                _ if self.cursor.line > 0 => {
                    self.cursor.line -= 1;
                    self.cursor.col = self.buffer.line_len(self.cursor.line).saturating_sub(1);
                }
                _ => return,
            }
        }

        // Move to start of word
        while self.cursor.col > 0 {
            if let Some(c) = self.buffer.char_at(self.cursor.line, self.cursor.col - 1) {
                if Self::is_word_char(c) {
                    self.cursor.col -= 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Move to end of word (e)
    pub fn move_word_end(&mut self) {
        let line_count = self.buffer.line_count();

        // Move right at least once
        self.cursor.col += 1;

        // Skip whitespace
        loop {
            match self.buffer.char_at(self.cursor.line, self.cursor.col) {
                Some(c) if Self::is_word_char(c) => break,
                Some(_) => self.cursor.col += 1,
                None => {
                    if self.cursor.line + 1 < line_count {
                        self.cursor.line += 1;
                        self.cursor.col = 0;
                    } else {
                        self.clamp_cursor_col();
                        return;
                    }
                }
            }
        }

        // Move to end of word
        while let Some(c) = self.buffer.char_at(self.cursor.line, self.cursor.col + 1) {
            if Self::is_word_char(c) {
                self.cursor.col += 1;
            } else {
                break;
            }
        }
    }

    pub fn enter_insert_mode(&mut self) {
        self.mode = Mode::Insert;
    }

    /// Append after cursor (a)
    pub fn append(&mut self) {
        let line_len = self.buffer.line_len(self.cursor.line);
        if self.cursor.col < line_len {
            self.cursor.col += 1;
        }
        self.enter_insert_mode();
    }

    /// Append at end of line (A)
    pub fn append_end_of_line(&mut self) {
        self.cursor.col = self.buffer.line_len(self.cursor.line);
        self.enter_insert_mode();
    }

    /// Open line below (o)
    pub fn open_line_below(&mut self) {
        let line_len = self.buffer.line_len(self.cursor.line);
        self.cursor.col = line_len;
        self.buffer
            .insert_newline(self.cursor.line, self.cursor.col);
        self.cursor.line += 1;
        self.cursor.col = 0;
        self.enter_insert_mode();
    }

    /// Open line above (O)
    pub fn open_line_above(&mut self) {
        self.cursor.col = 0;
        self.buffer.insert_newline(self.cursor.line, 0);
        self.enter_insert_mode();
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

    pub fn toggle_file_browser(&mut self) {
        if self.mode == Mode::FileBrowser {
            self.mode = Mode::Normal;
        } else {
            self.file_browser.refresh();
            self.mode = Mode::FileBrowser;
        }
    }

    pub fn open_selected_file(&mut self) {
        if let Some(path) = self.file_browser.select() {
            self.buffer = Buffer::from_file(path);
            self.cursor = Cursor::new();
            self.mode = Mode::Normal;
        }
    }

    pub fn execute_command(&mut self) {
        let cmd = self.command_buffer.trim().to_string();
        match cmd.as_str() {
            "q" | "quit" => self.quit(),
            "w" | "write" => match self.save() {
                Ok(_) => self.set_message("Written"),
                Err(e) => self.set_message(format!("Error: {}", e)),
            },
            "wq" => match self.save() {
                Ok(_) => self.quit(),
                Err(e) => self.set_message(format!("Error: {}", e)),
            },
            "" => {}
            _ => {
                self.set_message(format!("Unknown command: {}", cmd));
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
            message: None,
            file_browser: FileBrowser::new(),
            scroll_offset: 0,
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

    #[test]
    fn adjust_scroll_scrolls_down_when_cursor_below_viewport() {
        let mut editor = editor_with_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        editor.cursor.line = 8; // Line 9 (0-indexed)
        editor.adjust_scroll(5); // Viewport of 5 lines
        assert_eq!(editor.scroll_offset, 4); // Should scroll to show line 8
    }

    #[test]
    fn adjust_scroll_scrolls_up_when_cursor_above_viewport() {
        let mut editor = editor_with_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        editor.scroll_offset = 5;
        editor.cursor.line = 2;
        editor.adjust_scroll(5);
        assert_eq!(editor.scroll_offset, 2); // Should scroll up to show line 2
    }

    #[test]
    fn adjust_scroll_does_nothing_when_cursor_visible() {
        let mut editor = editor_with_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
        editor.scroll_offset = 2;
        editor.cursor.line = 4; // Visible in viewport (lines 2-6)
        editor.adjust_scroll(5);
        assert_eq!(editor.scroll_offset, 2); // No change
    }
}
