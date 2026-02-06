// src/tui/state.rs
use crate::core::novel::Novel;
use ratatui::widgets::ListState;
use noveltui_theme::ThemeColors;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum FocusArea {
    Toc,
    #[default]
    Content,
    Bookmark,
}

/// Holds the runtime state of the UI (selections, focus, flags).
pub struct AppState {
    pub running: bool,
    pub focus: FocusArea,

    // UI List States
    pub toc_state: ListState,
    pub content_state: ListState,
    pub bookmark_state: ListState,

    // Display Flags
    pub show_bookmark_menu: bool,
    pub show_toc_menu: bool,
    pub show_title: bool,
    pub show_help: bool,

    // Auto Scroll
    pub auto_scroll: bool,
    pub auto_scroll_speed_ms: u64,
    pub last_scroll_time: std::time::Instant,

    // Current Navigation
    pub active_chapter_index: usize,

    // Delete Confirmation Dialog
    pub show_delete_confirmation: bool,

    // page size for content display
    pub page_size: usize,

    // flag to indicate terminal needs re-initialization
    pub needs_reinit: bool,

    // whether remove bookmark
    pub should_remove_bookmark: bool,

    // whether increase line space
    pub inc_line_space: bool,

    // Theme colors
    pub theme_colors: ThemeColors,
}

impl AppState {
    pub fn new(show_title: bool, speed: f64, page_size: usize, theme_colors: ThemeColors) -> Self {
        let mut toc_state = ListState::default();
        toc_state.select(Some(0));
        let mut content_state = ListState::default();
        content_state.select(Some(0));

        Self {
            running: false,
            focus: FocusArea::Content,
            toc_state,
            content_state,
            bookmark_state: ListState::default(),
            show_bookmark_menu: false,
            show_toc_menu: false,
            show_title,
            show_help: false,
            auto_scroll: false,
            auto_scroll_speed_ms: (speed * 1000.0) as u64,
            last_scroll_time: std::time::Instant::now(),
            active_chapter_index: 0,
            show_delete_confirmation: false,
            page_size,
            needs_reinit: false,
            should_remove_bookmark: true,
            inc_line_space: false,
            theme_colors,
        }
    }

    pub fn refresh_bookmarks(&mut self, novel: &Novel) {
        let bookmarks = novel.collect_bookmarks();
        let len = bookmarks.len();
        if len > 0 {
            if self.bookmark_state.selected().is_none() {
                self.bookmark_state.select(Some(0));
            } else if let Some(sel) = self.bookmark_state.selected() {
                if sel >= len {
                    self.bookmark_state.select(Some(len - 1));
                }
            }
        } else {
            self.bookmark_state.select(None);
        }
    }
}
