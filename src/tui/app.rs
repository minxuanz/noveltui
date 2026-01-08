// src/tui/app.rs
use color_eyre::Result;
use crossterm::event;
use ratatui::DefaultTerminal;
use std::time::{Duration, Instant};

use crate::cmd::args::Options;
use crate::core::novel::Novel;
use crate::infra::fs;
use crate::tui::inputs::Action;
use crate::tui::state::{AppState, FocusArea};
use crate::tui::{inputs, renderer};

pub struct App {
    options: Options,
    novel: Novel,
    state: AppState,
}

impl App {
    pub fn new(options: Options) -> Result<Self> {
        let lines = fs::load_content(&options.file_path)?;
        let novel = Novel::new(lines);
        let mut state = AppState::new(!options.simple_mode, options.speed, options.page_size);

        // Initial Bookmark Cache
        state.refresh_bookmarks(&novel);

        Ok(Self {
            options,
            novel,
            state,
        })
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.state.running = true;
        self.handle_initial_jumps()?;

        while self.state.running {
            if self.state.needs_reinit {
                // If a suspend/resume cycle occurred, re-initialize the terminal
                // This re-assigns the `terminal` variable that `run` holds.
                terminal = ratatui::init();
                self.state.needs_reinit = false; // Reset the flag
            }
            // Render
            terminal.draw(|f| {
                renderer::render_ui(f, &mut self.state, &self.novel, &self.options.file_path);
            })?;

            // Event Loop with Auto-Scroll timing
            self.handle_events_and_timer()?;
        }

        Ok(())
    }

    fn handle_events_and_timer(&mut self) -> Result<()> {
        let timeout = if self.state.auto_scroll && self.state.focus == FocusArea::Content {
            let elapsed = self.state.last_scroll_time.elapsed().as_millis() as u64;
            let speed = self.state.auto_scroll_speed_ms;
            if elapsed >= speed {
                Duration::ZERO
            } else {
                Duration::from_millis(speed - elapsed)
            }
        } else {
            Duration::from_millis(100)
        };

        if event::poll(timeout)? {
            let action = inputs::resolve_event(event::read()?, &self.state);
            self.process_action(action)?;
        }

        // Auto Scroll Logic
        if self.state.auto_scroll && self.state.focus == FocusArea::Content {
            let now = Instant::now();
            if now.duration_since(self.state.last_scroll_time).as_millis() as u64
                >= self.state.auto_scroll_speed_ms
            {
                self.move_cursor_down();
                self.state.last_scroll_time = now;
            }
        }
        Ok(())
    }

    fn process_action(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.state.running = false,
            Action::SaveAndQuit => {
                // Ensure current modification is saved, though we modify memory directly in toggle
                // Just need to flush to disk.
                fs::save_content(&self.options.file_path, &self.novel.lines)?;
                self.state.running = false;
            }
            Action::Suspend => self.suspend()?,
            Action::ToggleBookmarkMenu => {
                self.state.show_bookmark_menu = !self.state.show_bookmark_menu;
                self.state.focus = if self.state.show_bookmark_menu {
                    FocusArea::Bookmark
                } else {
                    FocusArea::Content
                };
            }
            Action::ToggleTocMenu => self.state.show_toc_menu = !self.state.show_toc_menu,
            Action::ToggleTitleFooter => self.state.show_title = !self.state.show_title,
            Action::ToggleBookmarkAtCursor => self.toggle_bookmark(),
            Action::ClearAllBookmarks => self.state.show_delete_confirmation = true,
            Action::ToggleHelp => self.state.show_help = !self.state.show_help,
            Action::ConfirmDelete => {
                if self.state.show_delete_confirmation {
                    self.clear_all_bookmarks();
                    self.state.show_delete_confirmation = false;
                }
            }
            Action::CancelDelete => {
                self.state.show_delete_confirmation = false;
            }

            Action::FocusLeft => self.switch_focus(true),
            Action::FocusRight => self.switch_focus(false),

            Action::MoveUp => self.move_cursor_up(),
            Action::MoveDown => self.move_cursor_down(),

            Action::NextChapter => {
                let curr = self.state.active_chapter_index;
                if curr + 1 < self.novel.chapters.len() {
                    self.select_chapter(curr + 1);
                }
            }
            Action::PrevChapter => {
                let curr = self.state.active_chapter_index;
                if curr > 0 {
                    self.select_chapter(curr - 1);
                }
            }

            Action::Enter => self.on_enter(),
            Action::AutoScroll => self.state.auto_scroll = !self.state.auto_scroll,
            Action::PageUp => {
                let curr = self.state.content_state.selected().unwrap_or(0);
                let page_size = self.state.page_size;
                if curr >= page_size {
                    self.state.content_state.select(Some(curr - page_size));
                } else {
                    self.state.content_state.select(Some(0));
                }
            }
            Action::PageDown => {
                let curr = self.state.content_state.selected().unwrap_or(0);
                let page_size = self.state.page_size;
                self.state.content_state.select(Some(curr + page_size));
            }
            Action::None => {}
        }
        Ok(())
    }

    // --- Navigation Logic ---

    fn move_cursor_up(&mut self) {
        match self.state.focus {
            FocusArea::Toc => {
                let curr = self.state.toc_state.selected().unwrap_or(0);
                if curr > 0 {
                    self.state.toc_state.select(Some(curr - 1));
                }
            }
            FocusArea::Content => {
                let curr = self.state.content_state.selected().unwrap_or(0);
                if curr > 0 {
                    self.state.content_state.select(Some(curr - 1));
                } else if self.state.active_chapter_index > 0 {
                    // Go to previous chapter, last line
                    self.select_chapter(self.state.active_chapter_index - 1);
                    let len = self
                        .novel
                        .get_chapter_lines(self.state.active_chapter_index)
                        .map(|l| l.len())
                        .unwrap_or(0);
                    if len > 0 {
                        self.state.content_state.select(Some(len - 1));
                    }
                }
            }
            FocusArea::Bookmark => {
                let curr = self.state.bookmark_state.selected().unwrap_or(0);
                if curr > 0 {
                    self.state.bookmark_state.select(Some(curr - 1));
                    self.jump_to_bookmark();
                }
            }
        }
    }

    fn move_cursor_down(&mut self) {
        match self.state.focus {
            FocusArea::Toc => {
                let curr = self.state.toc_state.selected().unwrap_or(0);
                if curr + 1 < self.novel.chapters.len() {
                    self.state.toc_state.select(Some(curr + 1));
                    // self.select_chapter(curr + 1);
                }
            }
            FocusArea::Content => {
                let curr = self.state.content_state.selected().unwrap_or(0);
                let lines_len = self
                    .novel
                    .get_chapter_lines(self.state.active_chapter_index)
                    .map(|l| l.len())
                    .unwrap_or(0);

                if curr + 1 < lines_len {
                    self.state.content_state.select(Some(curr + 1));
                } else if self.state.active_chapter_index + 1 < self.novel.chapters.len() {
                    // Next chapter
                    self.select_chapter(self.state.active_chapter_index + 1);
                }
            }
            FocusArea::Bookmark => {
                let curr = self.state.bookmark_state.selected().unwrap_or(0);
                if curr + 1 < self.state.cached_bookmarks.len() {
                    self.state.bookmark_state.select(Some(curr + 1));
                    self.jump_to_bookmark();
                }
            }
        }
    }

    fn select_chapter(&mut self, idx: usize) {
        self.state.active_chapter_index = idx;
        self.state.toc_state.select(Some(idx));
        self.state.content_state.select(Some(0));
    }

    fn switch_focus(&mut self, left: bool) {
        use FocusArea::*;
        self.state.focus = match (self.state.focus, left) {
            (Toc, false) => Content,
            (Content, true) if self.state.show_toc_menu => {
                self.state.toc_state.select(Some(self.state.active_chapter_index));
                Toc
            },
            (Content, false) if self.state.show_bookmark_menu => Bookmark,
            (Bookmark, true) => Content,
            // Cycles or stays
            (s, _) => s,
        };
    }

    fn on_enter(&mut self) {
        if self.state.focus == FocusArea::Toc {
            self.state.active_chapter_index = self
                .state
                .toc_state
                .selected()
                .unwrap_or(self.state.active_chapter_index);
            self.state.content_state.select(Some(0));
            self.state.focus = FocusArea::Content;
        }
    }

    // --- Bookmark Logic ---

    fn toggle_bookmark(&mut self) {
        if self.state.focus == FocusArea::Toc {
            return;
        }

        let chapter_idx = self.state.active_chapter_index;
        let line_in_view = self.state.content_state.selected().unwrap_or(0);

        if let Some(meta) = self.novel.chapters.get(chapter_idx) {
            let global_idx = meta.range.start + line_in_view;
            if self.novel.toggle_bookmark(global_idx) {
                // Update cache immediately
                self.state.refresh_bookmarks(&self.novel);
                // Save best effort
                let _ = fs::save_content(&self.options.file_path, &self.novel.lines);
            }
        }
    }

    fn clear_all_bookmarks(&mut self) {
        let indices: Vec<usize> = self
            .state
            .cached_bookmarks
            .iter()
            .map(|b| b.global_index)
            .collect();
        for idx in indices {
            self.novel.remove_bookmark(idx);
        }
        self.state.refresh_bookmarks(&self.novel);
        let _ = fs::save_content(&self.options.file_path, &self.novel.lines);
    }

    fn jump_to_bookmark(&mut self) {
        if let Some(idx) = self.state.bookmark_state.selected() {
            if let Some(bm) = self.state.cached_bookmarks.get(idx) {
                let chapter_idx = bm.chapter_index;
                let line_in_chapter = bm.line_in_chapter;
                self.select_chapter(chapter_idx);
                self.state.content_state.select(Some(line_in_chapter));
            }
        }
    }

    // --- Initial Jumps / Suspend ---

    fn handle_initial_jumps(&mut self) -> Result<()> {
        if let Some(ch) = self.options.chapter {
            if ch > 0 && ch <= self.novel.chapters.len() {
                self.select_chapter(ch);
                self.state.focus = FocusArea::Content;
            }
        } else if let Some(bm_num) = self.options.bookmark {
            if bm_num > 0 && bm_num <= self.state.cached_bookmarks.len() {
                self.state.bookmark_state.select(Some(bm_num - 1));
                self.jump_to_bookmark();
                self.state.focus = FocusArea::Content;
            }
        } else {
            // Default to last bookmarks
            if !self.state.cached_bookmarks.is_empty() {
                let last_idx = self.state.cached_bookmarks.len() - 1;
                self.state.bookmark_state.select(Some(last_idx));
                self.jump_to_bookmark();
                self.state.focus = FocusArea::Content;
            }
        }
        Ok(())
    }

    fn suspend(&mut self) -> Result<()> {
        #[cfg(unix)]
        {
            use crossterm::cursor::Show;
            use crossterm::terminal::LeaveAlternateScreen;
            // Restore terminal before suspending
            ratatui::restore();
            // show cursor
            crossterm::execute!(std::io::stdout(), Show, LeaveAlternateScreen)?;
            // simple shell suspend
            use signal_hook::consts::signal::SIGTSTP;
            use signal_hook::low_level::raise;
            raise(SIGTSTP).unwrap();

            // Resume
            self.state.needs_reinit = true;
        }
        Ok(())
    }
}
