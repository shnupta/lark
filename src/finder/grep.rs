use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// A single grep match result
#[derive(Debug, Clone)]
pub struct GrepMatch {
    pub file: PathBuf,
    pub line: usize,
    pub col: usize,
    pub text: String,
}

/// Result of grep operation
pub enum GrepResult {
    /// User selected a match
    Selected(GrepMatch),
    /// User cancelled
    Cancelled,
    /// No matches found
    NoMatches,
    /// Error occurred
    Error(String),
}

/// Grep files with ripgrep and let user select with fzf
pub fn grep_files(pattern: &str, cwd: &PathBuf) -> GrepResult {
    // Check if rg is available
    if Command::new("rg").arg("--version").output().is_err() {
        return GrepResult::Error(
            "ripgrep not found. Install with: brew install ripgrep".to_string(),
        );
    }

    // Check if fzf is available
    if Command::new("fzf").arg("--version").output().is_err() {
        return GrepResult::Error("fzf not found. Install with: brew install fzf".to_string());
    }

    // Run ripgrep
    let rg_output = match Command::new("rg")
        .args([
            "--line-number",
            "--column",
            "--color=never",
            "--no-heading",
            pattern,
        ])
        .current_dir(cwd)
        .output()
    {
        Ok(output) => output,
        Err(e) => return GrepResult::Error(format!("Failed to run rg: {}", e)),
    };

    if rg_output.stdout.is_empty() {
        return GrepResult::NoMatches;
    }

    // Parse rg output into lines
    let matches: Vec<String> = BufReader::new(&rg_output.stdout[..])
        .lines()
        .filter_map(|l| l.ok())
        .collect();

    if matches.is_empty() {
        return GrepResult::NoMatches;
    }

    // Pipe to fzf for selection
    let mut child = match Command::new("fzf")
        .args([
            "--height=40%",
            "--layout=reverse",
            "--border",
            "--prompt=Grep: ",
            "--delimiter=:",
            "--preview-window=right:50%",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .current_dir(cwd)
        .spawn()
    {
        Ok(child) => child,
        Err(e) => return GrepResult::Error(format!("Failed to spawn fzf: {}", e)),
    };

    // Write matches to fzf
    if let Some(mut stdin) = child.stdin.take() {
        for m in &matches {
            let _ = writeln!(stdin, "{}", m);
        }
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(e) => return GrepResult::Error(format!("fzf error: {}", e)),
    };

    if output.status.success() {
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if selected.is_empty() {
            return GrepResult::Cancelled;
        }

        // Parse: file:line:col:text
        if let Some(grep_match) = parse_rg_line(&selected, cwd) {
            GrepResult::Selected(grep_match)
        } else {
            GrepResult::Error("Failed to parse selection".to_string())
        }
    } else {
        GrepResult::Cancelled
    }
}

fn parse_rg_line(line: &str, cwd: &PathBuf) -> Option<GrepMatch> {
    // Format: file:line:col:text
    let mut parts = line.splitn(4, ':');
    let file = parts.next()?;
    let line_num: usize = parts.next()?.parse().ok()?;
    let col: usize = parts.next()?.parse().ok()?;
    let text = parts.next().unwrap_or("").to_string();

    Some(GrepMatch {
        file: cwd.join(file),
        line: line_num,
        col,
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rg_line() {
        let cwd = PathBuf::from("/home/user/project");
        let line = "src/main.rs:42:10:fn main() {";
        let result = parse_rg_line(line, &cwd).unwrap();

        assert_eq!(result.file, PathBuf::from("/home/user/project/src/main.rs"));
        assert_eq!(result.line, 42);
        assert_eq!(result.col, 10);
        assert_eq!(result.text, "fn main() {");
    }

    #[test]
    fn test_parse_rg_line_with_colons_in_text() {
        let cwd = PathBuf::from("/home/user");
        let line = "test.rs:1:5:let x: i32 = 0;";
        let result = parse_rg_line(line, &cwd).unwrap();

        assert_eq!(result.line, 1);
        assert_eq!(result.col, 5);
        assert_eq!(result.text, "let x: i32 = 0;");
    }
}
