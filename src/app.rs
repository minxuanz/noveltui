use std::fs;
use std::path::PathBuf;

use crate::args::Options;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::*,
};

use crate::bookmark::{self, BOOKMARK_SYMBOL, Bookmark};
use crate::chapter::{self, Chapter};
use chardetng::EncodingDetector;
use textwrap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Focus {
    #[default]
    Toc,
    Content,
    Bookmark,
}

#[derive(Debug, Default)]
pub struct App {
    // state
    running: bool,
    // file path
    file_path: PathBuf,
    // full file content
    lines: Vec<String>,
    // offset is now relative to view_lines
    view_offset: usize,
    // TOC
    chapters: Vec<Chapter>,
    // TOC selection state
    toc_state: ListState,
    // current view (either selected chapter or whole file)
    view_lines: Vec<String>,
    // new: content selection state + focus
    content_state: ListState,
    // current focus
    focus: Focus,
    // bookmarks
    bookmarks: Vec<Bookmark>,
    // bookmark selection state
    bookmark_state: ListState,
    // whether to show bookmark menu
    show_bookmark_menu: bool,
    // whether to show title and footer
    show_tilte_footer: bool,
    // initial jump targets
    initial_bookmark_jump: Option<usize>,
    // new: initial chapter jump target
    initial_chapter_jump: Option<usize>,
}

impl App {
    pub fn new(args: Options) -> Self {
        // Change signature to accept Options
        let mut toc_state = ListState::default();
        toc_state.select(Some(0));
        let mut content_state = ListState::default();
        content_state.select(Some(0));
        Self {
            running: false,
            file_path: args.file_path,
            lines: Vec::new(),
            view_offset: 0,
            chapters: Vec::new(),
            toc_state,
            view_lines: Vec::new(),
            content_state,
            focus: Focus::Toc,
            bookmarks: Vec::new(),
            bookmark_state: ListState::default(),
            show_bookmark_menu: false,
            show_tilte_footer: true,
            initial_bookmark_jump: args.bookmark, // Store the bookmark index
            initial_chapter_jump: args.chapter,   // Store the chapter index
        }
    }

    fn load_file(&mut self) -> Result<()> {
        // Try reading as UTF-8 first
        let content = match fs::read_to_string(&self.file_path) {
            Ok(s) => s,
            Err(_) => {
                // Fallback: read bytes and auto-detect encoding then decode
                let bytes = fs::read(&self.file_path)?;
                Self::decode_with_auto_detect(&bytes)
            }
        };
        self.lines = content.lines().map(|s| s.to_string()).collect();
        // parse chapters from lines
        self.chapters = chapter::parse_lines(&self.lines);
        self.bookmarks = bookmark::parse_bookmarks(&self.chapters);
        if !self.bookmarks.is_empty() {
            self.bookmark_state.select(Some(0));
        }
        // set initial view: first chapter if exists, else whole file
        if !self.chapters.is_empty() {
            self.toc_state.select(Some(0));
            self.select_chapter(0);
        } else {
            self.toc_state.select(None);
            self.view_lines = self.lines.clone();
            self.view_offset = 0;
            if !self.view_lines.is_empty() {
                self.content_state.select(Some(0));
            } else {
                self.content_state.select(None);
            }
        }
        Ok(())
    }

    // helper
    fn decode_with_auto_detect(bytes: &[u8]) -> String {
        let mut det = EncodingDetector::new();
        det.feed(bytes, true);
        let encoding = det.guess(None, true);
        let (cow, _, _) = encoding.decode(bytes);
        cow.into_owned()
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.load_file()?;
        self.running = true;

        // Call the new method to handle initial jumps
        self.handle_initial_jumps()?; // New line: call the extracted logic

        while self.running {
            terminal.draw(|f| {
                self.render(f);
            })?;
            self.handle_crossterm_event();
        }
        Ok(())
    }

    // New private method to handle initial chapter and bookmark jumps
    fn handle_initial_jumps(&mut self) -> Result<()> {
        match (
            self.initial_chapter_jump.take(),
            self.initial_bookmark_jump.take(),
        ) {
            (Some(chapter_idx), None) => {
                // 处理章节跳转
                if chapter_idx < self.chapters.len() {
                    self.select_chapter(chapter_idx.saturating_sub(1));
                    self.focus = Focus::Content; // 跳转后聚焦内容区
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "Only have {} Chapter(s). Cannot jump to chapter {}.",
                        self.chapters.len(),
                        chapter_idx
                    ));
                }
            }
            (None, Some(bookmark_idx)) => {
                // 处理书签跳转
                if bookmark_idx < self.bookmarks.len() {
                    self.bookmark_state
                        .select(Some(bookmark_idx.saturating_sub(1)));
                    self.jump_to_selected_bookmark();
                    self.focus = Focus::Content; // 跳转后聚焦内容区
                } else {
                    return Err(color_eyre::eyre::eyre!(
                        "Only have {} Bookmark(s). Cannot jump to bookmark {}.",
                        self.bookmarks.len(),
                        bookmark_idx
                    ));
                }
            }
            (None, None) => {}
            (Some(_), Some(_)) => {
                unreachable!()
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = self.get_layout_chunks(frame.area());
        if self.show_tilte_footer {
            self.render_title(frame, chunks[0]);
        }

        let index = if self.show_tilte_footer { 1 } else { 0 };
        // middle area: split into left TOC, content, and optionally bookmark
        let middle_chunks = if self.show_bookmark_menu {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(20),
                    Constraint::Min(1),
                    Constraint::Length(20),
                ])
                .split(chunks[index])
        } else {
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(20), Constraint::Min(1)])
                .split(chunks[index])
        };

        self.render_toc(frame, middle_chunks[0]);
        self.render_content(frame, middle_chunks[1]);

        if self.show_bookmark_menu {
            self.render_bookmark_menu(frame, middle_chunks[2]);
        }

        if self.show_tilte_footer {
            self.render_footer(frame, chunks[2]);
        }
    }

    fn get_layout_chunks(&self, area: Rect) -> Vec<Rect> {
        if !self.show_tilte_footer {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1)].as_ref())
                .split(area)
                .to_vec()
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Min(1),
                        Constraint::Length(1),
                    ]
                    .as_ref(),
                )
                .split(area)
                .to_vec()
        }
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title_text = self.file_path.to_str().unwrap_or("NovelTUI");
        let p = Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        // render content as a stateful List so we can highlight current line
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Content");

        // compute available inner width for wrapping (leave 5 for borders)
        let inner_width = area.width.saturating_sub(5) as usize;
        let wrap_width = if inner_width == 0 { 1 } else { inner_width };

        let items: Vec<ListItem> = if !self.view_lines.is_empty() {
            self.view_lines
                .iter()
                .map(|line| {
                    // wrap the logical line into visual lines
                    let wrapped = textwrap::wrap(line, wrap_width);
                    let joined = if wrapped.is_empty() {
                        "".to_string()
                    } else {
                        wrapped
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<_>>()
                            .join("\n")
                    };
                    ListItem::new(Text::from(joined))
                })
                .collect()
        } else {
            vec![ListItem::new("")]
        };

        // highlight style depends on focus
        let highlight_style = if self.focus != Focus::Toc {
            Style::default()
                .fg(Color::Black)
                .bg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        // render stateful so content_state keeps the selected line across frames
        frame.render_stateful_widget(list, area, &mut self.content_state);
    }

    // new: render TOC (目录)
    fn render_toc(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = if !self.chapters.is_empty() {
            self.chapters
                .iter()
                .map(|c| ListItem::new(c.title.clone()))
                .collect()
        } else {
            vec![ListItem::new("NONE")]
        };

        let toc_highlight = {
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title("TOC"),
            )
            .highlight_style(toc_highlight);

        frame.render_stateful_widget(list, area, &mut self.toc_state);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        // render outer border for footer
        let block = Block::default()
            .borders(Borders::NONE)
            .border_type(BorderType::Rounded);
        frame.render_widget(block, area);

        // inner rect to avoid overlapping border (leave 1 cell margin)
        let inner = Rect {
            x: area.x.saturating_add(1),
            y: area.y.saturating_add(0),
            width: area.width.saturating_sub(2),
            height: area.height,
        };

        // split into left (chapter info) and right (hints)
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(inner);

        let chapter_info = if !self.chapters.is_empty() {
            if let Some(sel) = self.toc_state.selected() {
                self.chapters
                    .get(sel)
                    .map(|c| c.title.clone())
                    .unwrap_or_else(|| {
                        self.file_path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string()
                    })
            } else {
                self.file_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string()
            }
        } else {
            self.file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        };

        let focus_label = match self.focus {
            Focus::Toc => "[TOC]",
            Focus::Content => "[CONTENT]",
            Focus::Bookmark => "[BOOKMARK]",
        };

        let left = Paragraph::new(format!("{} {}", focus_label, chapter_info))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::LightCyan));
        frame.render_widget(left, cols[0]);

        let total_lines = self.lines.len();
        let _view_lines = self.view_lines.len();
        let selected_line_in_view = match self.content_state.selected() {
            Some(idx) => idx + 1,
            None => 0,
        };
        let global_line_number = self
            .chapters
            .get(self.toc_state.selected().unwrap_or(0))
            .map_or(0, |chapter| chapter.start_line + selected_line_in_view);
        let progress_indicator = format!(
            "{}/{} [m]Toggle Mark [b]Bookmark",
            global_line_number, total_lines
        );
        //let hints = "[q]Quit [b]Bookmark [m]Toggle Mark | [h/←]Left [l/→]Right | [j/↓]Down [k/↑]Up";
        let right = Paragraph::new(progress_indicator)
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::White));
        frame.render_widget(right, cols[1]);
    }

    // helper to change selected chapter and update view
    fn select_chapter(&mut self, idx: usize) {
        if let Some(ch) = self.chapters.get(idx) {
            self.view_lines = ch.content.clone();
            self.view_offset = 0;
            self.toc_state.select(Some(idx));
            // reset content cursor
            if !self.view_lines.is_empty() {
                self.content_state.select(Some(0));
            } else {
                self.content_state.select(None);
            }
        }
    }

    fn handle_crossterm_event(&mut self) {
        match event::read() {
            Ok(Event::Key(key_event)) => match key_event.code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.running = false
                }
                KeyCode::Char('b') => self.toggle_bookmark_menu(),
                KeyCode::Char('s') => self.show_tilte_footer = !self.show_tilte_footer,
                KeyCode::Char('m') => self.toggle_bookmark_at_current_line(),
                KeyCode::Char('h') | KeyCode::Left => self.switch_focus_left(),
                KeyCode::Char('l') | KeyCode::Right => self.switch_focus_right(),
                KeyCode::Char('k') | KeyCode::Up => self.handle_move_up(),
                KeyCode::Char('j') | KeyCode::Down => self.handle_move_down(),
                KeyCode::Enter => self.handle_enter(),
                _ => {}
            },
            _ => {}
        }
    }

    fn switch_focus_left(&mut self) {
        self.focus = match self.focus {
            Focus::Bookmark => Focus::Content,
            Focus::Content => Focus::Toc,
            Focus::Toc => {
                if self.show_bookmark_menu {
                    Focus::Bookmark
                } else {
                    Focus::Content
                }
            }
        };
    }

    fn switch_focus_right(&mut self) {
        self.focus = match self.focus {
            Focus::Toc => Focus::Content,
            // if bookmark menu is shown, switch to it
            // if not,go to TOC
            Focus::Content => {
                if self.show_bookmark_menu {
                    self.jump_to_selected_bookmark();
                    Focus::Bookmark
                } else {
                    Focus::Toc
                }
            }
            Focus::Bookmark => Focus::Toc,
        };
    }

    fn handle_move_up(&mut self) {
        match self.focus {
            Focus::Toc => self.move_toc_up(),
            Focus::Content => self.move_content_up(),
            Focus::Bookmark => self.move_bookmark_up(),
        }
    }

    fn handle_move_down(&mut self) {
        match self.focus {
            Focus::Toc => self.move_toc_down(),
            Focus::Content => self.move_content_down(),
            Focus::Bookmark => self.move_bookmark_down(),
        }
    }

    fn move_toc_up(&mut self) {
        if let Some(selected) = self.toc_state.selected() {
            if selected > 0 {
                self.select_chapter(selected - 1);
            } else if !self.chapters.is_empty() {
                self.select_chapter(self.chapters.len() - 1);
            }
        }
    }

    fn move_toc_down(&mut self) {
        if let Some(selected) = self.toc_state.selected() {
            if selected + 1 < self.chapters.len() {
                self.select_chapter(selected + 1);
            } else if !self.chapters.is_empty() {
                self.select_chapter(0);
            }
        }
    }

    fn move_content_up(&mut self) {
        if let Some(sel) = self.content_state.selected() {
            if sel > 0 {
                self.content_state.select(Some(sel - 1));
            } else if let Some(toc_sel) = self.toc_state.selected() {
                // jump to previous chapter
                if toc_sel > 0 {
                    self.select_chapter(toc_sel - 1);
                    if !self.view_lines.is_empty() {
                        // select last line of new chapter
                        self.content_state.select(Some(self.view_lines.len() - 1));
                    }
                }
            }
        } else if !self.view_lines.is_empty() {
            self.content_state.select(Some(0));
        }
    }

    fn move_content_down(&mut self) {
        if let Some(sel) = self.content_state.selected() {
            if sel + 1 < self.view_lines.len() {
                self.content_state.select(Some(sel + 1));
            } else if let Some(toc_sel) = self.toc_state.selected() {
                if toc_sel + 1 < self.chapters.len() {
                    self.select_chapter(toc_sel + 1);
                }
            }
        } else if !self.view_lines.is_empty() {
            self.content_state.select(Some(0));
        }
    }

    fn handle_enter(&mut self) {
        if self.focus == Focus::Toc {
            if let Some(idx) = self.toc_state.selected() {
                self.select_chapter(idx);
                self.focus = Focus::Content;
            }
        }
    }

    fn toggle_bookmark_menu(&mut self) {
        self.show_bookmark_menu = !self.show_bookmark_menu;
        if self.show_bookmark_menu {
            self.focus = Focus::Bookmark;
        } else {
            // Revert focus to content when hiding
            self.focus = Focus::Content;
        }
    }

    fn render_bookmark_menu(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .bookmarks
            .iter()
            .enumerate() // Add enumerate to get the index
            .map(|(i, b)| {
                ListItem::new(format!("{}. {}", i + 1, b.line_content.clone())) // Prepend with index
            })
            .collect();
        let highlight_style = if self.focus == Focus::Bookmark {
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::DarkGray)
        } else {
            Style::default().fg(Color::Gray)
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Bookmarks")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_style(highlight_style);

        frame.render_stateful_widget(list, area, &mut self.bookmark_state);
    }

    fn toggle_bookmark_at_current_line(&mut self) {
        if self.focus != Focus::Content {
            return;
        }

        if let (Some(chapter_idx), Some(line_idx_in_view)) =
            (self.toc_state.selected(), self.content_state.selected())
        {
            // Get immutable info before mutable borrow
            let chapter_start_line = match self.chapters.get(chapter_idx) {
                Some(c) => c.start_line,
                None => return, // Chapter not found, should not happen
            };

            if let Some(chapter) = self.chapters.get_mut(chapter_idx) {
                if let Some(line) = chapter.content.get_mut(line_idx_in_view) {
                    if line.trim().is_empty() {
                        return; // Don't bookmark empty lines
                    }

                    if line.trim().ends_with(BOOKMARK_SYMBOL) {
                        // Bookmarked: remove the symbol from the end
                        if let Some(pos) = line.rfind(BOOKMARK_SYMBOL) {
                            let new_line = &line[..pos];
                            *line = new_line.trim_end().to_string();
                        }
                    } else {
                        // Not bookmarked: add symbol to the end
                        *line = line.trim_end().to_string();
                        line.push_str(&format!(" {}", BOOKMARK_SYMBOL));
                    }

                    if let Some(view_line) = self.view_lines.get_mut(line_idx_in_view) {
                        *view_line = line.clone();
                    }

                    // Update the line in the full file content (self.lines)
                    let global_line_idx = chapter_start_line + line_idx_in_view;
                    if let Some(global_line) = self.lines.get_mut(global_line_idx) {
                        *global_line = line.clone();
                    }

                    // Re-parse bookmarks
                    self.bookmarks = bookmark::parse_bookmarks(&self.chapters);
                    if self.bookmark_state.selected().is_none() && !self.bookmarks.is_empty() {
                        self.bookmark_state.select(Some(0));
                    } else if let Some(selected) = self.bookmark_state.selected() {
                        if selected >= self.bookmarks.len() {
                            self.bookmark_state.select(if self.bookmarks.is_empty() {
                                None
                            } else {
                                Some(self.bookmarks.len() - 1)
                            });
                        }
                    }

                    // Persist changes to disk
                    if self.save_file().is_err() {
                        // In a real app, we'd want to handle this error, maybe show a message
                        // pritnf error to stderr
                        eprintln!("Error saving file after toggling bookmark.");
                    }
                }
            }
        }
    }

    fn move_bookmark_up(&mut self) {
        if let Some(selected) = self.bookmark_state.selected() {
            if selected > 0 {
                self.bookmark_state.select(Some(selected - 1));
                self.jump_to_selected_bookmark();
            }
        }
    }

    fn move_bookmark_down(&mut self) {
        if let Some(selected) = self.bookmark_state.selected() {
            if selected + 1 < self.bookmarks.len() {
                self.bookmark_state.select(Some(selected + 1));
                self.jump_to_selected_bookmark();
            }
        }
    }

    fn jump_to_selected_bookmark(&mut self) {
        if let Some(selected) = self.bookmark_state.selected() {
            if let Some(bookmark) = self.bookmarks.get(selected).cloned() {
                self.select_chapter(bookmark.chapter_index);
                self.content_state.select(Some(bookmark.line_in_chapter));
            }
        }
    }

    fn save_file(&self) -> Result<()> {
        let content = self.lines.join("\n");
        fs::write(&self.file_path, content)?;
        Ok(())
    }
}
