use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::*,
};

#[derive(Debug, Default)]
pub struct App {
    running: bool,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
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
        // render a Hello World paragraph with a footer showing key hints
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)].as_ref())
            .split(size);

        let contex:&str = read_from_file(); 

        let block = Block::default()
            .title("Ratatui Demo")
            .borders(Borders::NONE);
        let p = Paragraph::new("Hello, TUI!")
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(p, chunks[0]);

        let hint: &str = " n: next    p: prev    q: quit ";
        let footer = Paragraph::new(hint)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        // you can add styling here if desired (e.g. reverse/background)
        frame.render_widget(footer, chunks[1]);
    }

    fn handle_crossterm_event(&mut self) {
        // handle crossterm events here
        match event::read() {
            Ok(Event::Key(key_event)) => {
                if key_event.code == KeyCode::Char('q') {
                    self.running = false;
                }
            }
            _ => {}
        }
    }
}
