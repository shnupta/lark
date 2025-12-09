use crossterm::event::{KeyCode, KeyModifiers};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl Key {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn char(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::NONE)
    }

    pub fn ctrl(c: char) -> Self {
        Self::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveToLineStart,
    MoveToLineEnd,
    MoveToFirstLine,
    MoveToLastLine,
    MoveWordForward,
    MoveWordBackward,
    MoveWordEnd,
    PageDown,
    PageUp,

    // Mode changes
    EnterInsertMode,
    EnterInsertModeAppend,
    EnterInsertModeAppendLine,
    EnterInsertModeOpenBelow,
    EnterInsertModeOpenAbove,
    EnterNormalMode,
    EnterCommandMode,

    // Window/pane management
    SplitVertical,
    SplitHorizontal,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    FocusNext,

    // File browser
    ToggleFileBrowser,
    FocusFileBrowser,

    // Leader sequences
    LeaderKey,
    FindFile,
    Grep,

    // Pane selection mode
    SelectPane(char),

    // Tabs
    NewTab,
    NextTab,
    PrevTab,
    CloseTab,

    // Other
    Quit,
}

pub struct KeySequenceState {
    pending: Vec<Key>,
    last_key_time: Instant,
    timeout: Duration,
    pub waiting_for_pane_select: bool,
    pub count: Option<usize>,
}

impl KeySequenceState {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            last_key_time: Instant::now(),
            timeout: Duration::from_millis(1000),
            waiting_for_pane_select: false,
            count: None,
        }
    }

    fn check_timeout(&mut self) {
        if self.last_key_time.elapsed() > self.timeout {
            self.pending.clear();
            self.count = None;
        }
    }

    pub fn process_key(&mut self, key: Key, mode: &str) -> KeyResult {
        self.check_timeout();
        self.last_key_time = Instant::now();

        // Handle pane selection mode
        if self.waiting_for_pane_select {
            if let KeyCode::Char(c) = key.code {
                if c.is_ascii_lowercase() {
                    self.waiting_for_pane_select = false;
                    return KeyResult::Action(Action::SelectPane(c), 1);
                }
            }
            if key.code == KeyCode::Esc {
                self.waiting_for_pane_select = false;
                self.count = None;
                return KeyResult::Cancelled;
            }
            return KeyResult::Pending;
        }

        // Handle count prefix (digits at start, but not 0 as first digit)
        if let KeyCode::Char(c) = key.code {
            if c.is_ascii_digit() && key.modifiers == KeyModifiers::NONE {
                if c != '0' || self.count.is_some() {
                    let digit = c.to_digit(10).unwrap() as usize;
                    self.count = Some(self.count.unwrap_or(0) * 10 + digit);
                    return KeyResult::Pending;
                }
            }
        }

        self.pending.push(key.clone());

        match self.match_sequence(mode) {
            MatchResult::Complete(action) => {
                let count = self.count.unwrap_or(1);
                self.pending.clear();
                self.count = None;
                KeyResult::Action(action, count)
            }
            MatchResult::Prefix => KeyResult::Pending,
            MatchResult::NoMatch => {
                if self.pending.len() > 1 {
                    self.pending.clear();
                    self.pending.push(key);
                    match self.match_sequence(mode) {
                        MatchResult::Complete(action) => {
                            let count = self.count.unwrap_or(1);
                            self.pending.clear();
                            self.count = None;
                            KeyResult::Action(action, count)
                        }
                        MatchResult::Prefix => KeyResult::Pending,
                        MatchResult::NoMatch => {
                            self.pending.clear();
                            self.count = None;
                            KeyResult::Unhandled
                        }
                    }
                } else {
                    self.pending.clear();
                    self.count = None;
                    KeyResult::Unhandled
                }
            }
        }
    }

    fn match_sequence(&self, mode: &str) -> MatchResult {
        let pending = &self.pending;

        // Ctrl-W window commands (work in any mode)
        if !pending.is_empty() && pending[0] == Key::ctrl('w') {
            if pending.len() == 1 {
                return MatchResult::Prefix;
            }
            if pending.len() == 2 {
                let action = match pending[1].code {
                    KeyCode::Char('h') | KeyCode::Left => Some(Action::FocusLeft),
                    KeyCode::Char('j') | KeyCode::Down => Some(Action::FocusDown),
                    KeyCode::Char('k') | KeyCode::Up => Some(Action::FocusUp),
                    KeyCode::Char('l') | KeyCode::Right => Some(Action::FocusRight),
                    KeyCode::Char('w') => Some(Action::FocusNext),
                    KeyCode::Char('v') => Some(Action::SplitVertical),
                    KeyCode::Char('s') => Some(Action::SplitHorizontal),
                    _ => None,
                };
                return match action {
                    Some(a) => MatchResult::Complete(a),
                    None => MatchResult::NoMatch,
                };
            }
        }

        // Ctrl+G toggle file browser (works in any mode)
        if pending.len() == 1 && pending[0] == Key::ctrl('g') {
            return MatchResult::Complete(Action::ToggleFileBrowser);
        }

        // Ctrl+D/U for page down/up
        if pending.len() == 1 && pending[0].modifiers.contains(KeyModifiers::CONTROL) {
            let action = match pending[0].code {
                KeyCode::Char('d') => Some(Action::PageDown),
                KeyCode::Char('u') => Some(Action::PageUp),
                KeyCode::Char('c') => Some(Action::Quit),
                _ => None,
            };
            if let Some(a) = action {
                return MatchResult::Complete(a);
            }
        }

        // Leader key (space) - normal mode only
        if !pending.is_empty() && pending[0] == Key::char(' ') && mode == "normal" {
            if pending.len() == 1 {
                return MatchResult::Prefix;
            }
            if pending.len() == 2 {
                let action = match pending[1].code {
                    KeyCode::Char('f') => Some(Action::LeaderKey), // Prefix for file commands
                    KeyCode::Char('g') => Some(Action::Grep),
                    KeyCode::Char('e') => Some(Action::FocusFileBrowser),
                    _ => None,
                };
                if let Some(a) = action {
                    if a == Action::LeaderKey {
                        return MatchResult::Prefix;
                    }
                    return MatchResult::Complete(a);
                }
            }
            if pending.len() == 3 && pending[1] == Key::char('f') {
                let action = match pending[2].code {
                    KeyCode::Char('f') => Some(Action::FindFile),
                    KeyCode::Char('g') => Some(Action::Grep),
                    _ => None,
                };
                return match action {
                    Some(a) => MatchResult::Complete(a),
                    None => MatchResult::NoMatch,
                };
            }
        }

        // Normal mode commands
        if mode == "normal" {
            // gg - go to first line
            if !pending.is_empty() && pending[0] == Key::char('g') {
                if pending.len() == 1 {
                    return MatchResult::Prefix;
                }
                if pending.len() == 2 && pending[1] == Key::char('g') {
                    return MatchResult::Complete(Action::MoveToFirstLine);
                }
                return MatchResult::NoMatch;
            }

            // tt, tn, tp, tc - tab commands
            if !pending.is_empty() && pending[0] == Key::char('t') {
                if pending.len() == 1 {
                    return MatchResult::Prefix;
                }
                if pending.len() == 2 {
                    let action = match pending[1].code {
                        KeyCode::Char('t') => Some(Action::NewTab),
                        KeyCode::Char('n') => Some(Action::NextTab),
                        KeyCode::Char('p') => Some(Action::PrevTab),
                        KeyCode::Char('c') => Some(Action::CloseTab),
                        _ => None,
                    };
                    return match action {
                        Some(a) => MatchResult::Complete(a),
                        None => MatchResult::NoMatch,
                    };
                }
            }

            // Single key commands
            if pending.len() == 1 {
                let action = match pending[0].code {
                    KeyCode::Char('h') | KeyCode::Left => Some(Action::MoveLeft),
                    KeyCode::Char('j') | KeyCode::Down => Some(Action::MoveDown),
                    KeyCode::Char('k') | KeyCode::Up => Some(Action::MoveUp),
                    KeyCode::Char('l') | KeyCode::Right => Some(Action::MoveRight),
                    KeyCode::Char('0') => Some(Action::MoveToLineStart),
                    KeyCode::Char('$') => Some(Action::MoveToLineEnd),
                    KeyCode::Char('G') => Some(Action::MoveToLastLine),
                    KeyCode::Char('w') => Some(Action::MoveWordForward),
                    KeyCode::Char('b') => Some(Action::MoveWordBackward),
                    KeyCode::Char('e') => Some(Action::MoveWordEnd),
                    KeyCode::Char('i') => Some(Action::EnterInsertMode),
                    KeyCode::Char('a') => Some(Action::EnterInsertModeAppend),
                    KeyCode::Char('A') => Some(Action::EnterInsertModeAppendLine),
                    KeyCode::Char('o') => Some(Action::EnterInsertModeOpenBelow),
                    KeyCode::Char('O') => Some(Action::EnterInsertModeOpenAbove),
                    KeyCode::Char(':') => Some(Action::EnterCommandMode),
                    KeyCode::Esc => Some(Action::EnterNormalMode),
                    _ => None,
                };

                return match action {
                    Some(a) => MatchResult::Complete(a),
                    None => MatchResult::NoMatch,
                };
            }
        }

        if mode == "insert" {
            if pending.len() == 1 {
                let action = match pending[0].code {
                    KeyCode::Esc => Some(Action::EnterNormalMode),
                    KeyCode::Left => Some(Action::MoveLeft),
                    KeyCode::Right => Some(Action::MoveRight),
                    KeyCode::Up => Some(Action::MoveUp),
                    KeyCode::Down => Some(Action::MoveDown),
                    _ => None,
                };
                return match action {
                    Some(a) => MatchResult::Complete(a),
                    None => MatchResult::NoMatch,
                };
            }
        }

        MatchResult::NoMatch
    }

    pub fn pending_display(&self) -> String {
        let mut s = String::new();
        if let Some(count) = self.count {
            s.push_str(&count.to_string());
        }
        for k in &self.pending {
            s.push_str(&key_to_string(k));
        }
        s
    }
}

fn key_to_string(key: &Key) -> String {
    let mut s = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("C-");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        s.push_str("A-");
    }
    match key.code {
        KeyCode::Char(c) => s.push(c),
        KeyCode::Esc => s.push_str("Esc"),
        KeyCode::Enter => s.push_str("Enter"),
        KeyCode::Left => s.push_str("←"),
        KeyCode::Right => s.push_str("→"),
        KeyCode::Up => s.push_str("↑"),
        KeyCode::Down => s.push_str("↓"),
        _ => s.push_str("?"),
    }
    s
}

impl Default for KeySequenceState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum MatchResult {
    Complete(Action),
    Prefix,
    NoMatch,
}

#[derive(Debug)]
pub enum KeyResult {
    Action(Action, usize), // Action with count
    Pending,
    Unhandled,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_key_returns_action() {
        let mut state = KeySequenceState::new();
        let result = state.process_key(Key::char('j'), "normal");

        match result {
            KeyResult::Action(Action::MoveDown, 1) => {}
            _ => panic!("Expected MoveDown action with count 1"),
        }
    }

    #[test]
    fn gg_returns_move_to_first_line() {
        let mut state = KeySequenceState::new();

        // First 'g' should be pending
        let result = state.process_key(Key::char('g'), "normal");
        assert!(matches!(result, KeyResult::Pending));

        // Second 'g' should complete
        let result = state.process_key(Key::char('g'), "normal");
        match result {
            KeyResult::Action(Action::MoveToFirstLine, 1) => {}
            _ => panic!("Expected MoveToFirstLine action"),
        }
    }

    #[test]
    fn count_prefix_works() {
        let mut state = KeySequenceState::new();

        // Type "123j"
        assert!(matches!(
            state.process_key(Key::char('1'), "normal"),
            KeyResult::Pending
        ));
        assert!(matches!(
            state.process_key(Key::char('2'), "normal"),
            KeyResult::Pending
        ));
        assert!(matches!(
            state.process_key(Key::char('3'), "normal"),
            KeyResult::Pending
        ));

        let result = state.process_key(Key::char('j'), "normal");
        match result {
            KeyResult::Action(Action::MoveDown, 123) => {}
            _ => panic!("Expected MoveDown with count 123, got {:?}", result),
        }
    }

    #[test]
    fn ctrl_w_sequence_works() {
        let mut state = KeySequenceState::new();

        // Ctrl-W should be pending
        let result = state.process_key(Key::ctrl('w'), "normal");
        assert!(matches!(result, KeyResult::Pending));

        // 'v' should complete with SplitVertical
        let result = state.process_key(Key::char('v'), "normal");
        match result {
            KeyResult::Action(Action::SplitVertical, 1) => {}
            _ => panic!("Expected SplitVertical action"),
        }
    }

    #[test]
    fn tab_commands_work() {
        let mut state = KeySequenceState::new();

        // tt should create new tab
        assert!(matches!(
            state.process_key(Key::char('t'), "normal"),
            KeyResult::Pending
        ));
        match state.process_key(Key::char('t'), "normal") {
            KeyResult::Action(Action::NewTab, 1) => {}
            _ => panic!("Expected NewTab action"),
        }

        // tn should go to next tab
        assert!(matches!(
            state.process_key(Key::char('t'), "normal"),
            KeyResult::Pending
        ));
        match state.process_key(Key::char('n'), "normal") {
            KeyResult::Action(Action::NextTab, 1) => {}
            _ => panic!("Expected NextTab action"),
        }
    }

    #[test]
    fn insert_mode_esc_returns_to_normal() {
        let mut state = KeySequenceState::new();

        let result = state.process_key(Key::new(KeyCode::Esc, KeyModifiers::NONE), "insert");
        match result {
            KeyResult::Action(Action::EnterNormalMode, 1) => {}
            _ => panic!("Expected EnterNormalMode action"),
        }
    }

    #[test]
    fn pending_display_shows_count_and_keys() {
        let mut state = KeySequenceState::new();
        state.process_key(Key::char('5'), "normal");
        state.process_key(Key::char('g'), "normal");

        assert_eq!(state.pending_display(), "5g");
    }

    #[test]
    fn zero_not_treated_as_count_when_first() {
        let mut state = KeySequenceState::new();

        // '0' as first key should be MoveToLineStart
        let result = state.process_key(Key::char('0'), "normal");
        match result {
            KeyResult::Action(Action::MoveToLineStart, 1) => {}
            _ => panic!("Expected MoveToLineStart action"),
        }
    }

    #[test]
    fn zero_treated_as_count_after_digit() {
        let mut state = KeySequenceState::new();

        // "10j" should move down 10 lines
        state.process_key(Key::char('1'), "normal");
        state.process_key(Key::char('0'), "normal");
        let result = state.process_key(Key::char('j'), "normal");

        match result {
            KeyResult::Action(Action::MoveDown, 10) => {}
            _ => panic!("Expected MoveDown with count 10"),
        }
    }
}
