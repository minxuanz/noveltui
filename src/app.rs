use std::fs;
use std::path::PathBuf;

use crate::args::Options;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Text},
    widgets::*,
};

#[derive(Debug, Default)]
pub struct App {
    running: bool,
    file_path: PathBuf,
    lines: Vec<String>,
    offset: usize,
}

impl App {
    pub fn new(args: Options) -> Self {
        Self {
            running: false,
            file_path: args.file_path,
            lines: Vec::new(),
            offset: 0,
        }
    }

    fn load_file(&mut self) -> Result<()> {
        // load file content into self.lines
        let content = fs::read_to_string(&self.file_path);
        self.lines = content?.lines().map(|s| s.to_string()).collect();
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
        self.render_content(frame, chunks[1]);
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
        let title = Paragraph::new(self.file_path.to_str().unwrap_or("NovelTUI"))
            .style(Style::default().fg(Color::White));
        frame.render_widget(title, area);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);
        let text: Vec<Line> = self.lines.iter().map(|line| Line::from(line.as_str())).collect();
        let p = Paragraph::new(Text::from(text))
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((self.offset as u16, 0));
        frame.render_widget(p, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let hint = " quit (q) | next (n) | prev (p) ";
        let footer = Paragraph::new(hint)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true })
            .style(Style::default());
        frame.render_widget(footer, area);
    }

    fn handle_crossterm_event(&mut self) {
        // handle crossterm events here
        match event::read() {
            Ok(Event::Key(key_event)) => match key_event.code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('j') => self.scroll_down(),
                KeyCode::Char('k') => self.scroll_up(),
                _ => {}
            },
            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        if self.offset < self.lines.len() - 1 {
            self.offset += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.offset > 0 {
            self.offset -= 1;
        }
    }
}
