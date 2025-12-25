// src/tui/state.rs
use crate::core::novel::{BookmarkEntry, Novel};
use ratatui::widgets::ListState;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum FocusArea {
    #[default]
    Toc,
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

    // Cached Data for UI
    pub cached_bookmarks: Vec<BookmarkEntry>,

    // Current Navigation
    pub active_chapter_index: usize,

    // Delete Confirmation Dialog
    pub show_delete_confirmation: bool,
}

impl AppState {
    pub fn new(show_title: bool, speed: f64) -> Self {
        let mut toc_state = ListState::default();
        toc_state.select(Some(0));
        let mut content_state = ListState::default();
        content_state.select(Some(0));

        Self {
            running: false,
            focus: FocusArea::Toc,
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
            cached_bookmarks: Vec::new(),
            active_chapter_index: 0,
            show_delete_confirmation: false,
        }
    }

    pub fn refresh_bookmarks(&mut self, novel: &Novel) {
        self.cached_bookmarks = novel.collect_bookmarks();
        // Adjust selection if out of bounds
        if !self.cached_bookmarks.is_empty() {
            if self.bookmark_state.selected().is_none() {
                self.bookmark_state.select(Some(0));
            } else if let Some(sel) = self.bookmark_state.selected() {
                if sel >= self.cached_bookmarks.len() {
                    self.bookmark_state
                        .select(Some(self.cached_bookmarks.len() - 1));
                }
            }
        } else {
            self.bookmark_state.select(None);
        }
    }
}
