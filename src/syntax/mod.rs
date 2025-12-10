//! Syntax highlighting module using Tree-sitter
//!
//! Provides syntax highlighting for supported languages using Tree-sitter grammars.
//! Grammars are installed on-demand to `~/.config/lark/grammars/`.

mod highlighter;
mod installer;
mod languages;
mod metadata;

#[allow(unused_imports)] // Will be used when rendering integrates highlighting
pub use highlighter::{Highlight, HighlightKind, HighlightedLine, Highlighter};
pub use installer::{GrammarInstaller, InstallResult};
pub use languages::{Language, LanguageRegistry};
#[allow(unused_imports)]
// GrammarMetadata used internally, TREE_SITTER_ABI_VERSION for :TSStatus
pub use metadata::{GrammarMetadata, TREE_SITTER_ABI_VERSION};
