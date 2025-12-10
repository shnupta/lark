//! Grammar installer for Tree-sitter
//!
//! Downloads and compiles Tree-sitter grammars from GitHub.
//! Tracks ABI versions and auto-reinstalls when needed.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::languages::Language;
use super::metadata::GrammarMetadata;

/// Result of a grammar installation
#[derive(Debug)]
pub enum InstallResult {
    Success,
    AlreadyInstalled,
    Reinstalled, // Grammar was outdated and reinstalled
    Error(String),
}

/// Grammar installer
pub struct GrammarInstaller {
    grammars_dir: PathBuf,
    cache_dir: PathBuf,
    metadata: GrammarMetadata,
}

impl GrammarInstaller {
    /// Create a new installer
    pub fn new() -> Self {
        let base_dir = dirs::home_dir()
            .map(|h| h.join(".config").join("lark"))
            .unwrap_or_else(|| PathBuf::from(".lark"));

        Self {
            grammars_dir: base_dir.join("grammars"),
            cache_dir: base_dir.join("cache"),
            metadata: GrammarMetadata::load(),
        }
    }

    /// Get the grammars directory
    pub fn grammars_dir(&self) -> &Path {
        &self.grammars_dir
    }

    /// Check if a grammar needs reinstalling due to ABI mismatch
    pub fn needs_reinstall(&self, lang: Language) -> bool {
        self.metadata.needs_reinstall(lang)
    }

    /// Get list of outdated grammars
    pub fn outdated_grammars(&self) -> Vec<String> {
        self.metadata.outdated_grammars()
    }

    /// Check and auto-reinstall a grammar if ABI is outdated
    pub fn ensure_compatible(&mut self, lang: Language) -> InstallResult {
        if self.metadata.needs_reinstall(lang) {
            // Force reinstall by removing the old library first
            if let Some(name) = lang.grammar_name() {
                let lib_path = self.library_path(name);
                let _ = std::fs::remove_file(&lib_path);
            }
            match self.install_internal(lang, true) {
                InstallResult::Success => InstallResult::Reinstalled,
                other => other,
            }
        } else {
            InstallResult::AlreadyInstalled
        }
    }

    /// Install a grammar
    pub fn install(&mut self, lang: Language) -> InstallResult {
        self.install_internal(lang, false)
    }

    /// Internal install implementation
    fn install_internal(&mut self, lang: Language, force: bool) -> InstallResult {
        let grammar_name = match lang.grammar_name() {
            Some(name) => name,
            None => return InstallResult::Error("Unknown language".to_string()),
        };

        let repo = match lang.grammar_repo() {
            Some(repo) => repo,
            None => return InstallResult::Error("No repository for this language".to_string()),
        };

        // Check if already installed (unless forcing)
        let lib_path = self.library_path(grammar_name);
        if lib_path.exists() && !force {
            return InstallResult::AlreadyInstalled;
        }

        // Ensure directories exist
        if let Err(e) = std::fs::create_dir_all(&self.grammars_dir) {
            return InstallResult::Error(format!("Failed to create grammars directory: {}", e));
        }
        if let Err(e) = std::fs::create_dir_all(&self.cache_dir) {
            return InstallResult::Error(format!("Failed to create cache directory: {}", e));
        }

        // Clone or update the repository
        let repo_dir = self.cache_dir.join(grammar_name);
        if repo_dir.exists() {
            // Pull latest
            let status = Command::new("git")
                .args(["pull", "--depth=1"])
                .current_dir(&repo_dir)
                .status();

            if let Err(e) = status {
                return InstallResult::Error(format!("Failed to update repository: {}", e));
            }
        } else {
            // Clone
            let url = format!("https://github.com/{}.git", repo);
            let status = Command::new("git")
                .args(["clone", "--depth=1", &url])
                .arg(&repo_dir)
                .status();

            match status {
                Ok(s) if s.success() => {}
                Ok(s) => {
                    return InstallResult::Error(format!(
                        "git clone failed with exit code: {:?}",
                        s.code()
                    ));
                }
                Err(e) => {
                    return InstallResult::Error(format!("Failed to clone repository: {}", e));
                }
            }
        }

        // Regenerate the grammar to ensure ABI compatibility
        if let Err(e) = self.regenerate_grammar(&repo_dir, lang) {
            // Not fatal - try to compile with existing files
            eprintln!("[syntax] Warning: Could not regenerate grammar: {}", e);
        }

        // Find the source directory (some repos have src/ in root, some in subdirs)
        let src_dir = self.find_src_dir(&repo_dir, lang);
        if !src_dir.exists() {
            return InstallResult::Error(format!(
                "Could not find parser.c in repository at {:?}",
                src_dir
            ));
        }

        // Compile the grammar
        match self.compile_grammar(grammar_name, &src_dir) {
            Ok(_) => {
                // Record in metadata
                self.metadata.record_install(lang);
                if let Err(e) = self.metadata.save() {
                    eprintln!("[syntax] Warning: Failed to save metadata: {}", e);
                }
                InstallResult::Success
            }
            Err(e) => InstallResult::Error(e),
        }
    }

    /// Regenerate the grammar using tree-sitter CLI
    fn regenerate_grammar(&self, repo_dir: &Path, lang: Language) -> Result<(), String> {
        // Check if tree-sitter CLI is available
        if Command::new("tree-sitter")
            .arg("--version")
            .output()
            .is_err()
        {
            return Err(
                "tree-sitter CLI not found. Install with: npm install -g tree-sitter-cli"
                    .to_string(),
            );
        }

        // For TypeScript, we need to generate in the subdirectory
        let generate_dir = match lang {
            Language::TypeScript => repo_dir.join("typescript"),
            Language::Tsx => repo_dir.join("tsx"),
            _ => repo_dir.to_path_buf(),
        };

        // Run tree-sitter generate
        let output = Command::new("tree-sitter")
            .arg("generate")
            .current_dir(&generate_dir)
            .output()
            .map_err(|e| format!("Failed to run tree-sitter generate: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "tree-sitter generate failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Find the source directory containing parser.c
    fn find_src_dir(&self, repo_dir: &Path, lang: Language) -> PathBuf {
        // TypeScript has subdirectories for typescript and tsx
        if lang == Language::TypeScript {
            return repo_dir.join("typescript").join("src");
        }
        if lang == Language::Tsx {
            return repo_dir.join("tsx").join("src");
        }

        // Standard location
        let standard = repo_dir.join("src");
        if standard.join("parser.c").exists() {
            return standard;
        }

        // Some repos have it in a grammar subdirectory
        let grammar_subdir = repo_dir.join("grammar").join("src");
        if grammar_subdir.join("parser.c").exists() {
            return grammar_subdir;
        }

        standard
    }

    /// Compile a grammar to a dynamic library
    fn compile_grammar(&self, name: &str, src_dir: &Path) -> Result<(), String> {
        let parser_c = src_dir.join("parser.c");
        let scanner_c = src_dir.join("scanner.c");
        let scanner_cc = src_dir.join("scanner.cc");

        if !parser_c.exists() {
            return Err(format!("parser.c not found at {:?}", parser_c));
        }

        let lib_path = self.library_path(name);

        // Compile using cc
        #[cfg(target_os = "macos")]
        let compile_result =
            self.compile_macos(name, &parser_c, &scanner_c, &scanner_cc, &lib_path);

        #[cfg(target_os = "linux")]
        let compile_result =
            self.compile_linux(name, &parser_c, &scanner_c, &scanner_cc, &lib_path);

        #[cfg(target_os = "windows")]
        let compile_result =
            self.compile_windows(name, &parser_c, &scanner_c, &scanner_cc, &lib_path);

        compile_result
    }

    #[cfg(target_os = "macos")]
    fn compile_macos(
        &self,
        _name: &str,
        parser_c: &Path,
        scanner_c: &Path,
        scanner_cc: &Path,
        lib_path: &Path,
    ) -> Result<(), String> {
        let mut args = vec![
            "-shared",
            "-fPIC",
            "-O2",
            "-I",
            parser_c.parent().unwrap().to_str().unwrap(),
        ];

        let parser_c_str = parser_c.to_str().unwrap();
        args.push(parser_c_str);

        // Add scanner if it exists
        let scanner_c_str;
        let scanner_cc_str;
        if scanner_c.exists() {
            scanner_c_str = scanner_c.to_str().unwrap().to_string();
            args.push(&scanner_c_str);
        } else if scanner_cc.exists() {
            scanner_cc_str = scanner_cc.to_str().unwrap().to_string();
            args.push(&scanner_cc_str);
            args.push("-lstdc++");
        }

        args.push("-o");
        let lib_path_str = lib_path.to_str().unwrap();
        args.push(lib_path_str);

        let output = Command::new("cc")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run compiler: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    #[cfg(target_os = "linux")]
    fn compile_linux(
        &self,
        _name: &str,
        parser_c: &Path,
        scanner_c: &Path,
        scanner_cc: &Path,
        lib_path: &Path,
    ) -> Result<(), String> {
        let mut args = vec![
            "-shared",
            "-fPIC",
            "-O2",
            "-I",
            parser_c.parent().unwrap().to_str().unwrap(),
        ];

        let parser_c_str = parser_c.to_str().unwrap();
        args.push(parser_c_str);

        let scanner_c_str;
        let scanner_cc_str;
        if scanner_c.exists() {
            scanner_c_str = scanner_c.to_str().unwrap().to_string();
            args.push(&scanner_c_str);
        } else if scanner_cc.exists() {
            scanner_cc_str = scanner_cc.to_str().unwrap().to_string();
            args.push(&scanner_cc_str);
            args.push("-lstdc++");
        }

        args.push("-o");
        let lib_path_str = lib_path.to_str().unwrap();
        args.push(lib_path_str);

        let output = Command::new("cc")
            .args(&args)
            .output()
            .map_err(|e| format!("Failed to run compiler: {}", e))?;

        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Compilation failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    #[cfg(target_os = "windows")]
    fn compile_windows(
        &self,
        _name: &str,
        _parser_c: &Path,
        _scanner_c: &Path,
        _scanner_cc: &Path,
        _lib_path: &Path,
    ) -> Result<(), String> {
        Err("Windows compilation not yet implemented. Please use WSL.".to_string())
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

    /// Uninstall a grammar
    pub fn uninstall(&mut self, lang: Language) -> Result<(), String> {
        let grammar_name = lang
            .grammar_name()
            .ok_or_else(|| "Unknown language".to_string())?;

        let lib_path = self.library_path(grammar_name);
        if lib_path.exists() {
            std::fs::remove_file(&lib_path)
                .map_err(|e| format!("Failed to remove grammar: {}", e))?;
        }

        // Also remove cached source
        let cache_dir = self.cache_dir.join(grammar_name);
        if cache_dir.exists() {
            std::fs::remove_dir_all(&cache_dir)
                .map_err(|e| format!("Failed to remove cache: {}", e))?;
        }

        // Remove from metadata
        self.metadata.record_uninstall(lang);
        if let Err(e) = self.metadata.save() {
            eprintln!("[syntax] Warning: Failed to save metadata: {}", e);
        }

        Ok(())
    }

    /// Reinstall all outdated grammars
    pub fn reinstall_outdated(&mut self) -> Vec<(Language, InstallResult)> {
        let outdated: Vec<Language> = Language::all_installable()
            .into_iter()
            .filter(|lang| self.metadata.needs_reinstall(*lang))
            .collect();

        outdated
            .into_iter()
            .map(|lang| {
                let result = self.ensure_compatible(lang);
                (lang, result)
            })
            .collect()
    }
}

impl Default for GrammarInstaller {
    fn default() -> Self {
        Self::new()
    }
}
