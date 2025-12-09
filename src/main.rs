use std::env;
use std::path::PathBuf;

use crossterm::event::EventStream;
use futures::StreamExt;

mod editor;
mod input;
mod render;
mod theme;

use editor::Workspace;
use input::InputState;
use render::Renderer;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line args
    let args: Vec<String> = env::args().collect();
    let mut workspace = if args.len() > 1 {
        Workspace::open(PathBuf::from(&args[1]))
    } else {
        Workspace::new()
    };

    // Set up terminal
    Renderer::setup()?;
    let renderer = Renderer::new()?;

    // Input state for key sequences
    let mut input_state = InputState::new();

    // Initial render
    let current_theme = theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
    renderer.render(&workspace, &current_theme)?;

    // Event stream for async key reading
    let mut event_stream = EventStream::new();

    // Main loop
    while workspace.running {
        tokio::select! {
            Some(Ok(event)) = event_stream.next() => {
                input::handle_event(&mut workspace, event, &mut input_state);

                // Adjust scroll for focused pane
                let text_height = renderer.text_height(&workspace);
                workspace.focused_pane_mut().adjust_scroll(text_height);

                // Get current theme (may have changed via :theme command)
                let current_theme = theme::get_builtin_theme(&workspace.theme_name).unwrap_or_default();
                renderer.render(&workspace, &current_theme)?;
            }
        }
    }

    // Cleanup
    Renderer::teardown()?;

    Ok(())
}
