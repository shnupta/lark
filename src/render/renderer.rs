use std::io::{self, Write, stdout};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    execute, queue,
    style::{Attribute, Print, SetAttribute},
    terminal::{
        self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

use crate::editor::{Editor, Mode};

pub struct Renderer {
    width: u16,
    height: u16,
}

impl Renderer {
    pub fn new() -> io::Result<Self> {
        let (width, height) = terminal::size()?;
        Ok(Self { width, height })
    }

    /// Set up terminal for the editor (raw mode, alternate screen, etc.)
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

    /// Restore terminal to normal state
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

    /// Render the editor to the terminal
    pub fn render(&self, editor: &Editor) -> io::Result<()> {
        let mut stdout = stdout();

        // Calculate viewport
        let text_height = self.height.saturating_sub(1); // Reserve 1 line for status

        // Clear and draw buffer contents
        for row in 0..text_height {
            let line_idx = row as usize; // TODO: add scroll offset
            queue!(stdout, MoveTo(0, row))?;
            queue!(stdout, Clear(ClearType::CurrentLine))?;

            if line_idx < editor.buffer.line_count() {
                let line = editor.buffer.line(line_idx);
                let line_str: String = line.chars().take(self.width as usize).collect();
                // Remove trailing newline for display
                let display = line_str.trim_end_matches('\n');
                queue!(stdout, Print(display))?;
            } else {
                // Empty line indicator (like vim's ~)
                queue!(stdout, Print("~"))?;
            }
        }

        // Draw status line (or command line in command mode)
        self.render_status_line(&mut stdout, editor)?;

        // Set cursor style based on mode
        let cursor_style = match editor.mode {
            Mode::Insert => SetCursorStyle::BlinkingBar,
            Mode::Normal => SetCursorStyle::SteadyBlock,
            Mode::Command => SetCursorStyle::BlinkingBar,
        };
        queue!(stdout, cursor_style)?;

        // Position cursor
        if editor.mode == Mode::Command {
            // Cursor at end of command input
            let cmd_col = 1 + editor.command_buffer.len() as u16; // after ":"
            let cmd_row = self.height.saturating_sub(1);
            queue!(stdout, MoveTo(cmd_col, cmd_row))?;
        } else {
            let cursor_x = editor.cursor.col as u16;
            let cursor_y = editor.cursor.line as u16; // TODO: subtract scroll offset
            queue!(stdout, MoveTo(cursor_x, cursor_y))?;
        }
        queue!(stdout, Show)?;

        stdout.flush()?;
        Ok(())
    }

    fn render_status_line(&self, stdout: &mut impl Write, editor: &Editor) -> io::Result<()> {
        let status_row = self.height.saturating_sub(1);
        queue!(stdout, MoveTo(0, status_row))?;
        queue!(stdout, Clear(ClearType::CurrentLine))?;

        // In command mode, show command input instead of status
        if editor.mode == Mode::Command {
            queue!(stdout, Print(format!(":{}", editor.command_buffer)))?;
            return Ok(());
        }

        // Invert colors for status line
        queue!(stdout, SetAttribute(Attribute::Reverse))?;

        // Build status line content
        let mode = editor.mode.display();
        let filename = "[No Name]"; // TODO: get from buffer
        let position = format!("{}:{}", editor.cursor.line + 1, editor.cursor.col + 1);

        // Left side: mode + filename
        let left = format!(" {} | {} ", mode, filename);

        // Right side: position
        let right = format!(" {} ", position);

        // Calculate padding
        let padding = self.width as usize - left.len() - right.len();
        let middle = " ".repeat(padding.max(0));

        let status = format!("{}{}{}", left, middle, right);
        // Truncate if too long
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
