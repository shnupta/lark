use std::io::{self, Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::editor::{Mode, PaneKind, Rect, Workspace};

pub struct Renderer {
    pub width: u16,
    pub height: u16,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self { width, height })
    }

    pub fn setup() -> io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            stdout(),
            EnterAlternateScreen,
            DisableLineWrap,
            Hide,
            Clear(ClearType::All)
        )?;
        Ok(())
    }

    pub fn teardown() -> io::Result<()> {
        execute!(
            stdout(),
            SetCursorStyle::DefaultUserShape,
            Show,
            EnableLineWrap,
            LeaveAlternateScreen
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn text_height(&self, workspace: &Workspace) -> usize {
        let tab_bar_height = if workspace.tab_count() > 1 { 1 } else { 0 };
        self.height.saturating_sub(1 + tab_bar_height) as usize
    }

    pub fn render(&self, workspace: &Workspace) -> io::Result<()> {
        let mut stdout = stdout();

        // Hide cursor during redraw to prevent flicker
        queue!(stdout, Hide)?;

        let has_tabs = workspace.tab_count() > 1;
        let tab_bar_height = if has_tabs { 1u16 } else { 0 };

        // Render tab bar if multiple tabs
        if has_tabs {
            self.render_tab_bar(&mut stdout, workspace)?;
        }

        // Calculate layout - reserve lines for tab bar (if any) and status
        let content_area = Rect::new(
            0,
            tab_bar_height,
            self.width,
            self.height.saturating_sub(1 + tab_bar_height),
        );
        let pane_rects = workspace.calculate_rects(content_area);

        // Render each pane
        for (pane_id, rect) in &pane_rects {
            if let Some(pane) = workspace.pane(*pane_id) {
                let is_focused = workspace.is_focused(*pane_id);
                match pane.kind {
                    PaneKind::Editor => {
                        self.render_editor_pane(&mut stdout, pane, rect, is_focused)?
                    }
                    PaneKind::FileBrowser => {
                        self.render_file_browser_pane(&mut stdout, workspace, rect, is_focused)?
                    }
                }
            }
        }

        // If selecting pane, show overlay labels
        if workspace.selecting_pane {
            let labeled_panes = workspace.get_editor_panes_with_labels();
            for (label, pane_id) in labeled_panes {
                if let Some((_, rect)) = pane_rects.iter().find(|(id, _)| *id == pane_id) {
                    let center_x = rect.x + rect.width / 2;
                    let center_y = rect.y + rect.height / 2;

                    queue!(stdout, MoveTo(center_x.saturating_sub(2), center_y))?;
                    queue!(stdout, SetForegroundColor(Color::Black))?;
                    queue!(stdout, SetBackgroundColor(Color::Yellow))?;
                    queue!(stdout, SetAttribute(Attribute::Bold))?;
                    queue!(stdout, Print(format!(" {} ", label.to_ascii_uppercase())))?;
                    queue!(stdout, SetAttribute(Attribute::Reset))?;
                    queue!(stdout, SetForegroundColor(Color::Reset))?;
                    queue!(stdout, SetBackgroundColor(Color::Reset))?;
                }
            }
        }

        // Render global status line
        self.render_status_line(&mut stdout, workspace)?;

        // Position cursor in focused pane
        let focused_pane = workspace.focused_pane();
        if let Some((_, rect)) = pane_rects.iter().find(|(id, _)| *id == focused_pane.id) {
            if workspace.mode() == Mode::Command {
                let cmd_col = 1 + workspace.command_buffer.len() as u16;
                let cmd_row = self.height.saturating_sub(1);
                queue!(stdout, MoveTo(cmd_col, cmd_row))?;
                queue!(stdout, SetCursorStyle::BlinkingBar)?;
                queue!(stdout, Show)?;
            } else if focused_pane.kind == PaneKind::Editor {
                let gutter_width = 4u16;
                let cursor_x = rect.x + gutter_width + focused_pane.cursor.col as u16;
                let cursor_y =
                    rect.y + (focused_pane.cursor.line - focused_pane.scroll_offset) as u16;
                queue!(stdout, MoveTo(cursor_x, cursor_y))?;

                let cursor_style = match focused_pane.mode {
                    Mode::Insert => SetCursorStyle::BlinkingBar,
                    _ => SetCursorStyle::SteadyBlock,
                };
                queue!(stdout, cursor_style)?;
                queue!(stdout, Show)?;
            } else {
                queue!(stdout, Hide)?;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn render_tab_bar(&self, stdout: &mut impl Write, workspace: &Workspace) -> io::Result<()> {
        queue!(stdout, MoveTo(0, 0))?;

        let mut tab_display = String::new();
        for (i, tab) in workspace.tabs.iter().enumerate() {
            if i == workspace.active_tab {
                tab_display.push_str(&format!(" [{}] ", tab.name));
            } else {
                tab_display.push_str(&format!("  {}  ", tab.name));
            }
        }

        let padded: String = format!("{:width$}", tab_display, width = self.width as usize)
            .chars()
            .take(self.width as usize)
            .collect();

        queue!(stdout, SetBackgroundColor(Color::DarkGrey))?;
        queue!(stdout, SetForegroundColor(Color::White))?;
        queue!(stdout, Print(&padded))?;
        queue!(stdout, SetBackgroundColor(Color::Reset))?;
        queue!(stdout, SetForegroundColor(Color::Reset))?;

        Ok(())
    }

    fn render_editor_pane(
        &self,
        stdout: &mut impl Write,
        pane: &crate::editor::Pane,
        rect: &Rect,
        is_focused: bool,
    ) -> io::Result<()> {
        let line_count = pane.buffer.line_count();
        let gutter_width = 4;
        let text_width = rect.width.saturating_sub(gutter_width) as usize;

        for row in 0..rect.height {
            let line_idx = row as usize + pane.scroll_offset;
            queue!(stdout, MoveTo(rect.x, rect.y + row))?;

            if line_idx < line_count {
                let is_cursor_line = line_idx == pane.cursor.line;

                let line_num = if is_cursor_line {
                    line_idx + 1
                } else {
                    (line_idx as isize - pane.cursor.line as isize).unsigned_abs()
                };

                if is_cursor_line {
                    queue!(stdout, SetForegroundColor(Color::Yellow))?;
                } else {
                    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
                }

                queue!(stdout, Print(format!("{:>3} ", line_num)))?;
                queue!(stdout, SetForegroundColor(Color::Reset))?;

                let line = pane.buffer.line(line_idx);
                let line_str: String = line.chars().take(text_width).collect();
                let content = line_str.trim_end_matches('\n').to_string();

                let padded: String = format!("{:width$}", content, width = text_width)
                    .chars()
                    .take(text_width)
                    .collect();

                if !is_focused {
                    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
                }
                queue!(stdout, Print(&padded))?;
                if !is_focused {
                    queue!(stdout, SetForegroundColor(Color::Reset))?;
                }
            } else {
                queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
                queue!(stdout, Print("  ~ "))?;
                queue!(stdout, SetForegroundColor(Color::Reset))?;

                let blank = " ".repeat(text_width);
                queue!(stdout, Print(&blank))?;
            }
        }

        Ok(())
    }

    fn render_file_browser_pane(
        &self,
        stdout: &mut impl Write,
        workspace: &Workspace,
        rect: &Rect,
        is_focused: bool,
    ) -> io::Result<()> {
        // Title row
        queue!(stdout, MoveTo(rect.x, rect.y))?;
        queue!(stdout, SetAttribute(Attribute::Bold))?;
        let title = " Files ";
        let title: String = title.chars().take(rect.width as usize).collect();
        let padded: String = format!("{:width$}", title, width = rect.width as usize)
            .chars()
            .take(rect.width as usize)
            .collect();
        queue!(stdout, Print(&padded))?;
        queue!(stdout, SetAttribute(Attribute::Reset))?;

        let file_browser = workspace.file_browser();

        // File list
        for row in 1..rect.height {
            let idx = row as usize - 1;
            queue!(stdout, MoveTo(rect.x, rect.y + row))?;

            if let Some(entry) = file_browser.entries.get(idx) {
                let is_selected = idx == file_browser.selected;

                if is_selected && is_focused {
                    queue!(stdout, SetAttribute(Attribute::Reverse))?;
                }

                let indent = "  ".repeat(entry.depth);
                let icon = if entry.is_dir {
                    if file_browser.is_expanded(&entry.path) {
                        "▾ "
                    } else {
                        "▸ "
                    }
                } else {
                    "  "
                };

                let available_width = rect.width as usize;
                let prefix = format!("{}{}", indent, icon);
                let name_width = available_width.saturating_sub(prefix.len());
                let name: String = entry.name.chars().take(name_width).collect();
                let display = format!("{}{}", prefix, name);
                let padded: String = format!("{:width$}", display, width = available_width)
                    .chars()
                    .take(available_width)
                    .collect();

                if entry.is_dir {
                    queue!(stdout, SetForegroundColor(Color::Blue))?;
                } else if !is_focused {
                    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
                }
                queue!(stdout, Print(&padded))?;
                queue!(stdout, SetForegroundColor(Color::Reset))?;

                if is_selected && is_focused {
                    queue!(stdout, SetAttribute(Attribute::Reset))?;
                }
            } else {
                let blank = " ".repeat(rect.width as usize);
                queue!(stdout, Print(&blank))?;
            }
        }

        Ok(())
    }

    fn render_status_line(&self, stdout: &mut impl Write, workspace: &Workspace) -> io::Result<()> {
        let status_row = self.height.saturating_sub(1);
        queue!(stdout, MoveTo(0, status_row))?;
        queue!(stdout, Clear(ClearType::CurrentLine))?;

        if workspace.mode() == Mode::Command {
            queue!(stdout, Print(format!(":{}", workspace.command_buffer)))?;
            return Ok(());
        }

        if let Some(ref msg) = workspace.message {
            queue!(stdout, Print(msg))?;
            return Ok(());
        }

        queue!(stdout, SetAttribute(Attribute::Reverse))?;

        let pane = workspace.focused_pane();
        let mode = pane.mode.display();
        let filename = pane
            .buffer
            .path()
            .map(|p| {
                p.file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "[No Name]".to_string())
            })
            .unwrap_or_else(|| "[No Name]".to_string());
        let position = format!("{}:{}", pane.cursor.line + 1, pane.cursor.col + 1);

        let pending = if !workspace.pending_keys.is_empty() {
            format!(" [{}]", workspace.pending_keys)
        } else {
            String::new()
        };

        let left = format!(" {} | {}{} ", mode, filename, pending);
        let right = format!(" {} ", position);

        let padding = self.width as usize - left.len() - right.len();
        let middle = " ".repeat(padding.max(0));

        let status = format!("{}{}{}", left, middle, right);
        let status: String = status.chars().take(self.width as usize).collect();

        queue!(stdout, Print(status))?;
        queue!(stdout, SetAttribute(Attribute::Reset))?;

        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new().expect("Failed to create renderer")
    }
}
