use crossterm::event::KeyCode;

/// 应用操作指令
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppAction {
    // 应用控制
    Quit,
    Refresh,
    ToggleTitle,

    // 输入模式
    StartInput,
    CancelInput,
    ConfirmInput,

    // 导航
    MoveUp,
    MoveDown,
    PageUp,
    PageDown,
    NextChapter,
    PrevChapter,

    // 历史导航
    HistoryUp,
    HistoryDown,

    // 字符输入
    InputChar(char),
    InputBackspace,

    // 无操作
    None,
}

/// 模式类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Input,
}

/// 事件处理器
pub struct EventHandler {
    mode: InputMode,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Normal,
        }
    }

    pub fn mode(&self) -> InputMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn handle_key(&mut self, code: KeyCode) -> AppAction {
        match self.mode {
            InputMode::Normal => self.handle_normal_mode(code),
            InputMode::Input => self.handle_input_mode(code),
        }
    }

    fn handle_normal_mode(&mut self, code: KeyCode) -> AppAction {
        match code {
            KeyCode::Char('q') => AppAction::Quit,
            KeyCode::Char('/') => {
                self.mode = InputMode::Input;
                AppAction::StartInput
            }
            KeyCode::Char('j') | KeyCode::Down => AppAction::MoveDown,
            KeyCode::Char('k') | KeyCode::Up => AppAction::MoveUp,
            KeyCode::Char('n') => AppAction::NextChapter,
            KeyCode::Char('p') => AppAction::PrevChapter,
            KeyCode::Char('r') => AppAction::Refresh,
            KeyCode::Char('s') => AppAction::ToggleTitle,
            KeyCode::PageUp => AppAction::PageUp,
            KeyCode::PageDown => AppAction::PageDown,
            _ => AppAction::None,
        }
    }

    fn handle_input_mode(&mut self, code: KeyCode) -> AppAction {
        match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.mode = InputMode::Normal;
                AppAction::CancelInput
            }
            KeyCode::Up => AppAction::HistoryUp,
            KeyCode::Down => AppAction::HistoryDown,
            KeyCode::Char(c) => AppAction::InputChar(c),
            KeyCode::Backspace => AppAction::InputBackspace,
            KeyCode::Enter => {
                self.mode = InputMode::Normal;
                AppAction::ConfirmInput
            }
            _ => AppAction::None,
        }
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}
