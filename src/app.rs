use std::fs;
use std::path::PathBuf;

use crate::args::Options;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::*,
};

use crate::chapter::{self, Chapter}; // added

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Focus {
    Toc,
    Content,
}

impl Default for Focus {
    fn default() -> Self {
        Focus::Toc
    }
}

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    file_path: PathBuf,
    lines: Vec<String>,
    // offset is now relative to view_lines (keeps paragraph scroll if you keep paragraph; currently not used)
    view_offset: usize,
    // TOC
    chapters: Vec<Chapter>,
    list_state: ListState,
    // current view (either selected chapter or whole file)
    view_lines: Vec<String>,

    // new: content selection state + focus
    content_state: ListState,
    focus: Focus,
}

impl App {
    pub fn new(args: Options) -> Self {
        let mut ls = ListState::default();
        ls.select(Some(0));
        let mut content_state = ListState::default();
        content_state.select(Some(0));
        Self {
            running: false,
            file_path: args.file_path,
            lines: Vec::new(),
            view_offset: 0,
            chapters: Vec::new(),
            list_state: ls,
            view_lines: Vec::new(),
            content_state,
            focus: Focus::Toc,
        }
    }

    fn load_file(&mut self) -> Result<()> {
        // load file content into self.lines
        let content = fs::read_to_string(&self.file_path)?;
        self.lines = content.lines().map(|s| s.to_string()).collect();
        // parse chapters from lines
        self.chapters = chapter::parse_lines(&self.lines);
        // set initial view: first chapter if exists, else whole file
        if !self.chapters.is_empty() {
            self.list_state.select(Some(0));
            self.select_chapter(0);
        } else {
            self.list_state.select(None);
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

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.load_file()?;
        self.running = true;
        while self.running {
            terminal.draw(|f| {
                self.render(f);
            })?;
            self.handle_crossterm_event();
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = self.get_layout_chunks(frame.area());
        self.render_title(frame, chunks[0]);

        // middle area: split into left TOC and right content
        let middle = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(20), Constraint::Min(1)].as_ref())
            .split(chunks[1]);

        self.render_toc(frame, middle[0]);
        self.render_content(frame, middle[1]);

        self.render_footer(frame, chunks[2]);
    }

    fn get_layout_chunks(&self, area: Rect) -> Vec<Rect> {
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

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title_text = self.file_path.to_str().unwrap_or("NovelTUI");
        let p = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(p, area);
    }

    fn render_content(&mut self, frame: &mut Frame, area: Rect) {
        // render content as a stateful List so we can highlight current line
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title("Content");

        let items: Vec<ListItem> = if !self.view_lines.is_empty() {
            self.view_lines.iter().map(|line| ListItem::new(line.clone())).collect()
        } else {
            vec![ListItem::new("") ]
        };

        // highlight style depends on focus
        let highlight_style = if self.focus == Focus::Content {
            Style::default().fg(Color::Black).bg(Color::White).add_modifier(Modifier::BOLD)
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
            self.chapters.iter().map(|c| {
                ListItem::new(c.title.clone())
            }).collect()
        } else {
            vec![ListItem::new("NONE")]
        };

        let toc_highlight = if self.focus == Focus::Toc {
            Style::default().fg(Color::Black).bg(Color::LightGreen).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).title("TOC"))
            .highlight_style(toc_highlight);

        frame.render_stateful_widget(list, area, &mut self.list_state);
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
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
            .split(inner);

        let chapter_info = if !self.chapters.is_empty() {
            if let Some(sel) = self.list_state.selected() {
                self.chapters.get(sel).map(|c| c.title.clone()).unwrap_or_else(|| {
                    self.file_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string()
                })
            } else {
                self.file_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string()
            }
        } else {
            self.file_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string()
        };

        let focus_label = match self.focus {
            Focus::Toc => "[TOC]",
            Focus::Content => "[CONTENT]",
        };

        let left = Paragraph::new(format!("{} {}", focus_label, chapter_info))
            .alignment(Alignment::Left)
            .style(Style::default().fg(Color::LightCyan));
        frame.render_widget(left, cols[0]);

        let hints = "[q] Quit   [h/←] Focus TOC   [l/→] Focus Content   [j/↓] Down   [k/↑] Up   [Enter] Jump";
        let right = Paragraph::new(hints)
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::White));
        frame.render_widget(right, cols[1]);
    }

    // helper to change selected chapter and update view
    fn select_chapter(&mut self, idx: usize) {
        if let Some(ch) = self.chapters.get(idx) {
            self.view_lines = ch.content.clone();
            self.view_offset = 0;
            self.list_state.select(Some(idx));
            // reset content cursor
            if !self.view_lines.is_empty() {
                self.content_state.select(Some(0));
            } else {
                self.content_state.select(None);
            }
        }
    }

    fn handle_crossterm_event(&mut self) {
        // handle crossterm events here
        match event::read() {
            Ok(Event::Key(key_event)) => match key_event.code {
                KeyCode::Char('q') => self.running = false,

                // focus switches
                KeyCode::Char('h') | KeyCode::Left => {
                    self.focus = Focus::Toc;
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    self.focus = Focus::Content;
                }

                // unified movement: Up/k and Down/j behave the same, target depends on focus
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.focus == Focus::Toc {
                        if let Some(selected) = self.list_state.selected() {
                            if selected > 0 {
                                let new = selected - 1;
                                self.select_chapter(new);
                            }
                        }
                    } else {
                        // content focus
                        if let Some(sel) = self.content_state.selected() {
                            if sel > 0 {
                                self.content_state.select(Some(sel - 1));
                            }
                        } else if !self.view_lines.is_empty() {
                            self.content_state.select(Some(0));
                        }
                    }
                }

                KeyCode::Down | KeyCode::Char('j') => {
                    if self.focus == Focus::Toc {
                        if let Some(selected) = self.list_state.selected() {
                            if selected + 1 < self.chapters.len() {
                                let new = selected + 1;
                                self.select_chapter(new);
                            }
                        } else if !self.chapters.is_empty() {
                            self.select_chapter(0);
                        }
                    } else {
                        // content focus
                        if let Some(sel) = self.content_state.selected() {
                            if sel + 1 < self.view_lines.len() {
                                self.content_state.select(Some(sel + 1));
                            }
                        } else if !self.view_lines.is_empty() {
                            self.content_state.select(Some(0));
                        }
                    }
                }

                KeyCode::Enter => {
                    if self.focus == Focus::Toc {
                        if let Some(idx) = self.list_state.selected() {
                            self.select_chapter(idx);
                            // move focus to content after jump
                            self.focus = Focus::Content;
                        }
                    } else {
                        // future: do something with content Enter
                    }
                }

                _ => {}
            },
            _ => {}
        }
    }

}
