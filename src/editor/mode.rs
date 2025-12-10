#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
    FileBrowser,
    MessageViewer,
}

impl Mode {
    pub fn display(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
            Mode::FileBrowser => "FILES",
            Mode::MessageViewer => "MESSAGE",
        }
    }
}
