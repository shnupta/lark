use ropey::Rope;
use std::{fs::File, io, path::PathBuf};

pub struct Buffer {
    text: Rope,
    filepath: Option<PathBuf>,
    dirty: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            text: Rope::new(),
            filepath: None,
            dirty: false,
        }
    }

    pub fn from_file(path: PathBuf) -> Self {
        let text = Rope::from_reader(File::open(&path).unwrap()).unwrap();
        Self {
            text,
            filepath: Some(path),
            dirty: false,
        }
    }

    /// Create a buffer from a string (useful for testing)
    #[cfg(test)]
    pub fn from_text(s: &str) -> Self {
        Self {
            text: Rope::from_str(s),
            filepath: None,
            dirty: false,
        }
    }

    pub fn save(&self) -> io::Result<()> {
        if !self.dirty {
            return Ok(());
        }
        if let Some(path) = &self.filepath {
            let mut file = File::create(path)?;
            self.text.write_to(&mut file)?;
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "No file path"))
        }
    }

    pub fn line_count(&self) -> usize {
        self.text.len_lines()
    }

    pub fn line(&self, idx: usize) -> ropey::RopeSlice {
        self.text.line(idx)
    }

    pub fn line_len(&self, idx: usize) -> usize {
        // Length excluding newline character
        let line = self.text.line(idx);
        let len = line.len_chars();
        if len > 0 && line.char(len - 1) == '\n' {
            len - 1
        } else {
            len
        }
    }

    /// Get character at position, returns None if out of bounds
    pub fn char_at(&self, line: usize, col: usize) -> Option<char> {
        if line >= self.line_count() {
            return None;
        }
        let rope_line = self.text.line(line);
        if col >= rope_line.len_chars() {
            return None;
        }
        Some(rope_line.char(col))
    }

    /// Convert (line, col) to a char index in the rope
    fn line_col_to_char(&self, line: usize, col: usize) -> usize {
        self.text.line_to_char(line) + col
    }

    /// Insert a character at the given position
    pub fn insert_char(&mut self, line: usize, col: usize, ch: char) {
        let idx = self.line_col_to_char(line, col);
        self.text.insert_char(idx, ch);
        self.dirty = true;
    }

    /// Delete the character at the given position
    pub fn delete_char(&mut self, line: usize, col: usize) {
        let idx = self.line_col_to_char(line, col);
        if idx < self.text.len_chars() {
            self.text.remove(idx..idx + 1);
            self.dirty = true;
        }
    }

    /// Delete the character before the given position (backspace)
    pub fn delete_char_backward(&mut self, line: usize, col: usize) -> bool {
        if col > 0 {
            self.delete_char(line, col - 1);
            true
        } else if line > 0 {
            // At start of line, join with previous line
            let idx = self.line_col_to_char(line, 0);
            if idx > 0 {
                self.text.remove(idx - 1..idx);
                self.dirty = true;
                return true;
            }
            false
        } else {
            false
        }
    }

    /// Insert a newline at the given position
    pub fn insert_newline(&mut self, line: usize, col: usize) {
        self.insert_char(line, col, '\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn buffer_from_str(s: &str) -> Buffer {
        Buffer {
            text: Rope::from_str(s),
            filepath: None,
            dirty: false,
        }
    }

    #[test]
    fn new_buffer_is_empty() {
        let buf = Buffer::new();
        assert_eq!(buf.line_count(), 1); // empty rope has 1 line
    }

    #[test]
    fn line_count_counts_lines() {
        let buf = buffer_from_str("hello\nworld\ntest\n");
        assert_eq!(buf.line_count(), 4); // 3 lines + trailing newline creates 4th
    }

    #[test]
    fn line_returns_correct_content() {
        let buf = buffer_from_str("first\nsecond\nthird");
        assert_eq!(buf.line(0).to_string(), "first\n");
        assert_eq!(buf.line(1).to_string(), "second\n");
        assert_eq!(buf.line(2).to_string(), "third");
    }

    #[test]
    fn line_len_excludes_newline() {
        let buf = buffer_from_str("hello\nworld");
        assert_eq!(buf.line_len(0), 5); // "hello" without \n
        assert_eq!(buf.line_len(1), 5); // "world" (no trailing \n)
    }

    #[test]
    fn line_len_handles_empty_lines() {
        let buf = buffer_from_str("hello\n\nworld");
        assert_eq!(buf.line_len(0), 5);
        assert_eq!(buf.line_len(1), 0); // empty line
        assert_eq!(buf.line_len(2), 5);
    }
}
