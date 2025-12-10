use std::collections::HashMap;
use std::path::PathBuf;

use super::file_browser::FileBrowser;
use super::layout::{Layout, Rect, SplitDirection};
use super::pane::{Pane, PaneId, PaneKind};
use super::{Buffer, Cursor};
use crate::syntax::Language;

/// A tab contains multiple panes with their layout
pub struct Tab {
    pub panes: HashMap<PaneId, Pane>,
    pub layout: Layout,
    pub focused_pane_id: PaneId,
    next_pane_id: PaneId,
    pub file_browser: FileBrowser,
    pub file_browser_pane_id: Option<PaneId>,
    pub name: String,
}

impl Tab {
    pub fn new() -> Self {
        let initial_pane = Pane::new_editor(0);
        let mut panes = HashMap::new();
        panes.insert(0, initial_pane);

        Self {
            panes,
            layout: Layout::new(0),
            focused_pane_id: 0,
            next_pane_id: 1,
            file_browser: FileBrowser::new(),
            file_browser_pane_id: None,
            name: "[No Name]".to_string(),
        }
    }

    pub fn with_file(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "[No Name]".to_string());

        let initial_pane = Pane::new_editor_with_file(0, path);
        let mut panes = HashMap::new();
        panes.insert(0, initial_pane);

        Self {
            panes,
            layout: Layout::new(0),
            focused_pane_id: 0,
            next_pane_id: 1,
            file_browser: FileBrowser::new(),
            file_browser_pane_id: None,
            name,
        }
    }

    pub fn focused_pane(&self) -> &Pane {
        self.panes
            .get(&self.focused_pane_id)
            .expect("Focused pane should exist")
    }

    pub fn focused_pane_mut(&mut self) -> &mut Pane {
        self.panes
            .get_mut(&self.focused_pane_id)
            .expect("Focused pane should exist")
    }

    pub fn pane(&self, id: PaneId) -> Option<&Pane> {
        self.panes.get(&id)
    }

    pub fn calculate_rects(&self, area: Rect) -> Vec<(PaneId, Rect)> {
        self.layout.calculate_rects(area)
    }

    pub fn is_focused(&self, pane_id: PaneId) -> bool {
        self.focused_pane_id == pane_id
    }

    // Split operations

    pub fn split_vertical(&mut self) {
        let new_id = self.next_pane_id;
        self.next_pane_id += 1;

        let new_pane = Pane::new_editor(new_id);

        self.panes.insert(new_id, new_pane);
        self.layout
            .split_pane(self.focused_pane_id, new_id, SplitDirection::Vertical);
    }

    pub fn split_horizontal(&mut self) {
        let new_id = self.next_pane_id;
        self.next_pane_id += 1;

        let new_pane = Pane::new_editor(new_id);

        self.panes.insert(new_id, new_pane);
        self.layout
            .split_pane(self.focused_pane_id, new_id, SplitDirection::Horizontal);
    }

    pub fn focus_next(&mut self) {
        let pane_ids = self.layout.pane_ids();
        if let Some(pos) = pane_ids.iter().position(|&id| id == self.focused_pane_id) {
            let next_pos = (pos + 1) % pane_ids.len();
            self.focused_pane_id = pane_ids[next_pos];
        }
    }

    pub fn focus_direction(&mut self, direction: super::layout::Direction, area: Rect) {
        if let Some(target_id) =
            self.layout
                .find_pane_in_direction(self.focused_pane_id, direction, area)
        {
            self.focused_pane_id = target_id;
        }
    }

    pub fn get_editor_panes_with_labels(&self) -> Vec<(char, PaneId)> {
        self.layout
            .pane_ids()
            .into_iter()
            .filter(|&id| {
                self.panes
                    .get(&id)
                    .map(|p| p.kind == PaneKind::Editor)
                    .unwrap_or(false)
            })
            .enumerate()
            .map(|(i, id)| ((b'a' + i as u8) as char, id))
            .collect()
    }

    pub fn focus_pane_by_label(&mut self, label: char) -> bool {
        let labeled = self.get_editor_panes_with_labels();
        if let Some((_, pane_id)) = labeled.iter().find(|(l, _)| *l == label) {
            self.focused_pane_id = *pane_id;
            true
        } else {
            false
        }
    }

    pub fn open_file_in_pane(&mut self, path: PathBuf, label: char) -> bool {
        let labeled = self.get_editor_panes_with_labels();
        if let Some((_, pane_id)) = labeled.iter().find(|(l, _)| *l == label) {
            if let Some(pane) = self.panes.get_mut(pane_id) {
                pane.buffer = Buffer::from_file(path.clone());
                pane.cursor = Cursor::new();
                pane.scroll_offset = 0;

                // Set up syntax highlighting
                let lang = Language::from_path(&path);
                pane.language = lang;
                if pane.highlighter.set_language(lang) {
                    pane.highlighter.parse(&pane.buffer.text());
                }

                self.focused_pane_id = *pane_id;
                return true;
            }
        }
        false
    }

    pub fn open_file_in_focused_pane(&mut self, path: PathBuf) {
        if let Some(pane) = self.panes.get_mut(&self.focused_pane_id) {
            pane.buffer = Buffer::from_file(path.clone());
            pane.cursor = Cursor::new();
            pane.scroll_offset = 0;

            // Set up syntax highlighting
            let lang = Language::from_path(&path);
            pane.language = lang;
            if pane.highlighter.set_language(lang) {
                pane.highlighter.parse(&pane.buffer.text());
            }
        }
    }

    /// Close the current pane. Returns true if closed, false if it was the last pane.
    pub fn close_focused_pane(&mut self) -> bool {
        let pane_ids = self.layout.pane_ids();
        if pane_ids.len() <= 1 {
            return false;
        }

        if Some(self.focused_pane_id) == self.file_browser_pane_id {
            self.file_browser_pane_id = None;
        }

        let closed_id = self.focused_pane_id;
        self.focus_next();
        self.layout.remove_pane(closed_id);
        self.panes.remove(&closed_id);
        true
    }

    // File browser

    pub fn toggle_file_browser(&mut self) {
        if let Some(fb_id) = self.file_browser_pane_id {
            if self.focused_pane_id == fb_id {
                self.focus_next();
            }
            self.layout.remove_pane(fb_id);
            self.panes.remove(&fb_id);
            self.file_browser_pane_id = None;
        } else {
            self.open_file_browser();
        }
    }

    fn open_file_browser(&mut self) {
        let new_id = self.next_pane_id;
        self.next_pane_id += 1;

        let fb_pane = Pane::new_file_browser(new_id);
        self.panes.insert(new_id, fb_pane);
        self.layout.add_left_pane(new_id, 0.2);
        self.file_browser_pane_id = Some(new_id);
        self.file_browser.refresh();
        self.focused_pane_id = new_id;
    }

    pub fn focus_file_browser(&mut self) {
        if let Some(fb_id) = self.file_browser_pane_id {
            self.focused_pane_id = fb_id;
        } else {
            self.open_file_browser();
        }
    }

    pub fn try_open_file_from_browser(&mut self) -> Option<PathBuf> {
        let editor_panes = self.get_editor_panes_with_labels();

        if editor_panes.len() <= 1 {
            if let Some(path) = self.file_browser.select() {
                if let Some((_, pane_id)) = editor_panes.first() {
                    if let Some(pane) = self.panes.get_mut(pane_id) {
                        pane.buffer = Buffer::from_file(path.clone());
                        pane.cursor = Cursor::new();
                        pane.scroll_offset = 0;

                        // Set up syntax highlighting
                        let lang = Language::from_path(&path);
                        pane.language = lang;
                        if pane.highlighter.set_language(lang) {
                            pane.highlighter.parse(&pane.buffer.text());
                        }
                    }
                    self.focused_pane_id = *pane_id;
                }
            }
            None
        } else {
            self.file_browser.select()
        }
    }

    /// Update tab name based on focused pane's buffer
    pub fn update_name(&mut self) {
        if let Some(pane) = self.panes.get(&self.focused_pane_id) {
            if pane.kind == PaneKind::Editor {
                self.name = pane
                    .buffer
                    .path()
                    .map(|p| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "[No Name]".to_string())
                    })
                    .unwrap_or_else(|| "[No Name]".to_string());
            }
        }
    }
}

impl Default for Tab {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tab_has_one_pane() {
        let tab = Tab::new();
        assert_eq!(tab.panes.len(), 1);
        assert_eq!(tab.focused_pane_id, 0);
    }

    #[test]
    fn split_vertical_adds_pane() {
        let mut tab = Tab::new();
        tab.split_vertical();

        assert_eq!(tab.panes.len(), 2);
        // New pane should exist
        assert!(tab.panes.contains_key(&1));
    }

    #[test]
    fn split_horizontal_adds_pane() {
        let mut tab = Tab::new();
        tab.split_horizontal();

        assert_eq!(tab.panes.len(), 2);
    }

    #[test]
    fn focus_next_cycles_through_panes() {
        let mut tab = Tab::new();
        tab.split_vertical();
        // Now have 2 panes

        let initial = tab.focused_pane_id;
        tab.focus_next();
        let after_first = tab.focused_pane_id;

        // Should have changed
        assert_ne!(initial, after_first);

        // Should cycle back
        tab.focus_next();
        assert_eq!(tab.focused_pane_id, initial);
    }

    #[test]
    fn close_focused_pane_removes_pane() {
        let mut tab = Tab::new();
        tab.split_vertical();
        assert_eq!(tab.panes.len(), 2);

        let closed = tab.close_focused_pane();
        assert!(closed);
        assert_eq!(tab.panes.len(), 1);
    }

    #[test]
    fn close_focused_pane_fails_with_single_pane() {
        let mut tab = Tab::new();

        let closed = tab.close_focused_pane();
        assert!(!closed);
        assert_eq!(tab.panes.len(), 1);
    }

    #[test]
    fn get_editor_panes_with_labels_assigns_letters() {
        let mut tab = Tab::new();
        tab.split_vertical();
        tab.split_vertical();

        let labeled = tab.get_editor_panes_with_labels();

        assert_eq!(labeled.len(), 3);
        assert_eq!(labeled[0].0, 'a');
        assert_eq!(labeled[1].0, 'b');
        assert_eq!(labeled[2].0, 'c');
    }

    #[test]
    fn focus_pane_by_label_works() {
        let mut tab = Tab::new();
        tab.split_vertical();

        tab.focused_pane_id = 0;
        let focused = tab.focus_pane_by_label('b');

        assert!(focused);
        assert_eq!(tab.focused_pane_id, 1);
    }

    #[test]
    fn focus_pane_by_invalid_label_fails() {
        let mut tab = Tab::new();

        let focused = tab.focus_pane_by_label('z');
        assert!(!focused);
    }

    #[test]
    fn toggle_file_browser_opens_and_closes() {
        let mut tab = Tab::new();
        assert!(tab.file_browser_pane_id.is_none());

        tab.toggle_file_browser();
        assert!(tab.file_browser_pane_id.is_some());
        assert_eq!(tab.panes.len(), 2);

        tab.toggle_file_browser();
        assert!(tab.file_browser_pane_id.is_none());
        assert_eq!(tab.panes.len(), 1);
    }
}
