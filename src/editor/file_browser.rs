use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
}

pub struct FileBrowser {
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub root_dir: PathBuf,
    expanded: HashSet<PathBuf>,
}

impl FileBrowser {
    pub fn new() -> Self {
        let root_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut browser = Self {
            entries: Vec::new(),
            selected: 0,
            root_dir,
            expanded: HashSet::new(),
        };
        browser.refresh();
        browser
    }

    pub fn refresh(&mut self) {
        self.entries.clear();
        self.build_tree(&self.root_dir.clone(), 0);
        if self.selected >= self.entries.len() {
            self.selected = self.entries.len().saturating_sub(1);
        }
    }

    fn build_tree(&mut self, dir: &PathBuf, depth: usize) {
        if let Ok(read_dir) = fs::read_dir(dir) {
            let mut entries: Vec<FileEntry> = read_dir
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = e.file_name().to_string_lossy().to_string();
                    FileEntry {
                        name,
                        path,
                        is_dir,
                        depth,
                    }
                })
                .filter(|e| !e.name.starts_with('.'))
                .collect();

            // Sort: directories first, then alphabetically
            entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            for entry in entries {
                let is_expanded = self.expanded.contains(&entry.path);
                let entry_path = entry.path.clone();
                let is_dir = entry.is_dir;
                self.entries.push(entry);

                // If directory is expanded, recurse
                if is_dir && is_expanded {
                    self.build_tree(&entry_path, depth + 1);
                }
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected + 1 < self.entries.len() {
            self.selected += 1;
        }
    }

    /// Toggle directory expansion or return file path
    pub fn select(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected).cloned() {
            if entry.is_dir {
                // Toggle expansion
                if self.expanded.contains(&entry.path) {
                    self.expanded.remove(&entry.path);
                } else {
                    self.expanded.insert(entry.path);
                }
                self.refresh();
                None
            } else {
                Some(entry.path)
            }
        } else {
            None
        }
    }

    /// Check if a directory is expanded
    pub fn is_expanded(&self, path: &PathBuf) -> bool {
        self.expanded.contains(path)
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}
