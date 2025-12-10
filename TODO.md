# Lark Editor - TODO

## Phase 2: Configuration & Scripting

### Rhai Scripting Engine
- [x] Add `rhai` crate dependency
- [x] Create scripting module (`src/scripting/`) - central runtime
- [x] Load `~/.config/lark/init.rhai` on startup
- [x] Namespaced API under `lark::*` (uses Rhai's static module system):
  - [x] `lark::config::*` - settings, themes, keybinds (implemented)
  - [ ] `lark::editor::*` - buffer operations, cursor, mode, commands
  - [ ] `lark::ui::*` - popups, floating windows, status messages, prompts
  - [ ] `lark::fs::*` - file/directory operations
  - [ ] `lark::process::*` - spawn commands, shell integration
  - [ ] `lark::events::*` - subscribe to editor events (on_save, on_open, etc.)
  - [ ] `lark::lsp::*` - interact with the lsp module, add new supported languages
  - [ ] `lark::plugins::*` - plugin manager to add, lazy-load, and configure plugins
- [ ] Module imports (`import "modules/my_plugin" as plugin;`)
- [ ] Apply custom keybinds from config
- [ ] Hot-reload config on file change
- [x] `:source` command to reload config

---

## Phase 3: Syntax Highlighting

### Filetype System (TODO: Refactor)
- [ ] Create a proper `Filetype` registry system
- [ ] Filetypes defined in config files, not hardcoded in Rust
- [ ] Each filetype specifies: extensions, grammar repo, LSP server, comment syntax, indent rules
- [ ] Allow plugins/Rhai to register new filetypes dynamically
- [ ] Use filetype everywhere: syntax, LSP, formatting, commenting, etc.
- [ ] Similar to Neovim's filetype.lua / vim.filetype.add()

### Tree-sitter Integration
- [x] Add `tree-sitter` core crate (grammars installed separately)
- [x] Create syntax module (`src/syntax/`)
- [x] Highlighter with node type to highlight kind mapping
- [x] Integrate highlighter with renderer (apply colors)
- [x] Parse buffers with tree-sitter on file load
- [ ] Incremental parsing on edits
- [ ] Support languages: Rust, Python, JavaScript/TypeScript, Go, C/C++, Markdown, JSON, TOML, Bash, Lua, Ruby, HTML, CSS, YAML

### Tree-sitter Grammar Manager
- [x] Grammar storage in `~/.config/lark/grammars/`
- [x] `:TSInstall <language>` - download and compile grammar
- [x] `:TSUninstall <language>` - remove grammar
- [x] `:TSList` - list available/installed grammars
- [x] Compile grammars from source (uses system cc)
- [x] `:TSUpdate` - reinstall all outdated grammars
- [x] `:TSStatus` - show ABI version and outdated grammars
- [x] ABI version tracking (metadata.json)
- [x] Auto-reinstall when ABI version changes
- [ ] Auto-install grammars on first use (optional)
- [ ] Allow arbitrary grammars via config (not just hardcoded list)

---

## Phase 4: Language Server Protocol

### LSP Client
- [ ] Add `lsp-types` and `tower-lsp` crates
- [ ] Create LSP module (`src/lsp/`)
- [ ] Auto-detect and spawn language servers
- [ ] Implement core LSP features:
  - [ ] Diagnostics (errors, warnings inline)
  - [ ] Go to definition (`gd`)
  - [ ] Find references
  - [ ] Hover information
  - [ ] Autocomplete popup
  - [ ] Code actions
  - [ ] Rename symbol
- [ ] Status bar indicator for LSP state

---

## Phase 5: Plugin System

### Plugin Architecture
- [ ] Design plugin API surface
- [ ] Plugin manifest format (`plugin.toml`)
- [ ] Plugin loading from `~/.config/lark/plugins/`
- [ ] Plugin capabilities:
  - [ ] Create/modify themes
  - [ ] Register custom commands
  - [ ] Create popup windows/floating UI
  - [ ] Read/write buffers
  - [ ] Insert/delete text
  - [ ] Subscribe to editor events (save, open, cursor move, etc.)
  - [ ] Access file system
  - [ ] Make HTTP requests (async)
  - [ ] Spawn child processes
- [ ] Plugin sandboxing and permissions
- [ ] `:plugin` command to list/enable/disable plugins

### Plugin Manager
- [ ] Plugin registry/repository (like vim-plug, lazy.nvim)
- [ ] `:PluginInstall` - install plugins from git/URL
- [ ] `:PluginUpdate` - update installed plugins
- [ ] `:PluginRemove` - remove plugins
- [ ] `:PluginList` - list installed plugins with status
- [ ] Lazy loading support (load on command, filetype, event)
- [ ] Plugin dependencies resolution
- [ ] Lock file for reproducible installs

---

## Phase 6: AI & Agent Integration

### MCP (Model Context Protocol) Support
- [ ] Implement MCP client
- [ ] Connect to MCP servers (filesystem, git, etc.)
- [ ] Expose editor context to MCP

### LLM/Agent Integration
- [ ] Agent mode with tool use
- [ ] Inline code completion suggestions
- [ ] Chat panel for conversations
- [ ] Code explanation/documentation
- [ ] Refactoring suggestions
- [ ] Natural language commands
- [ ] Context-aware assistance (current file, project, errors)

---

## Phase 7: Debugging

### Debugger Support (DAP)
- [ ] Add Debug Adapter Protocol client
- [ ] Breakpoint management:
  - [ ] Set/toggle breakpoints (`<leader>db`)
  - [ ] Conditional breakpoints
  - [ ] Breakpoint gutter indicators
- [ ] Debug session controls:
  - [ ] Start/stop debugging
  - [ ] Step over/into/out
  - [ ] Continue/pause
- [ ] Debug UI:
  - [ ] Variables panel
  - [ ] Call stack panel
  - [ ] Watch expressions
  - [ ] Debug console/REPL
- [ ] Inline value display
- [ ] Support debuggers: lldb, gdb, delve, node-debug

---

## Backlog / Nice to Have

### Editor Enhancements
- [ ] Multiple cursors
- [ ] Macros (record/playback)
- [ ] Marks and jumps
- [ ] Registers (vim-style)
- [ ] Visual mode (character, line, block)
- [ ] Search and replace (with regex)
- [ ] Undo tree visualization
- [ ] Session persistence (restore tabs/panes)

### UI Improvements
- [ ] Built-in fuzzy finder (no fzf dependency)
- [ ] Command palette (`Ctrl+Shift+P`)
- [ ] Minimap
- [ ] Breadcrumbs
- [ ] Git gutter (added/modified/deleted lines)
- [ ] Image preview (for image files)

### Performance
- [ ] Lazy loading for large files
- [ ] Background file indexing
- [ ] Async file operations

---

## Completed ✓

- [x] Basic modal editing (Normal, Insert, Command modes)
- [x] File browser with tree navigation
- [x] Multiple panes with splits (vertical/horizontal)
- [x] Tabs
- [x] Directional pane navigation (Ctrl-W + hjkl)
- [x] Pane borders with active highlighting
- [x] Relative line numbers
- [x] Theme system with multiple built-in themes
- [x] Theme switching (`:theme` command)
- [x] Count prefix for commands (e.g., `10j`)
- [x] Leader key sequences (`<space>`)
- [x] File finder (fzf integration)
- [x] Grep (ripgrep integration)
- [x] Page up/down (Ctrl+U/D)
- [x] Viewport scrolling
- [x] Status line

