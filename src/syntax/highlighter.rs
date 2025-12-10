//! Syntax highlighter using Tree-sitter

use std::path::Path;
use tree_sitter::{Parser, Tree};

use super::languages::{Language, LanguageRegistry};

/// A highlight span within a line
#[derive(Debug, Clone)]
pub struct Highlight {
    pub start: usize, // Column start (byte offset within line)
    pub end: usize,   // Column end (byte offset within line)
    pub kind: HighlightKind,
}

/// Types of syntax elements for highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightKind {
    Keyword,
    String,
    Number,
    Comment,
    Function,
    Type,
    Variable,
    Operator,
    Punctuation,
    Property,
    Constant,
    Namespace,
    Parameter,
    Label,
    Default,
}

impl HighlightKind {
    /// Map a Tree-sitter node type to a highlight kind
    pub fn from_node_type(node_type: &str, lang: Language) -> Self {
        // Try language-specific patterns first (they're more accurate)
        let specific = Self::from_language_specific(node_type, lang);
        if specific != HighlightKind::Default {
            return specific;
        }

        // Common patterns across languages
        match node_type {
            // Comments
            "comment" | "line_comment" | "block_comment" | "doc_comment" => HighlightKind::Comment,

            // Strings
            "string"
            | "string_literal"
            | "raw_string"
            | "raw_string_literal"
            | "char_literal"
            | "string_content"
            | "escape_sequence"
            | "interpreted_string_literal" => HighlightKind::String,

            // Numbers
            "number" | "integer" | "float" | "integer_literal" | "float_literal"
            | "number_literal" => HighlightKind::Number,

            // Keywords (generic)
            "keyword" | "storage_class" | "visibility_modifier" | "mutable_specifier" => {
                HighlightKind::Keyword
            }

            // Types
            "type"
            | "type_identifier"
            | "primitive_type"
            | "type_annotation"
            | "type_arguments"
            | "generic_type"
            | "class_definition"
            | "interface_declaration" => HighlightKind::Type,

            // Variables and identifiers (only if not matched by language-specific)
            "variable" | "shorthand_field_identifier" => HighlightKind::Variable,

            // Operators
            "operator" | "comparison_operator" | "assignment_operator" => HighlightKind::Operator,

            // Properties/fields
            "property" | "property_identifier" | "member_expression" => HighlightKind::Property,

            // Constants
            "true" | "false" | "null" | "none" | "nil" | "boolean" | "constant" => {
                HighlightKind::Constant
            }

            // Labels
            "label" | "loop_label" | "lifetime" => HighlightKind::Label,

            _ => HighlightKind::Default,
        }
    }

    fn from_language_specific(node_type: &str, lang: Language) -> Self {
        match lang {
            Language::Rust => Self::from_rust_node(node_type),
            Language::Python => Self::from_python_node(node_type),
            Language::JavaScript | Language::TypeScript | Language::Tsx => {
                Self::from_js_node(node_type)
            }
            Language::Go => Self::from_go_node(node_type),
            _ => HighlightKind::Default,
        }
    }

    fn from_rust_node(node_type: &str) -> Self {
        match node_type {
            // Keywords
            "let" | "fn" | "pub" | "mod" | "use" | "struct" | "enum" | "trait" | "impl" | "for"
            | "loop" | "while" | "if" | "else" | "match" | "return" | "break" | "continue"
            | "async" | "await" | "const" | "static" | "mut" | "ref" | "self" | "super"
            | "crate" | "where" | "as" | "in" | "dyn" | "move" | "type" | "unsafe" | "extern"
            | "default" | "union" | "become" | "box" | "do" | "final" | "macro" | "override"
            | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try" | "abstract" | "Self" => {
                HighlightKind::Keyword
            }

            // Punctuation and operators
            ";" | "," | "::" | ":" | "->" | "=>" | "=" | "+" | "-" | "*" | "/" | "%" | "&"
            | "|" | "^" | "!" | "<" | ">" | "?" | "@" | "#" | "." | ".." | "..." | "..=" | "+="
            | "-=" | "*=" | "/=" | "%=" | "&=" | "|=" | "^=" | "<<" | ">>" | "<<=" | ">>="
            | "==" | "!=" | "<=" | ">=" | "&&" | "||" => HighlightKind::Operator,

            // Brackets
            "(" | ")" | "[" | "]" | "{" | "}" => HighlightKind::Punctuation,

            // Types
            "type_identifier" | "primitive_type" | "scoped_type_identifier" => HighlightKind::Type,

            // Constants and literals
            "boolean_literal" | "integer_literal" | "float_literal" => HighlightKind::Number,
            "char_literal" => HighlightKind::String,

            // Macros (note: "!" is handled via parent context in determine_highlight_kind)
            "macro_invocation" | "macro_definition" | "macro_rules!" => HighlightKind::Function,

            // Identifiers in specific contexts
            "field_identifier" => HighlightKind::Property,
            "identifier" => HighlightKind::Variable,

            // Attributes
            "attribute_item" | "inner_attribute_item" | "attribute" => HighlightKind::Label,

            // Lifetime
            "lifetime" | "label" => HighlightKind::Label,

            // Strings
            "string_literal" | "raw_string_literal" | "string_content" | "escape_sequence" => {
                HighlightKind::String
            }

            // Comments
            "line_comment" | "block_comment" => HighlightKind::Comment,

            _ => HighlightKind::Default,
        }
    }

    fn from_python_node(node_type: &str) -> Self {
        match node_type {
            "def" | "class" | "if" | "elif" | "else" | "for" | "while" | "try" | "except"
            | "finally" | "with" | "as" | "import" | "from" | "return" | "yield" | "raise"
            | "pass" | "break" | "continue" | "lambda" | "and" | "or" | "not" | "in" | "is"
            | "global" | "nonlocal" | "assert" | "async" | "await" => HighlightKind::Keyword,
            "decorator" | "decorated_definition" => HighlightKind::Label,
            _ => HighlightKind::Default,
        }
    }

    fn from_js_node(node_type: &str) -> Self {
        match node_type {
            "function" | "const" | "let" | "var" | "if" | "else" | "for" | "while" | "do"
            | "switch" | "case" | "default" | "break" | "continue" | "return" | "throw" | "try"
            | "catch" | "finally" | "class" | "extends" | "new" | "this" | "super" | "import"
            | "export" | "from" | "async" | "await" | "yield" | "typeof" | "instanceof" | "in"
            | "of" | "delete" | "void" | "interface" | "type" | "enum" | "implements"
            | "public" | "private" | "protected" | "readonly" | "abstract" | "static" => {
                HighlightKind::Keyword
            }
            "jsx_element"
            | "jsx_opening_element"
            | "jsx_closing_element"
            | "jsx_self_closing_element" => HighlightKind::Type,
            _ => HighlightKind::Default,
        }
    }

    fn from_go_node(node_type: &str) -> Self {
        match node_type {
            "func" | "package" | "import" | "type" | "struct" | "interface" | "map" | "chan"
            | "if" | "else" | "for" | "range" | "switch" | "case" | "default" | "select"
            | "break" | "continue" | "return" | "go" | "defer" | "var" | "const"
            | "fallthrough" => HighlightKind::Keyword,
            _ => HighlightKind::Default,
        }
    }
}

/// A line with its syntax highlights
#[derive(Debug, Clone)]
pub struct HighlightedLine {
    pub highlights: Vec<Highlight>,
}

impl HighlightedLine {
    pub fn new() -> Self {
        Self {
            highlights: Vec::new(),
        }
    }

    /// Get the highlight kind at a given column
    pub fn kind_at(&self, col: usize) -> HighlightKind {
        for h in &self.highlights {
            if col >= h.start && col < h.end {
                return h.kind;
            }
        }
        HighlightKind::Default
    }
}

impl Default for HighlightedLine {
    fn default() -> Self {
        Self::new()
    }
}

/// Syntax highlighter for a buffer
pub struct Highlighter {
    parser: Parser,
    tree: Option<Tree>,
    language: Language,
    registry: LanguageRegistry,
    line_highlights: Vec<HighlightedLine>,
    load_error: Option<String>,
}

impl Highlighter {
    /// Create a new highlighter
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            tree: None,
            language: Language::Unknown,
            registry: LanguageRegistry::new(),
            line_highlights: Vec::new(),
            load_error: None,
        }
    }

    /// Set the language for this highlighter
    /// Returns (success, error_message)
    pub fn set_language(&mut self, lang: Language) -> bool {
        if lang == Language::Unknown {
            self.tree = None;
            self.language = lang;
            self.line_highlights.clear();
            self.load_error = None;
            return true;
        }

        match self.registry.load(lang) {
            Some(ts_lang) => {
                if self.parser.set_language(ts_lang).is_ok() {
                    self.language = lang;
                    self.tree = None;
                    self.line_highlights.clear();
                    self.load_error = None;
                    return true;
                } else {
                    self.load_error = Some("Failed to set parser language".to_string());
                }
            }
            None => {
                // Store the fact that we tried but couldn't load
                self.language = lang; // Set the language even if we can't load grammar
                self.load_error = Some(format!("Grammar for {} not loaded", lang.name()));
            }
        }
        false
    }

    /// Set language from file path
    pub fn set_language_from_path(&mut self, path: &Path) -> bool {
        let lang = Language::from_path(path);
        self.set_language(lang)
    }

    /// Get the current language
    pub fn language(&self) -> Language {
        self.language
    }

    /// Check if syntax highlighting is active
    pub fn is_active(&self) -> bool {
        self.language != Language::Unknown && self.tree.is_some()
    }

    /// Get the number of highlighted lines
    pub fn highlight_count(&self) -> usize {
        self.line_highlights.len()
    }

    /// Get status info for debugging
    pub fn status(&self) -> String {
        if self.language == Language::Unknown {
            if let Some(ref err) = self.load_error {
                format!("No language: {}", err)
            } else {
                "No language detected".to_string()
            }
        } else if self.tree.is_none() {
            if let Some(ref err) = self.load_error {
                format!("Language: {} ({})", self.language.name(), err)
            } else {
                format!("Language: {} (grammar not loaded)", self.language.name())
            }
        } else {
            format!(
                "Language: {}, {} lines highlighted",
                self.language.name(),
                self.line_highlights.len()
            )
        }
    }

    /// Parse the given source code
    pub fn parse(&mut self, source: &str) {
        if self.language == Language::Unknown {
            self.line_highlights.clear();
            return;
        }

        self.tree = self.parser.parse(source, self.tree.as_ref());

        // Clone the tree to avoid borrow checker issues
        if let Some(tree) = self.tree.clone() {
            self.build_highlights(source, &tree);
        }
    }

    /// Update highlights after an edit (incremental parsing)
    pub fn update(
        &mut self,
        source: &str,
        _start_byte: usize,
        _old_end_byte: usize,
        _new_end_byte: usize,
    ) {
        if self.language == Language::Unknown {
            return;
        }

        // For now, just do a full reparse
        // TODO: Implement proper incremental parsing with tree.edit()
        self.parse(source);
    }

    /// Get highlights for a specific line
    pub fn line_highlights(&self, line: usize) -> Option<&HighlightedLine> {
        self.line_highlights.get(line)
    }

    /// Debug: dump node types for the first N lines
    pub fn debug_tree(&self, max_lines: usize) -> String {
        let Some(ref tree) = self.tree else {
            return "No parse tree".to_string();
        };

        let mut result = vec![format!("Language: {:?}", self.language)];
        let mut cursor = tree.walk();
        let mut seen_types: std::collections::HashSet<String> = std::collections::HashSet::new();

        fn collect_types(
            cursor: &mut tree_sitter::TreeCursor,
            seen: &mut std::collections::HashSet<String>,
            max_row: usize,
            lang: Language,
        ) {
            loop {
                let node = cursor.node();
                if node.start_position().row > max_row {
                    break;
                }

                let kind = HighlightKind::from_node_type(node.kind(), lang);
                let info = format!(
                    "{}:{}{} -> {:?}",
                    node.kind(),
                    if node.is_named() { "N" } else { "A" },
                    if node.child_count() == 0 { "*" } else { "" },
                    kind
                );
                seen.insert(info);

                if cursor.goto_first_child() {
                    collect_types(cursor, seen, max_row, lang);
                    cursor.goto_parent();
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        collect_types(&mut cursor, &mut seen_types, max_lines, self.language);

        // Show highlights we generated for line 0
        let line0_info = if let Some(hl) = self.line_highlights.get(0) {
            format!("Line 0 highlights: {}", hl.highlights.len())
        } else {
            "No line 0 highlights".to_string()
        };
        result.push(line0_info);

        let mut types: Vec<_> = seen_types.into_iter().collect();
        types.sort();
        result.push(format!("Nodes: {}", types.join(", ")));

        result.join("\n")
    }

    /// Build highlights from the parse tree
    fn build_highlights(&mut self, source: &str, tree: &Tree) {
        // Count lines
        let line_count = source.lines().count().max(1);
        self.line_highlights = vec![HighlightedLine::new(); line_count];

        // Calculate line start offsets
        let mut line_starts: Vec<usize> = vec![0];
        for (i, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(i + 1);
            }
        }

        // Walk the tree and collect highlights
        let mut cursor = tree.walk();
        self.walk_tree_with_parent(&mut cursor, source, &line_starts, None);
    }

    /// Determine highlight kind considering parent context
    fn determine_highlight_kind(
        node_kind: &str,
        parent_kind: Option<&str>,
        lang: Language,
    ) -> HighlightKind {
        // First check for context-sensitive highlighting
        if let Some(parent) = parent_kind {
            match (node_kind, parent) {
                // Macro names (identifier or scoped_identifier inside macro_invocation)
                ("identifier", "macro_invocation") => return HighlightKind::Function,
                ("scoped_identifier", "macro_invocation") => return HighlightKind::Function,
                // The `!` in macros
                ("!", "macro_invocation") => return HighlightKind::Function,
                // Identifiers inside scoped macro names (e.g., tokio in tokio::select!)
                ("identifier", "scoped_identifier") if lang == Language::Rust => {
                    // This will be colored as Type by default, which is fine for paths
                }

                // Function names in call expressions
                ("identifier", "call_expression") => return HighlightKind::Function,
                ("field_identifier", "field_expression") if lang == Language::Rust => {
                    // Method calls like .iter(), .collect()
                    return HighlightKind::Function;
                }
                // Scoped function calls like theme::get_builtin_theme
                ("scoped_identifier", "call_expression") => return HighlightKind::Function,

                // Type context - identifiers in type positions
                ("identifier", "scoped_type_identifier") => return HighlightKind::Type,
                ("identifier", "type_arguments") => return HighlightKind::Type,
                ("identifier", "generic_type") => return HighlightKind::Type,
                ("scoped_identifier", "type_arguments") => return HighlightKind::Type,
                ("scoped_identifier", "generic_type") => return HighlightKind::Type,
                // Type annotations
                ("identifier", "type_binding") => return HighlightKind::Type,
                ("scoped_identifier", "type_binding") => return HighlightKind::Type,

                // Function parameters
                ("identifier", "parameter") => return HighlightKind::Parameter,
                ("identifier", "parameters") => return HighlightKind::Parameter,

                // Struct/enum field definitions
                ("identifier", "field_declaration") => return HighlightKind::Property,

                // Use declarations - color the path
                ("identifier", "use_declaration") => return HighlightKind::Type,
                ("scoped_identifier", "use_declaration") => return HighlightKind::Type,
                ("identifier", "scoped_identifier") => return HighlightKind::Type,
                ("identifier", "use_list") => return HighlightKind::Type,
                ("identifier", "use_as_clause") => return HighlightKind::Type,

                _ => {}
            }
        }

        // Fall back to regular matching
        HighlightKind::from_node_type(node_kind, lang)
    }

    fn walk_tree_with_parent(
        &mut self,
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        line_starts: &[usize],
        parent_kind: Option<&str>,
    ) {
        loop {
            let node = cursor.node();
            let node_kind = node.kind();

            // Determine highlight kind with parent context
            let kind = Self::determine_highlight_kind(node_kind, parent_kind, self.language);

            // Only add highlights for leaf nodes or specific node types
            if kind != HighlightKind::Default
                && (node.child_count() == 0 || is_highlightable_parent(node_kind))
            {
                let start_byte = node.start_byte();
                let end_byte = node.end_byte();
                let start_line = node.start_position().row;
                let end_line = node.end_position().row;

                // Add highlight to each line the node spans
                for line in start_line..=end_line {
                    if line >= self.line_highlights.len() {
                        break;
                    }

                    let line_start = line_starts.get(line).copied().unwrap_or(0);
                    let line_end = line_starts.get(line + 1).copied().unwrap_or(source.len());

                    let highlight_start = if line == start_line {
                        start_byte.saturating_sub(line_start)
                    } else {
                        0
                    };

                    let highlight_end = if line == end_line {
                        end_byte.saturating_sub(line_start)
                    } else {
                        line_end.saturating_sub(line_start)
                    };

                    if highlight_start < highlight_end {
                        self.line_highlights[line].highlights.push(Highlight {
                            start: highlight_start,
                            end: highlight_end,
                            kind,
                        });
                    }
                }
            }

            // Recurse into children with current node as parent
            if cursor.goto_first_child() {
                self.walk_tree_with_parent(cursor, source, line_starts, Some(node_kind));
                cursor.goto_parent();
            }

            // Move to next sibling
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a parent node type should be highlighted as a whole
fn is_highlightable_parent(node_type: &str) -> bool {
    matches!(
        node_type,
        "string"
            | "string_literal"
            | "raw_string"
            | "raw_string_literal"
            | "char_literal"
            | "comment"
            | "line_comment"
            | "block_comment"
            | "doc_comment"
            | "macro_invocation"
            | "attribute_item"
            | "inner_attribute_item"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_rust() {
        let mut highlighter = Highlighter::new();
        // Grammar may not be installed, so set_language may return false
        let set_ok = highlighter.set_language(Language::Rust);

        let source = r#"fn main() {
    let x = 42;
    println!("Hello");
}"#;

        highlighter.parse(source);

        // If grammar was loaded, we should have highlights
        // If not installed, this is fine - just skip the assertion
        if set_ok {
            assert!(highlighter.line_highlights(0).is_some());
        }
    }

    #[test]
    fn test_highlighter_unknown_language() {
        let mut highlighter = Highlighter::new();
        highlighter.set_language(Language::Unknown);

        highlighter.parse("some random text");

        // Should have no highlights for unknown language
        assert!(
            highlighter.line_highlights(0).is_none()
                || highlighter
                    .line_highlights(0)
                    .unwrap()
                    .highlights
                    .is_empty()
        );
    }

    #[test]
    fn test_highlight_kind_from_node() {
        assert_eq!(
            HighlightKind::from_node_type("comment", Language::Rust),
            HighlightKind::Comment
        );
        assert_eq!(
            HighlightKind::from_node_type("string", Language::Python),
            HighlightKind::String
        );
        assert_eq!(
            HighlightKind::from_node_type("integer", Language::Go),
            HighlightKind::Number
        );
    }
}
