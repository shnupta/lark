use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Result of the file finder
#[derive(Debug)]
pub enum FinderResult {
    /// User selected a file
    Selected(PathBuf),
    /// User cancelled (Esc)
    Cancelled,
    /// fzf not found or error
    Error(String),
}

/// Spawn fzf with file list and return the selected file
pub fn find_file(cwd: &PathBuf) -> FinderResult {
    // Check if fzf is available
    if Command::new("fzf").arg("--version").output().is_err() {
        return FinderResult::Error("fzf not found. Install with: brew install fzf".to_string());
    }

    // Use fd if available, otherwise fall back to find
    let file_list = get_file_list(cwd);

    let mut child = match Command::new("fzf")
        .args([
            "--height=40%",
            "--layout=reverse",
            "--border",
            "--prompt=Find file: ",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit()) // Let fzf display its UI
        .current_dir(cwd)
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return FinderResult::Error(format!("Failed to spawn fzf: {}", e)),
    };

    // Write file list to fzf's stdin
    if let Some(mut stdin) = child.stdin.take() {
        for file in file_list {
            let _ = writeln!(stdin, "{}", file);
        }
    }

    // Wait for fzf to complete and get the result
    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => return FinderResult::Error(format!("fzf error: {}", e)),
    };

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if selected.is_empty() {
            FinderResult::Cancelled
        } else {
            FinderResult::Selected(cwd.join(selected))
        }
    } else {
        // fzf returns non-zero when user cancels with Esc
        FinderResult::Cancelled
    }
}

fn get_file_list(cwd: &PathBuf) -> Vec<String> {
    // Try fd first (faster and respects .gitignore)
    if let Ok(output) = Command::new("fd")
        .args(["--type", "f", "--hidden", "--exclude", ".git"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            return BufReader::new(&output.stdout[..])
                .lines()
                .filter_map(|l| l.ok())
                .collect();
        }
    }

    // Fall back to find
    if let Ok(output) = Command::new("find")
        .args([".", "-type", "f", "-not", "-path", "*/.git/*"])
        .current_dir(cwd)
        .output()
    {
        if output.status.success() {
            return BufReader::new(&output.stdout[..])
                .lines()
                .filter_map(|l| l.ok())
                .map(|s| s.trim_start_matches("./").to_string())
                .collect();
        }
    }

    Vec::new()
}
