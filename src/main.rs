use std::env;
use std::path::PathBuf;

use crossterm::event::EventStream;
use futures::StreamExt;

mod editor;
mod input;
mod render;

use editor::Editor;
use render::Renderer;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Parse command line args
    let args: Vec<String> = env::args().collect();
    let mut editor = if args.len() > 1 {
        Editor::open(PathBuf::from(&args[1]))
    } else {
        Editor::new()
    };

    // Set up terminal
    Renderer::setup()?;
    let renderer = Renderer::new()?;

    // Initial render
    editor.adjust_scroll(renderer.text_height());
    renderer.render(&editor)?;

    // Event stream for async key reading
    let mut event_stream = EventStream::new();

    // Main loop
    while editor.running {
        tokio::select! {
            Some(Ok(event)) = event_stream.next() => {
                input::handle_event(&mut editor, event);
                editor.adjust_scroll(renderer.text_height());
                renderer.render(&editor)?;
            }
        }
    }

    // Cleanup
    Renderer::teardown()?;

    Ok(())
}
