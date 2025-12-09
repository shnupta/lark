use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

pub struct FileBrowser {
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub current_dir: PathBuf,
}

impl FileBrowser {
    pub fn new() -> Self {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut browser = Self {
            entries: Vec::new(),
            selected: 0,
            current_dir,
        };
        browser.refresh();
        browser
    }

    pub fn refresh(&mut self) {
        self.entries.clear();

        // Add parent directory entry if not at root
        if let Some(parent) = self.current_dir.parent() {
            self.entries.push(FileEntry {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
            });
        }

        // Read directory entries
        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut entries: Vec<FileEntry> = read_dir
                .filter_map(|e| e.ok())
                .map(|e| {
                    let path = e.path();
                    let is_dir = path.is_dir();
                    let name = e.file_name().to_string_lossy().to_string();
                    FileEntry { name, path, is_dir }
                })
                .filter(|e| !e.name.starts_with('.')) // Hide dotfiles
                .collect();

            // Sort: directories first, then alphabetically
            entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.entries.extend(entries);
        }

        self.selected = 0;
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

    /// Returns Some(path) if a file was selected, None if directory was entered
    pub fn select(&mut self) -> Option<PathBuf> {
        if let Some(entry) = self.entries.get(self.selected).cloned() {
            if entry.is_dir {
                self.current_dir = entry.path;
                self.refresh();
                None
            } else {
                Some(entry.path)
            }
        } else {
            None
        }
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}
