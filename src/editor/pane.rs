use super::{Buffer, Cursor, Mode};
use crate::syntax::{Highlighter, Language};
use std::path::PathBuf;

/// Unique identifier for a pane
pub type PaneId = usize;

/// Content type that a pane can display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneKind {
    Editor,
    FileBrowser,
}

/// A pane represents a single view in the editor (back to simple, no tabs)
pub struct Pane {
    pub id: PaneId,
    pub kind: PaneKind,
    pub buffer: Buffer,
    pub cursor: Cursor,
    pub scroll_offset: usize, // Vertical scroll
    pub scroll_col: usize,    // Horizontal scroll
    pub mode: Mode,
    pub highlighter: Highlighter,
    pub language: Language,
}

impl Pane {
    pub fn new_editor(id: PaneId) -> Self {
        Self {
            id,
            kind: PaneKind::Editor,
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            scroll_offset: 0,
            scroll_col: 0,
            mode: Mode::Normal,
            highlighter: Highlighter::new(),
            language: Language::Unknown,
        }
    }

    pub fn new_editor_with_file(id: PaneId, path: PathBuf) -> Self {
        let buffer = Buffer::from_file(path.clone());
        let mut highlighter = Highlighter::new();
        let language = Language::from_path(&path);

        // Set language and parse if grammar is available
        if highlighter.set_language(language) {
            highlighter.parse(&buffer.text());
        }

        Self {
            id,
            kind: PaneKind::Editor,
            buffer,
            cursor: Cursor::new(),
            scroll_offset: 0,
            scroll_col: 0,
            mode: Mode::Normal,
            highlighter,
            language,
        }
    }

    pub fn new_file_browser(id: PaneId) -> Self {
        Self {
            id,
            kind: PaneKind::FileBrowser,
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            scroll_offset: 0,
            scroll_col: 0,
            mode: Mode::FileBrowser,
            highlighter: Highlighter::new(),
            language: Language::Unknown,
        }
    }

    /// Re-parse the buffer for syntax highlighting
    pub fn reparse(&mut self) {
        if self.language != Language::Unknown {
            self.highlighter.parse(&self.buffer.text());
        }
    }

    /// Set language and reparse
    pub fn set_language(&mut self, lang: Language) {
        self.language = lang;
        if self.highlighter.set_language(lang) {
            self.highlighter.parse(&self.buffer.text());
        }
    }

    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        // Vertical scroll
        if self.cursor.line < self.scroll_offset {
            self.scroll_offset = self.cursor.line;
        }
        if self.cursor.line >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor.line - viewport_height + 1;
        }
    }

    pub fn adjust_scroll_horizontal(&mut self, viewport_width: usize) {
        // Horizontal scroll - keep some margin
        let margin = 5.min(viewport_width / 4);

        if self.cursor.col < self.scroll_col {
            self.scroll_col = self.cursor.col;
        }
        if self.cursor.col >= self.scroll_col + viewport_width.saturating_sub(margin) {
            self.scroll_col = self
                .cursor
                .col
                .saturating_sub(viewport_width.saturating_sub(margin - 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_editor_pane_has_correct_defaults() {
        let pane = Pane::new_editor(0);
        assert_eq!(pane.id, 0);
        assert_eq!(pane.kind, PaneKind::Editor);
        assert_eq!(pane.mode, Mode::Normal);
        assert_eq!(pane.scroll_offset, 0);
    }

    #[test]
    fn new_file_browser_pane_has_correct_mode() {
        let pane = Pane::new_file_browser(1);
        assert_eq!(pane.kind, PaneKind::FileBrowser);
        assert_eq!(pane.mode, Mode::FileBrowser);
    }

    #[test]
    fn adjust_scroll_scrolls_down_when_cursor_below_viewport() {
        let mut pane = Pane::new_editor(0);
        pane.cursor.line = 25;
        pane.scroll_offset = 0;

        pane.adjust_scroll(20); // viewport of 20 lines

        // Cursor at 25 should scroll so cursor is visible
        // scroll_offset = cursor - viewport + 1 = 25 - 20 + 1 = 6
        assert_eq!(pane.scroll_offset, 6);
    }

    #[test]
    fn adjust_scroll_scrolls_up_when_cursor_above_viewport() {
        let mut pane = Pane::new_editor(0);
        pane.cursor.line = 5;
        pane.scroll_offset = 10;

        pane.adjust_scroll(20);

        // Cursor at 5 is above scroll_offset of 10, so scroll up
        assert_eq!(pane.scroll_offset, 5);
    }

    #[test]
    fn adjust_scroll_does_nothing_when_cursor_visible() {
        let mut pane = Pane::new_editor(0);
        pane.cursor.line = 10;
        pane.scroll_offset = 5;

        pane.adjust_scroll(20);

        // Cursor at 10 is within viewport (5..25), no change needed
        assert_eq!(pane.scroll_offset, 5);
    }
}
