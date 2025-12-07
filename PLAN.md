# Lark – Rust Modal Terminal Editor

## Project Goals

* **Modal editing:** Insert, Normal, Visual (later) modes
* **Lightweight:** Fast startup, low memory footprint
* **Extensible:** Plugins and scripts via Rhai without recompiling
* **Adaptable:** Dynamic configuration, filetype-specific behaviors, runtime hot reload
* **Programming-focused:** Efficient navigation, editing, and extensibility for coding workflows
* **Single-language config:** All configuration, keymaps, and plugin logic in Rhai
* **LSP built-in:** First-class language server support in core
* **Tree-sitter powered:** Syntax highlighting and code navigation via tree-sitter

---

## Architecture

### Threading Model

```
┌─────────────────────────────────────────────────────────┐
│  Main Thread (tokio async runtime)                      │
│  ├── Event loop (crossterm EventStream)                 │
│  ├── Rendering                                          │
│  ├── Rhai script execution (Rhai engines not Sync)      │
│  └── Buffer mutations                                   │
└───────────────────────┬─────────────────────────────────┘
                        │ tokio::mpsc channels
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
   ┌─────────┐    ┌──────────┐    ┌──────────┐
   │ LSP     │    │ File I/O │    │ File     │
   │ Client  │    │ (async)  │    │ Watcher  │
   └─────────┘    └──────────┘    └──────────┘
        (spawned as tokio tasks)
```

**Key principles:**
- Rhai executes on main thread (not thread-safe)
- LSP, file ops, config watchers run as background tokio tasks
- Communication via async channels (`tokio::mpsc`)
- `tokio::select!` multiplexes all event sources in main loop

---

### Core Architecture

#### Editor Core

Responsibilities:
* Buffer management (ropey)
* Cursor tracking with viewport
* Modal input handling
* File I/O
* Event hooks for plugins and scripting

Core structs:

```rust
struct Editor {
    buffer: Buffer,
    cursor: Cursor,
    mode: Mode,
    viewport: Viewport,
    event_bus: EventBus,
    plugins: PluginManager,
}

struct Cursor { line: usize, col: usize }
enum Mode { Normal, Insert, Command, Visual }

struct KeyMap { 
    normal: HashMap<KeySeq, Action>, 
    insert: HashMap<KeySeq, Action>,
}
```

#### Input & Key Handling

* Async key events via `crossterm::event::EventStream`
* Map keys to actions based on current mode
* Support multi-key sequences: `dd`, `gg`, `yy`, `ciw`
* Keymaps defined in Rhai config

Example Rhai config:

```rhai
map("normal", "h", move_left);
map("normal", "l", move_right);
map("normal", "i", enter_insert_mode);
map("insert", "<Esc>", enter_normal_mode);
```

#### Rendering

* Double-buffered terminal redraw to prevent flicker
* Render only changed lines for efficiency
* Status line: mode, filename, cursor position, diagnostics
* Line numbers, syntax highlighting (tree-sitter)

#### Extensibility (Rhai Plugins)

* Single-language scripting and config: `~/.config/lark/config.rhai`
* Expose Rust API to Rhai:

```rust
engine.register_fn("set_option", |key: &str, value: Dynamic| { ... });
engine.register_fn("map", |mode: &str, key: &str, action: FnPtr| { ... });
engine.register_fn("on_file_open", |callback: FnPtr| { ... });
engine.register_fn("load_plugin", |name: &str| { ... });
```

* Hot reload scripts at runtime via file watcher
* Async plugin execution (future): many small Rhai engines pattern

---

## Technology & Dependencies

| Component           | Crate                      | Purpose                                        |
| ------------------- | -------------------------- | ---------------------------------------------- |
| Terminal I/O        | `crossterm`                | Cross-platform terminal, raw mode, key events  |
| Text Buffer         | `ropey`                    | Efficient rope-based text buffer               |
| Async Runtime       | `tokio`                    | Async event loop, background tasks, channels   |
| Scripting           | `rhai`                     | Config, plugins, keymaps, hooks                |
| Syntax Highlighting | `tree-sitter`              | Parsing, highlighting, text objects            |
| LSP                 | `tower-lsp` / `lsp-types`  | Language server protocol client                |
| File Watching       | `notify`                   | Hot-reload config and plugins                  |

---

## Phase Roadmap

### Phase 1 – Minimal Viable Editor (MVE)

Rust-only, no scripting. Establish core editing loop.

**Dependencies:**
```toml
[dependencies]
crossterm = "0.27"
ropey = "1.4"
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

**Deliverables:**
* Raw terminal I/O with crossterm (async EventStream)
* Single buffer with ropey (load/edit/save)
* Cursor tracking with viewport scrolling
* Normal mode: `h/j/k/l`, `x`, `dd`, `w/b`
* Insert mode: character input, backspace, `Esc`
* Command mode: `:w`, `:q`, `:wq`
* Status line (mode, filename, line:col)

**Project Structure:**
```
src/
  main.rs           # Async entry point, event loop
  editor/
    mod.rs
    buffer.rs       # Ropey wrapper, file I/O
    cursor.rs       # Cursor position, movement
    mode.rs         # Mode enum
    editor.rs       # Editor state
  input/
    mod.rs          # Key handling, action dispatch
  render/
    mod.rs          # Terminal rendering
```

---

### Phase 2 – Rhai Config & Scripting

**New dependencies:** `rhai`, `notify`

**Deliverables:**
* Load `~/.config/lark/config.rhai` at startup
* Expose Rust API to Rhai scripts
* Keymaps, options, hooks defined in Rhai
* Hot-reload config on file change
* Plugin manager (one Rhai engine per plugin)
* Event bus for plugin hooks

---

### Phase 3 – Tree-sitter, LSP, Themes

**New dependencies:** `tree-sitter`, language grammars, `tower-lsp`/`lsp-types`

**Deliverables:**
* Tree-sitter integration for syntax highlighting
* Tree-sitter text objects (function, class, block, etc.)
* LSP client (built into core):
  - Diagnostics (inline errors/warnings)
  - Go to definition
  - Completions
  - Hover information
* Filetype detection
* Theme system (colors configurable in Rhai)

---

### Phase 4 – Advanced Features

* Multi-buffer / split windows
* Visual mode (line, block selection)
* Full undo/redo tree
* Macro recording and playback
* Async plugin execution (many small Rhai engines)
* Community plugin ecosystem

---

## Key Principles

* **Async-first:** tokio runtime from day 1, enables LSP and plugin concurrency
* **Single-language config:** Rhai for keymaps, options, macros, hooks, plugins
* **LSP as core feature:** Not a plugin, built-in language server support
* **Tree-sitter for parsing:** Syntax highlighting, text objects, code navigation
* **Start small:** Terminal + buffer + modal editing first
* **Event-driven:** Expose hooks early for plugin integration
* **Rust handles concurrency:** Rhai scripts stay simple, async lives in Rust
