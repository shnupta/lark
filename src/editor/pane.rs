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

/// A pane represents a single view in the editor
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
