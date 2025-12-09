use std::io::{self, Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
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

    pub fn text_height(&self) -> usize {
        self.height.saturating_sub(1) as usize
    }

    pub fn render(&self, workspace: &Workspace) -> io::Result<()> {
        let mut stdout = stdout();

        // Hide cursor during redraw to prevent flicker
        queue!(stdout, Hide)?;

        // Calculate layout - reserve 1 line for global status
        let content_area = Rect::new(0, 0, self.width, self.height.saturating_sub(1));
        let pane_rects = workspace.calculate_rects(content_area);

        // Render each pane
        for (pane_id, rect) in &pane_rects {
            if let Some(pane) = workspace.pane(*pane_id) {
                let is_focused = workspace.is_focused(*pane_id);
                match pane.kind {
                    PaneKind::Editor => {
                        self.render_editor_pane(&mut stdout, workspace, pane, rect, is_focused)?
                    }
                    PaneKind::FileBrowser => {
                        self.render_file_browser_pane(&mut stdout, workspace, rect, is_focused)?
                    }
                }
            }
        }

        // Render global status line
        self.render_status_line(&mut stdout, workspace)?;

        // Position cursor in focused pane
        if let Some(pane) = workspace.pane(workspace.focused_pane().id) {
            let rect = pane_rects
                .iter()
                .find(|(id, _)| *id == pane.id)
                .map(|(_, r)| r);

            if let Some(rect) = rect {
                if workspace.mode() == Mode::Command {
                    let cmd_col = 1 + workspace.command_buffer.len() as u16;
                    let cmd_row = self.height.saturating_sub(1);
                    queue!(stdout, MoveTo(cmd_col, cmd_row))?;
                    queue!(stdout, SetCursorStyle::BlinkingBar)?;
                    queue!(stdout, Show)?;
                } else if pane.kind == PaneKind::Editor {
                    let gutter_width = 4u16;
                    let cursor_x = rect.x + gutter_width + pane.cursor.col as u16;
                    let cursor_y = rect.y + (pane.cursor.line - pane.scroll_offset) as u16;
                    queue!(stdout, MoveTo(cursor_x, cursor_y))?;

                    let cursor_style = match pane.mode {
                        Mode::Insert => SetCursorStyle::BlinkingBar,
                        _ => SetCursorStyle::SteadyBlock,
                    };
                    queue!(stdout, cursor_style)?;
                    queue!(stdout, Show)?;
                } else {
                    queue!(stdout, Hide)?;
                }
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn render_editor_pane(
        &self,
        stdout: &mut impl Write,
        _workspace: &Workspace,
        pane: &crate::editor::Pane,
        rect: &Rect,
        is_focused: bool,
    ) -> io::Result<()> {
        let line_count = pane.buffer.line_count();
        let gutter_width = 4; // Width for line numbers
        let text_width = rect.width.saturating_sub(gutter_width) as usize;

        for row in 0..rect.height {
            let line_idx = row as usize + pane.scroll_offset;
            queue!(stdout, MoveTo(rect.x, rect.y + row))?;

            // Render line number (relative or absolute)
            if line_idx < line_count {
                let is_cursor_line = line_idx == pane.cursor.line;

                // Line number: absolute for cursor line, relative for others
                let line_num = if is_cursor_line {
                    line_idx + 1 // 1-indexed absolute
                } else {
                    let diff = (line_idx as isize - pane.cursor.line as isize).unsigned_abs();
                    diff
                };

                // Color: bright for cursor line, dim for others
                if is_cursor_line {
                    queue!(stdout, SetForegroundColor(Color::Yellow))?;
                } else {
                    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
                }

                // Right-align the number in the gutter
                queue!(stdout, Print(format!("{:>3} ", line_num)))?;
                queue!(stdout, SetForegroundColor(Color::Reset))?;

                // Render line content
                let line = pane.buffer.line(line_idx);
                let line_str: String = line.chars().take(text_width).collect();
                let content = line_str.trim_end_matches('\n').to_string();

                // Pad to fill the remaining width
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
                // Empty line (beyond buffer)
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

        // File list
        for row in 1..rect.height {
            let idx = row as usize - 1;
            queue!(stdout, MoveTo(rect.x, rect.y + row))?;

            if let Some(entry) = workspace.file_browser.entries.get(idx) {
                let is_selected = idx == workspace.file_browser.selected && is_focused;

                if is_selected {
                    queue!(stdout, SetAttribute(Attribute::Reverse))?;
                }

                let icon = if entry.is_dir { "▸ " } else { "  " };
                let name: String = entry.name.chars().take(rect.width as usize - 2).collect();
                let display = format!("{}{}", icon, name);
                let padded: String = format!("{:width$}", display, width = rect.width as usize)
                    .chars()
                    .take(rect.width as usize)
                    .collect();

                if !is_focused && !is_selected {
                    queue!(stdout, SetForegroundColor(Color::DarkGrey))?;
                }
                queue!(stdout, Print(&padded))?;
                if !is_focused && !is_selected {
                    queue!(stdout, SetForegroundColor(Color::Reset))?;
                }

                if is_selected {
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

        // In command mode, show command input
        if workspace.mode() == Mode::Command {
            queue!(stdout, Print(format!(":{}", workspace.command_buffer)))?;
            return Ok(());
        }

        // If there's a message, show it
        if let Some(ref msg) = workspace.message {
            queue!(stdout, Print(msg))?;
            return Ok(());
        }

        // Normal status line
        queue!(stdout, SetAttribute(Attribute::Reverse))?;

        let pane = workspace.focused_pane();
        let mode = pane.mode.display();
        let filename = "[No Name]"; // TODO: get from buffer
        let position = format!("{}:{}", pane.cursor.line + 1, pane.cursor.col + 1);

        // Show pending keys if any
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
