use super::{Buffer, Cursor, Mode};
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
    pub scroll_offset: usize,
    pub mode: Mode,
}

impl Pane {
    pub fn new_editor(id: PaneId) -> Self {
        Self {
            id,
            kind: PaneKind::Editor,
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            scroll_offset: 0,
            mode: Mode::Normal,
        }
    }

    pub fn new_editor_with_file(id: PaneId, path: PathBuf) -> Self {
        Self {
            id,
            kind: PaneKind::Editor,
            buffer: Buffer::from_file(path),
            cursor: Cursor::new(),
            scroll_offset: 0,
            mode: Mode::Normal,
        }
    }

    pub fn new_file_browser(id: PaneId) -> Self {
        Self {
            id,
            kind: PaneKind::FileBrowser,
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            scroll_offset: 0,
            mode: Mode::FileBrowser,
        }
    }

    pub fn adjust_scroll(&mut self, viewport_height: usize) {
        if self.cursor.line < self.scroll_offset {
            self.scroll_offset = self.cursor.line;
        }
        if self.cursor.line >= self.scroll_offset + viewport_height {
            self.scroll_offset = self.cursor.line - viewport_height + 1;
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
