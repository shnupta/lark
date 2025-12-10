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

            // Keywords (language-specific patterns included)
            "keyword" | "storage_class" | "visibility_modifier" | "mutable_specifier" => {
                HighlightKind::Keyword
            }

            // Functions
            "function_item"
            | "function_definition"
            | "method_definition"
            | "function_declaration"
            | "call_expression"
            | "method_call" => HighlightKind::Function,

            // Types
            "type"
            | "type_identifier"
            | "primitive_type"
            | "type_annotation"
            | "type_arguments"
            | "generic_type"
            | "struct_item"
            | "enum_item"
            | "trait_item"
            | "class_definition"
            | "interface_declaration" => HighlightKind::Type,

            // Variables and identifiers
            "identifier" | "variable" | "field_identifier" | "shorthand_field_identifier" => {
                HighlightKind::Variable
            }

            // Operators
            "operator"
            | "binary_expression"
            | "unary_expression"
            | "comparison_operator"
            | "assignment_operator" => HighlightKind::Operator,

            // Punctuation
            "delimiter" | "bracket" | "parenthesis" | "brace" | "semicolon" | "comma" | "colon"
            | "arrow" | "fat_arrow" => HighlightKind::Punctuation,

            // Properties/fields
            "property" | "property_identifier" | "field_expression" | "member_expression" => {
                HighlightKind::Property
            }

            // Constants
            "true" | "false" | "null" | "none" | "nil" | "boolean" | "constant" | "const_item" => {
                HighlightKind::Constant
            }

            // Namespaces/modules
            "namespace" | "module" | "use_declaration" | "import_statement" | "import" | "use" => {
                HighlightKind::Namespace
            }

            // Parameters
            "parameter" | "formal_parameter" | "parameters" => HighlightKind::Parameter,

            // Labels
            "label" | "loop_label" | "lifetime" => HighlightKind::Label,

            // Language-specific patterns
            _ => Self::from_language_specific(node_type, lang),
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
            "let" | "fn" | "pub" | "mod" | "use" | "struct" | "enum" | "trait" | "impl" | "for"
            | "loop" | "while" | "if" | "else" | "match" | "return" | "break" | "continue"
            | "async" | "await" | "const" | "static" | "mut" | "ref" | "self" | "super"
            | "crate" | "where" | "as" | "in" | "dyn" | "move" | "type" | "unsafe" | "extern" => {
                HighlightKind::Keyword
            }
            "macro_invocation" | "macro_definition" | "macro_rules" => HighlightKind::Function,
            "attribute_item" | "inner_attribute_item" => HighlightKind::Label,
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
        self.walk_tree(&mut cursor, source, &line_starts);
    }

    fn walk_tree(
        &mut self,
        cursor: &mut tree_sitter::TreeCursor,
        source: &str,
        line_starts: &[usize],
    ) {
        loop {
            let node = cursor.node();
            let kind = HighlightKind::from_node_type(node.kind(), self.language);

            // Only add highlights for leaf nodes or specific node types
            if kind != HighlightKind::Default
                && (node.child_count() == 0 || is_highlightable_parent(node.kind()))
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

            // Recurse into children
            if cursor.goto_first_child() {
                self.walk_tree(cursor, source, line_starts);
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
            | "comment"
            | "line_comment"
            | "block_comment"
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
