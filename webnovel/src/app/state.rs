use ratatui::widgets::ListState;

/// 显示内容数据
#[derive(Debug, Clone)]
pub struct ContentData {
    pub lines: Vec<String>,
    pub title: String,
    pub is_loading: bool,
    pub success: bool,
}

impl ContentData {
    pub fn new(lines: Vec<String>, title: String) -> Self {
        Self {
            lines,
            title,
            is_loading: false,
            success: true,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            lines: vec![message],
            title: "Error".to_string(),
            is_loading: false,
            success: false,
        }
    }

    pub fn loading(url: &str) -> Self {
        Self {
            lines: vec![
                "Loading content, please wait...".to_string(),
                format!("URL: {}", url),
            ],
            title: format!("Fetching: {}", url),
            is_loading: true,
            success: false,
        }
    }

    pub fn welcome() -> Self {
        Self {
            lines: vec![
                "Welcome".to_string(),
                "Press / to enter a URL and start reading.".to_string(),
                "按下 / 键 输入小说章节网址并开始阅读".to_string(),
                "Only Supported sites: https://ixdzs8.com".to_string(),
                "仅支持 https://ixdzs8.com".to_string(),
                "Example: https://ixdzs8.com/read/12345/p1.html".to_string(),
            ],
            title: "NovelTUI".to_string(),
            is_loading: false,
            success: true,
        }
    }
}

/// 输入历史记录管理
#[derive(Debug)]
pub struct InputHistory {
    entries: Vec<String>,
    current_index: i32, // -1 表示不在历史中
}

impl InputHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            current_index: -1,
        }
    }

    pub fn add(&mut self, entry: String) {
        self.entries.push(entry);
        self.current_index = -1;
    }

    pub fn navigate_up(&mut self) -> Option<&str> {
        if self.current_index < (self.entries.len() as i32 - 1) {
            self.current_index += 1;
            self.entries
                .get(self.current_index as usize)
                .map(|s| s.as_str())
        } else {
            None
        }
    }

    pub fn navigate_down(&mut self) -> Option<&str> {
        if self.current_index > -1 {
            self.current_index -= 1;
            if self.current_index == -1 {
                Some("") // 返回空字符串表示新输入
            } else {
                self.entries
                    .get(self.current_index as usize)
                    .map(|s| s.as_str())
            }
        } else {
            None
        }
    }

    pub fn reset_index(&mut self) {
        self.current_index = -1;
    }

    pub fn is_in_history(&self) -> bool {
        self.current_index != -1
    }
}

/// 应用主状态
pub struct AppState {
    pub content_state: ListState,
    pub show_input: bool,
    pub input_buffer: String,
    pub page_size: usize,
    pub show_title: bool,
    pub history: InputHistory,
    pub inc_line_space: bool,
}

impl AppState {
    pub fn new(page_size: usize) -> Self {
        let mut content_state = ListState::default();
        content_state.select(Some(0));

        Self {
            content_state,
            show_input: false,
            input_buffer: String::new(),
            page_size,
            show_title: true,
            history: InputHistory::new(),
            inc_line_space: false,
        }
    }

    pub fn start_input(&mut self) {
        self.show_input = true;
        self.input_buffer.clear();
        self.history.reset_index();
    }

    pub fn cancel_input(&mut self) {
        self.show_input = false;
        self.input_buffer.clear();
    }

    pub fn confirm_input(&mut self) -> String {
        let input = self.input_buffer.trim().to_string();
        self.show_input = false;
        self.input_buffer.clear();
        input
    }

    pub fn handle_char_input(&mut self, c: char) {
        if self.history.is_in_history() {
            self.history.reset_index();
        }
        self.input_buffer.push(c);
    }

    pub fn handle_backspace(&mut self) {
        if self.history.is_in_history() {
            self.history.reset_index();
        }
        self.input_buffer.pop();
    }

    pub fn navigate_history_up(&mut self) {
        if let Some(entry) = self.history.navigate_up() {
            self.input_buffer = entry.to_string();
        }
    }

    pub fn navigate_history_down(&mut self) {
        if let Some(entry) = self.history.navigate_down() {
            self.input_buffer = entry.to_string();
        }
    }

    pub fn move_cursor_down(&mut self, max_index: usize) {
        let current = self.content_state.selected().unwrap_or(0);
        if current < max_index.saturating_sub(1) {
            self.content_state.select(Some(current + 1));
        }
    }

    pub fn move_cursor_up(&mut self) {
        let current = self.content_state.selected().unwrap_or(0);
        if current > 0 {
            self.content_state.select(Some(current - 1));
        }
    }

    pub fn page_up(&mut self) {
        let current = self.content_state.selected().unwrap_or(0);
        if current >= self.page_size {
            self.content_state.select(Some(current - self.page_size));
        } else {
            self.content_state.select(Some(0));
        }
    }

    pub fn page_down(&mut self, max_index: usize) {
        let current = self.content_state.selected().unwrap_or(0);
        let new_pos = (current + self.page_size).min(max_index.saturating_sub(1));
        self.content_state.select(Some(new_pos));
    }

    pub fn reset_cursor(&mut self) {
        self.content_state.select(Some(0));
    }
}
