#[derive(Debug, Clone, Default)]
pub struct Cursor {
    pub line: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { line: 0, col: 0 }
    }

    pub fn move_left(&mut self) {
        if self.col > 0 {
            self.col -= 1;
        }
    }

    pub fn move_right(&mut self, line_len: usize) {
        self.col += 1;
        self.col = self.col.clamp(0, line_len);
    }

    pub fn move_up(&mut self) {
        if self.line > 0 {
            self.line -= 1;
        }
    }

    pub fn move_down(&mut self, line_count: usize) {
        self.line += 1;
        self.line = self.line.clamp(0, line_count - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cursor_starts_at_origin() {
        let cursor = Cursor::new();
        assert_eq!(cursor.line, 0);
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn move_left_decrements_col() {
        let mut cursor = Cursor { line: 0, col: 5 };
        cursor.move_left();
        assert_eq!(cursor.col, 4);
    }

    #[test]
    fn move_left_stops_at_zero() {
        let mut cursor = Cursor::new();
        cursor.move_left();
        assert_eq!(cursor.col, 0);
    }

    #[test]
    fn move_right_increments_col() {
        let mut cursor = Cursor::new();
        cursor.move_right(10);
        assert_eq!(cursor.col, 1);
    }

    #[test]
    fn move_right_clamps_to_line_len() {
        let mut cursor = Cursor { line: 0, col: 9 };
        cursor.move_right(10);
        assert_eq!(cursor.col, 10);
        cursor.move_right(10);
        assert_eq!(cursor.col, 10); // stays at max
    }

    #[test]
    fn move_up_decrements_line() {
        let mut cursor = Cursor { line: 5, col: 0 };
        cursor.move_up();
        assert_eq!(cursor.line, 4);
    }

    #[test]
    fn move_up_stops_at_zero() {
        let mut cursor = Cursor::new();
        cursor.move_up();
        assert_eq!(cursor.line, 0);
    }

    #[test]
    fn move_down_increments_line() {
        let mut cursor = Cursor::new();
        cursor.move_down(10);
        assert_eq!(cursor.line, 1);
    }

    #[test]
    fn move_down_clamps_to_last_line() {
        let mut cursor = Cursor { line: 8, col: 0 };
        cursor.move_down(10); // 10 lines = indices 0-9
        assert_eq!(cursor.line, 9);
        cursor.move_down(10);
        assert_eq!(cursor.line, 9); // stays at max
    }
}
