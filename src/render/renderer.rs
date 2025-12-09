use std::io::{self, Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::{Attribute, Print, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::editor::{Mode, PaneKind, Rect, Workspace};
use crate::theme::Theme;

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

    /// Calculate the height of the focused pane for scroll adjustment
    pub fn focused_pane_height(&self, workspace: &Workspace) -> usize {
        let has_tabs = workspace.tab_count() > 1;
        let tab_bar_height = if has_tabs { 1u16 } else { 0 };
        let content_area = Rect::new(
            0,
            tab_bar_height,
            self.width,
            self.height.saturating_sub(1 + tab_bar_height),
        );
        let pane_rects = workspace.calculate_rects(content_area);

        // Find the focused pane's rect
        for (pane_id, rect) in &pane_rects {
            if workspace.is_focused(*pane_id) {
                return rect.height as usize;
            }
        }

        // Fallback to full content area
        content_area.height as usize
    }

    pub fn render(&self, workspace: &mut Workspace, theme: &Theme) -> io::Result<()> {
        let mut stdout = stdout();

        // Update terminal size in workspace for directional navigation
        workspace.terminal_size = (self.width, self.height);

        // Hide cursor during redraw to prevent flicker
        queue!(stdout, Hide)?;

        // Set background color for entire screen
        queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;

        let has_tabs = workspace.tab_count() > 1;
        let tab_bar_height = if has_tabs { 1u16 } else { 0 };

        // Render tab bar if multiple tabs
        if has_tabs {
            self.render_tab_bar(&mut stdout, workspace, theme)?;
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
                match pane.kind {
                    PaneKind::Editor => self.render_editor_pane(&mut stdout, pane, rect, theme)?,
                    PaneKind::FileBrowser => {
                        let is_focused = workspace.is_focused(*pane_id);
                        self.render_file_browser_pane(
                            &mut stdout,
                            workspace,
                            rect,
                            is_focused,
                            theme,
                        )?
                    }
                }
            }
        }

        // Render pane borders (only if there are multiple panes)
        if pane_rects.len() > 1 {
            self.render_pane_borders(&mut stdout, workspace, &pane_rects, theme)?;
        }

        // If selecting pane, show overlay labels
        if workspace.selecting_pane {
            self.render_pane_labels(&mut stdout, workspace, &pane_rects, theme)?;
        }

        // Render global status line
        self.render_status_line(&mut stdout, workspace, theme)?;

        // Position cursor in focused pane
        self.position_cursor(&mut stdout, workspace, &pane_rects, theme)?;

        stdout.flush()?;
        Ok(())
    }

    fn render_tab_bar(
        &self,
        stdout: &mut impl Write,
        workspace: &Workspace,
        theme: &Theme,
    ) -> io::Result<()> {
        queue!(stdout, MoveTo(0, 0))?;
        queue!(stdout, SetBackgroundColor(theme.tab_bar_bg.to_crossterm()))?;

        let mut x = 0u16;
        for (i, tab) in workspace.tabs.iter().enumerate() {
            let is_active = i == workspace.active_tab;

            if is_active {
                queue!(
                    stdout,
                    SetBackgroundColor(theme.tab_active_bg.to_crossterm())
                )?;
                queue!(
                    stdout,
                    SetForegroundColor(theme.tab_active_fg.to_crossterm())
                )?;
            } else {
                queue!(stdout, SetBackgroundColor(theme.tab_bar_bg.to_crossterm()))?;
                queue!(stdout, SetForegroundColor(theme.tab_bar_fg.to_crossterm()))?;
            }

            let tab_text = if is_active {
                format!(" [{}] ", tab.name)
            } else {
                format!("  {}  ", tab.name)
            };

            queue!(stdout, Print(&tab_text))?;
            x += tab_text.len() as u16;
        }

        // Fill remaining space
        queue!(stdout, SetBackgroundColor(theme.tab_bar_bg.to_crossterm()))?;
        if x < self.width {
            let remaining = " ".repeat((self.width - x) as usize);
            queue!(stdout, Print(&remaining))?;
        }

        queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;
        queue!(stdout, SetForegroundColor(theme.foreground.to_crossterm()))?;

        Ok(())
    }

    fn render_editor_pane(
        &self,
        stdout: &mut impl Write,
        pane: &crate::editor::Pane,
        rect: &Rect,
        theme: &Theme,
    ) -> io::Result<()> {
        let line_count = pane.buffer.line_count();
        let gutter_width = 4;
        let text_width = rect.width.saturating_sub(gutter_width) as usize;

        queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;

        for row in 0..rect.height {
            let line_idx = row as usize + pane.scroll_offset;
            queue!(stdout, MoveTo(rect.x, rect.y + row))?;

            if line_idx < line_count {
                let is_cursor_line = line_idx == pane.cursor.line;

                // Line number
                let line_num = if is_cursor_line {
                    line_idx + 1
                } else {
                    (line_idx as isize - pane.cursor.line as isize).unsigned_abs()
                };

                let line_num_color = if is_cursor_line {
                    theme.line_number_active
                } else {
                    theme.line_number
                };

                queue!(stdout, SetForegroundColor(line_num_color.to_crossterm()))?;
                queue!(stdout, Print(format!("{:>3} ", line_num)))?;

                // Line content
                let line = pane.buffer.line(line_idx);
                let line_str: String = line.chars().take(text_width).collect();
                let content = line_str.trim_end_matches('\n').to_string();

                let padded: String = format!("{:width$}", content, width = text_width)
                    .chars()
                    .take(text_width)
                    .collect();

                // Don't dim unfocused panes - keep same style
                queue!(stdout, SetForegroundColor(theme.foreground.to_crossterm()))?;
                queue!(stdout, Print(&padded))?;
            } else {
                // Empty line indicator
                queue!(stdout, SetForegroundColor(theme.line_number.to_crossterm()))?;
                queue!(stdout, Print("  ~ "))?;

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
        theme: &Theme,
    ) -> io::Result<()> {
        queue!(
            stdout,
            SetBackgroundColor(theme.file_browser_bg.to_crossterm())
        )?;

        // Title row
        queue!(stdout, MoveTo(rect.x, rect.y))?;
        queue!(stdout, SetForegroundColor(theme.foreground.to_crossterm()))?;
        queue!(stdout, SetAttribute(Attribute::Bold))?;
        let title = " Files ";
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
            queue!(
                stdout,
                SetBackgroundColor(theme.file_browser_bg.to_crossterm())
            )?;

            if let Some(entry) = file_browser.entries.get(idx) {
                let is_selected = idx == file_browser.selected;

                if is_selected && is_focused {
                    queue!(
                        stdout,
                        SetBackgroundColor(theme.file_browser_selected.to_crossterm())
                    )?;
                    queue!(stdout, SetForegroundColor(theme.background.to_crossterm()))?;
                } else if entry.is_dir {
                    queue!(
                        stdout,
                        SetForegroundColor(theme.file_browser_dir.to_crossterm())
                    )?;
                } else {
                    let color = if is_focused {
                        theme.file_browser_file
                    } else {
                        theme.line_number
                    };
                    queue!(stdout, SetForegroundColor(color.to_crossterm()))?;
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

                queue!(stdout, Print(&padded))?;
            } else {
                let blank = " ".repeat(rect.width as usize);
                queue!(stdout, Print(&blank))?;
            }
        }

        queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;
        Ok(())
    }

    fn render_pane_borders(
        &self,
        stdout: &mut impl Write,
        workspace: &Workspace,
        pane_rects: &[(usize, Rect)],
        theme: &Theme,
    ) -> io::Result<()> {
        // Simple approach: draw separators without trying to connect them
        // Active pane gets rounded corners at its border junctions

        queue!(stdout, SetForegroundColor(theme.pane_border.to_crossterm()))?;
        queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;

        // Draw all separators in inactive color
        for (pane_id, rect) in pane_rects {
            let sep_x = rect.x + rect.width;
            let sep_y = rect.y + rect.height;

            // Check for right neighbor - draw vertical separator
            let has_right = pane_rects
                .iter()
                .any(|(id, r)| *id != *pane_id && r.x == sep_x + 1);
            if has_right {
                for y in rect.y..(rect.y + rect.height) {
                    queue!(stdout, MoveTo(sep_x, y))?;
                    queue!(stdout, Print("│"))?;
                }
            }

            // Check for bottom neighbor - draw horizontal separator
            let has_bottom = pane_rects
                .iter()
                .any(|(id, r)| *id != *pane_id && r.y == sep_y + 1);
            if has_bottom {
                for x in rect.x..(rect.x + rect.width) {
                    queue!(stdout, MoveTo(x, sep_y))?;
                    queue!(stdout, Print("─"))?;
                }
            }
        }

        // Second pass: redraw active pane's adjacent separators with rounded corners
        for (pane_id, rect) in pane_rects {
            if !workspace.is_focused(*pane_id) {
                continue;
            }

            let sep_x = rect.x + rect.width;
            let sep_y = rect.y + rect.height;

            let has_right = pane_rects
                .iter()
                .any(|(id, r)| *id != *pane_id && r.x == sep_x + 1);
            let has_bottom = pane_rects
                .iter()
                .any(|(id, r)| *id != *pane_id && r.y == sep_y + 1);
            let has_left = pane_rects
                .iter()
                .any(|(id, r)| *id != *pane_id && r.x + r.width + 1 == rect.x);
            let has_top = pane_rects
                .iter()
                .any(|(id, r)| *id != *pane_id && r.y + r.height + 1 == rect.y);

            queue!(
                stdout,
                SetForegroundColor(theme.pane_border_active.to_crossterm())
            )?;

            // Draw left separator
            if has_left && rect.x > 0 {
                let left_x = rect.x - 1;
                for y in rect.y..(rect.y + rect.height) {
                    queue!(stdout, MoveTo(left_x, y))?;
                    queue!(stdout, Print("│"))?;
                }
            }

            // Draw right separator
            if has_right {
                for y in rect.y..(rect.y + rect.height) {
                    queue!(stdout, MoveTo(sep_x, y))?;
                    queue!(stdout, Print("│"))?;
                }
            }

            // Draw top separator
            if has_top && rect.y > 0 {
                let top_y = rect.y - 1;
                for x in rect.x..(rect.x + rect.width) {
                    queue!(stdout, MoveTo(x, top_y))?;
                    queue!(stdout, Print("─"))?;
                }
            }

            // Draw bottom separator
            if has_bottom {
                for x in rect.x..(rect.x + rect.width) {
                    queue!(stdout, MoveTo(x, sep_y))?;
                    queue!(stdout, Print("─"))?;
                }
            }

            // Draw rounded corners where borders meet
            // Top-left corner
            if has_left && has_top && rect.x > 0 && rect.y > 0 {
                queue!(stdout, MoveTo(rect.x - 1, rect.y - 1))?;
                queue!(stdout, Print("╭"))?;
            }

            // Top-right corner
            if has_right && has_top && rect.y > 0 {
                queue!(stdout, MoveTo(sep_x, rect.y - 1))?;
                queue!(stdout, Print("╮"))?;
            }

            // Bottom-left corner
            if has_left && has_bottom && rect.x > 0 {
                queue!(stdout, MoveTo(rect.x - 1, sep_y))?;
                queue!(stdout, Print("╰"))?;
            }

            // Bottom-right corner
            if has_right && has_bottom {
                queue!(stdout, MoveTo(sep_x, sep_y))?;
                queue!(stdout, Print("╯"))?;
            }

            break;
        }

        Ok(())
    }

    fn render_pane_labels(
        &self,
        stdout: &mut impl Write,
        workspace: &Workspace,
        pane_rects: &[(usize, Rect)],
        theme: &Theme,
    ) -> io::Result<()> {
        let labeled_panes = workspace.get_editor_panes_with_labels();
        for (label, pane_id) in labeled_panes {
            if let Some((_, rect)) = pane_rects.iter().find(|(id, _)| *id == pane_id) {
                let center_x = rect.x + rect.width / 2;
                let center_y = rect.y + rect.height / 2;

                queue!(stdout, MoveTo(center_x.saturating_sub(2), center_y))?;
                queue!(stdout, SetForegroundColor(theme.background.to_crossterm()))?;
                queue!(stdout, SetBackgroundColor(theme.cursor.to_crossterm()))?;
                queue!(stdout, SetAttribute(Attribute::Bold))?;
                queue!(stdout, Print(format!(" {} ", label.to_ascii_uppercase())))?;
                queue!(stdout, SetAttribute(Attribute::Reset))?;
                queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;
            }
        }
        Ok(())
    }

    fn render_status_line(
        &self,
        stdout: &mut impl Write,
        workspace: &Workspace,
        theme: &Theme,
    ) -> io::Result<()> {
        let status_row = self.height.saturating_sub(1);
        queue!(stdout, MoveTo(0, status_row))?;

        // Command mode - just show the command
        if workspace.mode() == Mode::Command {
            queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;
            queue!(stdout, SetForegroundColor(theme.foreground.to_crossterm()))?;
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            queue!(stdout, Print(format!(":{}", workspace.command_buffer)))?;
            return Ok(());
        }

        // Message - show prominently
        if let Some(ref msg) = workspace.message {
            queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;
            queue!(stdout, SetForegroundColor(theme.warning.to_crossterm()))?;
            queue!(stdout, Clear(ClearType::CurrentLine))?;
            queue!(stdout, Print(msg))?;
            return Ok(());
        }

        // Normal status bar
        queue!(
            stdout,
            SetBackgroundColor(theme.status_bar_bg.to_crossterm())
        )?;
        queue!(
            stdout,
            SetForegroundColor(theme.status_bar_fg.to_crossterm())
        )?;

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
        queue!(stdout, SetBackgroundColor(theme.background.to_crossterm()))?;

        Ok(())
    }

    fn position_cursor(
        &self,
        stdout: &mut impl Write,
        workspace: &Workspace,
        pane_rects: &[(usize, Rect)],
        _theme: &Theme,
    ) -> io::Result<()> {
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
        Ok(())
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new().expect("Failed to create renderer")
    }
}
