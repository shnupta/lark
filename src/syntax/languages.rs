//! Language registry for Tree-sitter grammars
//!
//! Grammars are loaded dynamically from `~/.config/lark/grammars/`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use libloading::{Library, Symbol};

use super::installer::GrammarInstaller;
use super::metadata::GrammarMetadata;

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Tsx,
    Go,
    C,
    Cpp,
    Json,
    Toml,
    Markdown,
    Bash,
    Lua,
    Ruby,
    Html,
    Css,
    Yaml,
    Unknown,
}

impl Language {
    /// Detect language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" | "pyw" | "pyi" => Language::Python,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "ts" | "mts" | "cts" => Language::TypeScript,
            "tsx" => Language::Tsx,
            "go" => Language::Go,
            "c" | "h" => Language::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "hh" => Language::Cpp,
            "json" => Language::Json,
            "toml" => Language::Toml,
            "md" | "markdown" => Language::Markdown,
            "sh" | "bash" | "zsh" => Language::Bash,
            "lua" => Language::Lua,
            "rb" => Language::Ruby,
            "html" | "htm" => Language::Html,
            "css" | "scss" | "sass" => Language::Css,
            "yaml" | "yml" => Language::Yaml,
            _ => Language::Unknown,
        }
    }

    /// Detect language from file path
    pub fn from_path(path: &Path) -> Self {
        // Check special filenames first
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            match name {
                "Cargo.toml" | "Cargo.lock" => return Language::Toml,
                "package.json" | "tsconfig.json" | "composer.json" => return Language::Json,
                "Makefile" | "makefile" | "GNUmakefile" => return Language::Unknown,
                ".bashrc" | ".bash_profile" | ".zshrc" => return Language::Bash,
                _ => {}
            }
        }

        // Then check extension
        path.extension()
            .and_then(|e| e.to_str())
            .map(Self::from_extension)
            .unwrap_or(Language::Unknown)
    }

    /// Get the display name for this language
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Tsx => "TSX",
            Language::Go => "Go",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::Json => "JSON",
            Language::Toml => "TOML",
            Language::Markdown => "Markdown",
            Language::Bash => "Bash",
            Language::Lua => "Lua",
            Language::Ruby => "Ruby",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Yaml => "YAML",
            Language::Unknown => "Plain Text",
        }
    }

    /// Get the grammar name (used for library loading)
    pub fn grammar_name(&self) -> Option<&'static str> {
        match self {
            Language::Rust => Some("rust"),
            Language::Python => Some("python"),
            Language::JavaScript => Some("javascript"),
            Language::TypeScript => Some("typescript"),
            Language::Tsx => Some("tsx"),
            Language::Go => Some("go"),
            Language::C => Some("c"),
            Language::Cpp => Some("cpp"),
            Language::Json => Some("json"),
            Language::Toml => Some("toml"),
            Language::Markdown => Some("markdown"),
            Language::Bash => Some("bash"),
            Language::Lua => Some("lua"),
            Language::Ruby => Some("ruby"),
            Language::Html => Some("html"),
            Language::Css => Some("css"),
            Language::Yaml => Some("yaml"),
            Language::Unknown => None,
        }
    }

    /// Get the GitHub repository for this grammar
    pub fn grammar_repo(&self) -> Option<&'static str> {
        match self {
            Language::Rust => Some("tree-sitter/tree-sitter-rust"),
            Language::Python => Some("tree-sitter/tree-sitter-python"),
            Language::JavaScript => Some("tree-sitter/tree-sitter-javascript"),
            Language::TypeScript => Some("tree-sitter/tree-sitter-typescript"),
            Language::Tsx => Some("tree-sitter/tree-sitter-typescript"),
            Language::Go => Some("tree-sitter/tree-sitter-go"),
            Language::C => Some("tree-sitter/tree-sitter-c"),
            Language::Cpp => Some("tree-sitter/tree-sitter-cpp"),
            Language::Json => Some("tree-sitter/tree-sitter-json"),
            Language::Toml => Some("tree-sitter/tree-sitter-toml"),
            Language::Markdown => Some("tree-sitter-grammars/tree-sitter-markdown"),
            Language::Bash => Some("tree-sitter/tree-sitter-bash"),
            Language::Lua => Some("tree-sitter-grammars/tree-sitter-lua"),
            Language::Ruby => Some("tree-sitter/tree-sitter-ruby"),
            Language::Html => Some("tree-sitter/tree-sitter-html"),
            Language::Css => Some("tree-sitter/tree-sitter-css"),
            Language::Yaml => Some("tree-sitter-grammars/tree-sitter-yaml"),
            Language::Unknown => None,
        }
    }

    /// List all installable languages
    pub fn all_installable() -> Vec<Language> {
        vec![
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Go,
            Language::C,
            Language::Cpp,
            Language::Json,
            Language::Toml,
            Language::Markdown,
            Language::Bash,
            Language::Lua,
            Language::Ruby,
            Language::Html,
            Language::Css,
            Language::Yaml,
        ]
    }
}

/// A loaded grammar library
struct LoadedGrammar {
    #[allow(dead_code)]
    library: Library,
    language: tree_sitter::Language,
}

/// Registry of available Tree-sitter languages
pub struct LanguageRegistry {
    grammars_dir: PathBuf,
    loaded: HashMap<Language, LoadedGrammar>,
    metadata: GrammarMetadata,
    installer: GrammarInstaller,
}

impl LanguageRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        let grammars_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("lark").join("grammars"))
            .unwrap_or_else(|| PathBuf::from("grammars"));

        Self {
            grammars_dir,
            loaded: HashMap::new(),
            metadata: GrammarMetadata::load(),
            installer: GrammarInstaller::new(),
        }
    }

    /// Get the grammars directory path
    pub fn grammars_dir(&self) -> &Path {
        &self.grammars_dir
    }

    /// Check if a grammar is installed
    pub fn is_installed(&self, lang: Language) -> bool {
        if let Some(name) = lang.grammar_name() {
            let lib_path = self.library_path(name);
            lib_path.exists()
        } else {
            false
        }
    }

    /// Check if a grammar needs reinstalling due to ABI mismatch
    pub fn needs_reinstall(&self, lang: Language) -> bool {
        self.metadata.needs_reinstall(lang)
    }

    /// Get list of outdated grammars that need reinstalling
    pub fn outdated_grammars(&self) -> Vec<String> {
        self.metadata.outdated_grammars()
    }

    /// Get the library path for a grammar
    fn library_path(&self, name: &str) -> PathBuf {
        #[cfg(target_os = "macos")]
        let ext = "dylib";
        #[cfg(target_os = "linux")]
        let ext = "so";
        #[cfg(target_os = "windows")]
        let ext = "dll";

        self.grammars_dir.join(format!("lib{}.{}", name, ext))
    }

    /// Load a grammar if installed, auto-reinstalling if ABI is outdated
    pub fn load(&mut self, lang: Language) -> Option<&tree_sitter::Language> {
        // Already loaded?
        if self.loaded.contains_key(&lang) {
            return self.loaded.get(&lang).map(|g| &g.language);
        }

        // Get grammar name
        let name = lang.grammar_name()?;

        // Check if installed
        let lib_path = self.library_path(name);
        if !lib_path.exists() {
            return None;
        }

        // Check ABI version - auto-reinstall if outdated
        if self.metadata.needs_reinstall(lang) {
            eprintln!(
                "[syntax] Grammar {} has outdated ABI, reinstalling...",
                name
            );

            // Remove from loaded cache (in case it was somehow there)
            self.loaded.remove(&lang);

            // Reinstall
            match self.installer.ensure_compatible(lang) {
                super::installer::InstallResult::Reinstalled => {
                    eprintln!("[syntax] Successfully reinstalled {}", name);
                    // Reload metadata after reinstall
                    self.metadata = GrammarMetadata::load();
                }
                super::installer::InstallResult::Error(e) => {
                    eprintln!("[syntax] Failed to reinstall {}: {}", name, e);
                    return None;
                }
                _ => {}
            }
        }

        // Load the dynamic library
        let library = unsafe { Library::new(&lib_path).ok()? };

        // Get the language function
        let func_name = format!("tree_sitter_{}", name);
        let language = unsafe {
            let func: Symbol<unsafe extern "C" fn() -> tree_sitter::Language> =
                library.get(func_name.as_bytes()).ok()?;
            func()
        };

        self.loaded
            .insert(lang, LoadedGrammar { library, language });

        self.loaded.get(&lang).map(|g| &g.language)
    }

    /// List installed grammars
    pub fn installed(&self) -> Vec<Language> {
        Language::all_installable()
            .into_iter()
            .filter(|lang| self.is_installed(*lang))
            .collect()
    }

    /// List not-yet-installed grammars
    pub fn not_installed(&self) -> Vec<Language> {
        Language::all_installable()
            .into_iter()
            .filter(|lang| !self.is_installed(*lang))
            .collect()
    }

    /// Get a mutable reference to the installer
    pub fn installer_mut(&mut self) -> &mut GrammarInstaller {
        &mut self.installer
    }

    /// Reload metadata from disk (after external changes)
    pub fn reload_metadata(&mut self) {
        self.metadata = GrammarMetadata::load();
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Language::Rust);
        assert_eq!(Language::from_extension("py"), Language::Python);
        assert_eq!(Language::from_extension("js"), Language::JavaScript);
        assert_eq!(Language::from_extension("ts"), Language::TypeScript);
        assert_eq!(Language::from_extension("go"), Language::Go);
        assert_eq!(Language::from_extension("json"), Language::Json);
        assert_eq!(Language::from_extension("xyz"), Language::Unknown);
    }

    #[test]
    fn test_language_from_path() {
        assert_eq!(
            Language::from_path(Path::new("src/main.rs")),
            Language::Rust
        );
        assert_eq!(Language::from_path(Path::new("Cargo.toml")), Language::Toml);
        assert_eq!(
            Language::from_path(Path::new("package.json")),
            Language::Json
        );
    }

    #[test]
    fn test_grammar_names() {
        assert_eq!(Language::Rust.grammar_name(), Some("rust"));
        assert_eq!(Language::Python.grammar_name(), Some("python"));
        assert_eq!(Language::Unknown.grammar_name(), None);
    }

    #[test]
    fn test_grammar_repos() {
        assert_eq!(
            Language::Rust.grammar_repo(),
            Some("tree-sitter/tree-sitter-rust")
        );
        assert_eq!(Language::Unknown.grammar_repo(), None);
    }
}
