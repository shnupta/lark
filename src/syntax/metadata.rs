//! Grammar metadata tracking for ABI version compatibility
//!
//! Tracks which tree-sitter ABI version each grammar was compiled with,
//! and triggers auto-reinstall when versions don't match.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::languages::Language;

/// Current tree-sitter ABI version
/// This should match the tree-sitter crate version
pub const TREE_SITTER_ABI_VERSION: u32 = 14; // tree-sitter 0.24.x uses ABI 14

/// Metadata for a single installed grammar
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrammarInfo {
    /// ABI version the grammar was compiled with
    pub abi_version: u32,
    /// When the grammar was installed
    pub installed_at: String,
    /// Git commit hash (if available)
    pub commit: Option<String>,
}

/// Metadata store for all installed grammars
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GrammarMetadata {
    /// Map of grammar name to info
    pub grammars: HashMap<String, GrammarInfo>,
}

impl GrammarMetadata {
    /// Load metadata from disk
    pub fn load() -> Self {
        let path = Self::metadata_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(metadata) = serde_json::from_str(&content) {
                    return metadata;
                }
            }
        }
        Self::default()
    }

    /// Save metadata to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::metadata_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create metadata directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write metadata: {}", e))
    }

    /// Get the metadata file path
    fn metadata_path() -> PathBuf {
        dirs::home_dir()
            .map(|h| {
                h.join(".config")
                    .join("lark")
                    .join("grammars")
                    .join("metadata.json")
            })
            .unwrap_or_else(|| PathBuf::from("grammars/metadata.json"))
    }

    /// Record that a grammar was installed
    pub fn record_install(&mut self, lang: Language) {
        if let Some(name) = lang.grammar_name() {
            self.grammars.insert(
                name.to_string(),
                GrammarInfo {
                    abi_version: TREE_SITTER_ABI_VERSION,
                    installed_at: chrono_lite_now(),
                    commit: None,
                },
            );
        }
    }

    /// Record that a grammar was uninstalled
    pub fn record_uninstall(&mut self, lang: Language) {
        if let Some(name) = lang.grammar_name() {
            self.grammars.remove(name);
        }
    }

    /// Check if a grammar needs reinstalling due to ABI mismatch
    pub fn needs_reinstall(&self, lang: Language) -> bool {
        if let Some(name) = lang.grammar_name() {
            if let Some(info) = self.grammars.get(name) {
                return info.abi_version != TREE_SITTER_ABI_VERSION;
            }
        }
        false
    }

    /// Get list of grammars that need reinstalling
    pub fn outdated_grammars(&self) -> Vec<String> {
        self.grammars
            .iter()
            .filter(|(_, info)| info.abi_version != TREE_SITTER_ABI_VERSION)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Check if a grammar is installed (in metadata)
    pub fn is_installed(&self, lang: Language) -> bool {
        if let Some(name) = lang.grammar_name() {
            self.grammars.contains_key(name)
        } else {
            false
        }
    }
}

/// Simple timestamp without external crate
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_serialization() {
        let mut metadata = GrammarMetadata::default();
        metadata.record_install(Language::Rust);

        let json = serde_json::to_string(&metadata).unwrap();
        let loaded: GrammarMetadata = serde_json::from_str(&json).unwrap();

        assert!(loaded.is_installed(Language::Rust));
    }

    #[test]
    fn test_needs_reinstall() {
        let mut metadata = GrammarMetadata::default();
        metadata.record_install(Language::Rust);

        // Current version should not need reinstall
        assert!(!metadata.needs_reinstall(Language::Rust));

        // Manually set old version
        if let Some(info) = metadata.grammars.get_mut("rust") {
            info.abi_version = 13; // Old version
        }

        assert!(metadata.needs_reinstall(Language::Rust));
    }
}
