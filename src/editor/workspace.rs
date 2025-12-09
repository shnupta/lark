use std::path::PathBuf;

use super::Mode;
use super::layout::Rect;
use super::pane::PaneId;
use super::tab::Tab;

/// The workspace manages tabs, each containing panes
pub struct Workspace {
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub command_buffer: String,
    pub message: Option<String>,
    pub running: bool,
    pub pending_keys: String,
    pub selecting_pane: bool,
    pub theme_name: String,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            tabs: vec![Tab::new()],
            active_tab: 0,
            command_buffer: String::new(),
            message: None,
            running: true,
            pending_keys: String::new(),
            selecting_pane: false,
            theme_name: "gruvbox-dark".to_string(),
        }
    }

    pub fn open(path: PathBuf) -> Self {
        Self {
            tabs: vec![Tab::with_file(path)],
            active_tab: 0,
            command_buffer: String::new(),
            message: None,
            running: true,
            pending_keys: String::new(),
            selecting_pane: false,
            theme_name: "gruvbox-dark".to_string(),
        }
    }

    // Tab access

    pub fn tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    pub fn tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    // Delegate to current tab for pane access

    pub fn focused_pane(&self) -> &super::Pane {
        self.tab().focused_pane()
    }

    pub fn focused_pane_mut(&mut self) -> &mut super::Pane {
        self.tab_mut().focused_pane_mut()
    }

    pub fn pane(&self, id: PaneId) -> Option<&super::Pane> {
        self.tab().pane(id)
    }

    pub fn calculate_rects(&self, area: Rect) -> Vec<(PaneId, Rect)> {
        self.tab().calculate_rects(area)
    }

    pub fn is_focused(&self, pane_id: PaneId) -> bool {
        self.tab().is_focused(pane_id)
    }

    pub fn mode(&self) -> Mode {
        if !self.command_buffer.is_empty() || self.tab().focused_pane().mode == Mode::Command {
            Mode::Command
        } else {
            self.tab().focused_pane().mode
        }
    }

    // Delegate split operations to current tab

    pub fn split_vertical(&mut self) {
        self.tab_mut().split_vertical();
    }

    pub fn split_horizontal(&mut self) {
        self.tab_mut().split_horizontal();
    }

    pub fn focus_next(&mut self) {
        self.tab_mut().focus_next();
    }

    pub fn get_editor_panes_with_labels(&self) -> Vec<(char, PaneId)> {
        self.tab().get_editor_panes_with_labels()
    }

    pub fn focus_pane_by_label(&mut self, label: char) -> bool {
        self.tab_mut().focus_pane_by_label(label)
    }

    pub fn open_file_in_pane(&mut self, path: PathBuf, label: char) -> bool {
        let result = self.tab_mut().open_file_in_pane(path, label);
        self.tab_mut().update_name();
        result
    }

    pub fn close_focused_pane(&mut self) -> bool {
        self.tab_mut().close_focused_pane()
    }

    // File browser (delegates to current tab)

    pub fn toggle_file_browser(&mut self) {
        self.tab_mut().toggle_file_browser();
    }

    pub fn focus_file_browser(&mut self) {
        self.tab_mut().focus_file_browser();
    }

    pub fn try_open_file_from_browser(&mut self) -> Option<PathBuf> {
        let result = self.tab_mut().try_open_file_from_browser();
        self.tab_mut().update_name();
        result
    }

    // Access file browser from current tab
    pub fn file_browser(&self) -> &super::file_browser::FileBrowser {
        &self.tab().file_browser
    }

    pub fn file_browser_mut(&mut self) -> &mut super::file_browser::FileBrowser {
        &mut self.tab_mut().file_browser
    }

    // Tab management

    pub fn new_tab(&mut self) {
        self.tabs.push(Tab::new());
        self.active_tab = self.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = (self.active_tab + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active_tab = if self.active_tab == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab - 1
            };
        }
    }

    pub fn close_tab(&mut self) -> bool {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
            true
        } else {
            false
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    // Messages

    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = Some(msg.into());
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn set_theme(&mut self, name: &str) {
        self.theme_name = name.to_string();
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workspace_has_one_tab() {
        let ws = Workspace::new();
        assert_eq!(ws.tab_count(), 1);
        assert_eq!(ws.active_tab, 0);
    }

    #[test]
    fn new_tab_adds_and_focuses() {
        let mut ws = Workspace::new();
        ws.new_tab();

        assert_eq!(ws.tab_count(), 2);
        assert_eq!(ws.active_tab, 1);
    }

    #[test]
    fn next_tab_cycles_forward() {
        let mut ws = Workspace::new();
        ws.new_tab();
        ws.new_tab();
        // Now at tab 2 (index 2)

        ws.active_tab = 0;
        ws.next_tab();
        assert_eq!(ws.active_tab, 1);

        ws.next_tab();
        assert_eq!(ws.active_tab, 2);

        ws.next_tab();
        assert_eq!(ws.active_tab, 0); // wraps around
    }

    #[test]
    fn prev_tab_cycles_backward() {
        let mut ws = Workspace::new();
        ws.new_tab();
        ws.new_tab();

        ws.active_tab = 0;
        ws.prev_tab();
        assert_eq!(ws.active_tab, 2); // wraps to last

        ws.prev_tab();
        assert_eq!(ws.active_tab, 1);
    }

    #[test]
    fn close_tab_removes_current() {
        let mut ws = Workspace::new();
        ws.new_tab();
        ws.new_tab();
        assert_eq!(ws.tab_count(), 3);
        ws.active_tab = 1;

        let closed = ws.close_tab();

        assert!(closed);
        assert_eq!(ws.tab_count(), 2);
        assert_eq!(ws.active_tab, 1); // stays at 1 (now points to what was tab 2)
    }

    #[test]
    fn close_tab_adjusts_index_when_at_end() {
        let mut ws = Workspace::new();
        ws.new_tab();
        // active_tab is now 1 (last tab)

        let closed = ws.close_tab();

        assert!(closed);
        assert_eq!(ws.tab_count(), 1);
        assert_eq!(ws.active_tab, 0); // adjusted to last valid index
    }

    #[test]
    fn close_tab_fails_with_single_tab() {
        let mut ws = Workspace::new();

        let closed = ws.close_tab();

        assert!(!closed);
        assert_eq!(ws.tab_count(), 1);
    }

    #[test]
    fn next_tab_does_nothing_with_single_tab() {
        let mut ws = Workspace::new();
        ws.next_tab();
        assert_eq!(ws.active_tab, 0);
    }

    #[test]
    fn message_can_be_set_and_cleared() {
        let mut ws = Workspace::new();
        assert!(ws.message.is_none());

        ws.set_message("Hello");
        assert_eq!(ws.message, Some("Hello".to_string()));

        ws.clear_message();
        assert!(ws.message.is_none());
    }

    #[test]
    fn quit_sets_running_to_false() {
        let mut ws = Workspace::new();
        assert!(ws.running);

        ws.quit();

        assert!(!ws.running);
    }
}
