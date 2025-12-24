use ratatui::{prelude::*, widgets::*};

pub struct AppState {
    pub content_state: ListState,
    pub loading_success: bool,
    pub is_loading: bool,
}

pub fn render_ui(frame: &mut Frame, state: &mut AppState, content: &[String], title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main Content
            Constraint::Length(1), // Status Bar
        ])
        .split(frame.area());

    // 1. Content Rendering with automatic text wrapping
    let inner_width = chunks[0].width.saturating_sub(4) as usize;
    let items: Vec<ListItem> = content
        .iter()
        .map(|line| {
            let wrapped = textwrap::wrap(line, inner_width);
            let joined = wrapped
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            ListItem::new(joined)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" {} ", title)) // Title in the top border
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" > ");

    frame.render_stateful_widget(list, chunks[0], &mut state.content_state);

    // 2. Footer Rendering
    let foot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(1),
            Constraint::Length(30),
        ])
        .split(chunks[1]);

    let (status_text, status_color) = match state.is_loading {
        true => ("  LOADING ", Color::Yellow),
        false if state.loading_success => ("  LOADED", Color::Green),
        _ => ("  FAILED  ", Color::Red),
    };

    frame.render_widget(
        Paragraph::new(status_text)
            .style(Style::default().bg(status_color).fg(Color::Black).bold()),
        foot_chunks[0],
    );

    // Navigation Help
    frame.render_widget(
        Paragraph::new(" [N] Next [P] Prev [Q] Quit ")
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::Gray)),
        foot_chunks[2],
    );
}
