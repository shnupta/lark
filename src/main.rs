use std::env;
use std::path::PathBuf;

use crossterm::event::EventStream;
use futures::StreamExt;

mod config;
mod editor;
mod finder;
mod input;
mod render;
mod scripting;
mod syntax;
mod theme;

use editor::{FinderAction, Workspace};
use finder::{FinderResult, GrepMatch};
use input::InputState;
use render::Renderer;
use scripting::ScriptEngine;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load configuration using the scripting engine
    let mut script_engine = ScriptEngine::new();
    let config_error = script_engine.load_default().err();
    let settings = script_engine.settings();

    // Parse command line args
    let args: Vec<String> = env::args().collect();
    let mut verbose = false;
    let mut file_path: Option<PathBuf> = None;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "--verbose" | "-v" => verbose = true,
            "--help" | "-h" => {
                println!("lark - a modal terminal editor");
                println!();
                println!("Usage: lark [OPTIONS] [FILE]");
                println!();
                println!("Options:");
                println!("  -v, --verbose    Enable verbose logging");
                println!("  -h, --help       Show this help");
                return Ok(());
            }
            _ => {
                if !arg.starts_with('-') {
                    file_path = Some(PathBuf::from(arg));
                }
            }
        }
    }

    let mut workspace = if let Some(path) = file_path {
        Workspace::open(path)
    } else {
        Workspace::new()
    };

    workspace.verbose = verbose;

    // Apply settings from config
    workspace.theme_name = settings.theme.clone();

    // Show config error if any
    if let Some(err) = config_error {
        workspace.set_error(err);
    }

    // Log startup info
    if verbose {
        workspace.log("Lark started in verbose mode");
        workspace.log(format!("Theme: {}", workspace.theme_name));
    }

    // Set up terminal
    Renderer::setup()?;
    let renderer = Renderer::new()?;

    // Input state for key sequences
    let mut input_state = InputState::new();

    // Initial render
    let current_theme = theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
    renderer.render(&mut workspace, &current_theme)?;

    // Event stream for async key reading
    let mut event_stream = EventStream::new();

    // Main loop
    while workspace.running {
        // Check for pending finder actions (need to run outside of raw mode)
        if let Some(finder_action) = workspace.pending_finder.take() {
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            // Teardown terminal for fzf
            Renderer::teardown()?;

            let result = match finder_action {
                FinderAction::FindFile => {
                    match finder::find_file(&cwd) {
                        FinderResult::Selected(path) => Some((path, None)),
                        FinderResult::Cancelled => None,
                        FinderResult::Error(e) => {
                            // Re-setup terminal first, then show error
                            Renderer::setup()?;
                            workspace.set_message(e);
                            let current_theme =
                                theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
                            renderer.render(&mut workspace, &current_theme)?;
                            continue;
                        }
                    }
                }
                FinderAction::Grep(pattern) => {
                    // If no pattern, use word under cursor
                    let search_pattern = if pattern.is_empty() {
                        get_word_under_cursor(&workspace)
                    } else {
                        pattern
                    };

                    if search_pattern.is_empty() {
                        Renderer::setup()?;
                        workspace.set_message("No pattern to search".to_string());
                        let current_theme =
                            theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
                        renderer.render(&mut workspace, &current_theme)?;
                        continue;
                    }

                    match finder::grep_files(&search_pattern, &cwd) {
                        finder::grep::GrepResult::Selected(grep_match) => {
                            let file = grep_match.file.clone();
                            Some((file, Some(grep_match)))
                        }
                        finder::grep::GrepResult::Cancelled => None,
                        finder::grep::GrepResult::NoMatches => {
                            Renderer::setup()?;
                            workspace.set_message(format!("No matches for: {}", search_pattern));
                            let current_theme =
                                theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
                            renderer.render(&mut workspace, &current_theme)?;
                            continue;
                        }
                        finder::grep::GrepResult::Error(e) => {
                            Renderer::setup()?;
                            workspace.set_message(e);
                            let current_theme =
                                theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
                            renderer.render(&mut workspace, &current_theme)?;
                            continue;
                        }
                    }
                }
            };

            // Re-setup terminal
            Renderer::setup()?;

            // Open the selected file
            if let Some((path, grep_match)) = result {
                workspace.open_file_in_focused_pane(path);

                // If grep match, jump to line/col
                if let Some(GrepMatch { line, col, .. }) = grep_match {
                    let pane = workspace.focused_pane_mut();
                    pane.cursor.line = line.saturating_sub(1);
                    pane.cursor.col = col.saturating_sub(1);
                }
            }

            let current_theme = theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
            renderer.render(&mut workspace, &current_theme)?;
            continue;
        }

        tokio::select! {
            Some(Ok(event)) = event_stream.next() => {
                input::handle_event(&mut workspace, event, &mut input_state);

                // Adjust scroll for focused pane based on its actual height
                let pane_height = renderer.focused_pane_height(&workspace);
                workspace.focused_pane_mut().adjust_scroll(pane_height);

                // Get current theme (may have changed via :theme command)
                let current_theme = theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
                renderer.render(&mut workspace, &current_theme)?;
            }
        }
    }

    // Cleanup
    Renderer::teardown()?;

    Ok(())
}

fn get_word_under_cursor(workspace: &Workspace) -> String {
    let pane = workspace.focused_pane();
    let line_text = pane.buffer.line(pane.cursor.line);
    let col = pane.cursor.col;

    let chars: Vec<char> = line_text.chars().collect();

    if chars.is_empty() || col >= chars.len() {
        return String::new();
    }

    // Find word boundaries
    let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

    if !is_word_char(chars[col]) {
        return String::new();
    }

    let mut start = col;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }

    let mut end = col;
    while end < chars.len() && is_word_char(chars[end]) {
        end += 1;
    }

    chars[start..end].iter().collect()
}
