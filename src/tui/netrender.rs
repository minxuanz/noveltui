// src/tui/netrender.rs
use ratatui::{prelude::*, widgets::*};

pub struct AppState {
    pub content_state: ListState,
    pub loading_success: bool,
    pub is_loading: bool,
    // 是否处于输入模式
    pub show_input: bool,
    // 输入框的内容
    pub input_buffer: String,
    // page size for content display
    pub page_size: usize,
    // show title
    pub show_title: bool,
}

pub fn render_ui(frame: &mut Frame, state: &mut AppState, content: &[String], title: &str, url: &str) {
    let constraint = if state.show_input || state.show_title {
        [Constraint::Min(1), Constraint::Length(1)]
    } else {
        [Constraint::Min(1), Constraint::Length(0)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraint)
        .split(frame.area());

    // 1. Content Rendering
    let inner_width = chunks[0].width.saturating_sub(7) as usize;
    let items: Vec<ListItem> = content
        .iter()
        .map(|line| {
            let wrapped = textwrap::wrap(line, inner_width);
            let mut joined = wrapped
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            joined.push_str("\n\n");
            ListItem::new(joined)
        })
        .collect();

    let blocks = if state.show_title {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(format!(" {} ", title))
            .border_style(Style::default().fg(Color::DarkGray))
    } else {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray))
    };

    let list = List::new(items)
        .block(blocks)
        .highlight_style(
            Style::default()
                // #96CEC1
                .fg(Color::Rgb(150, 206, 193)),
        )
        .highlight_symbol(" > ");

    frame.render_stateful_widget(list, chunks[0], &mut state.content_state);

    // 2. Footer Rendering
    let side_padding = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(chunks[1]);

    let inner_area = side_padding[1];
    let foot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(1),
            Constraint::Length(55),
        ])
        .split(inner_area);

    if state.show_input {
        frame.render_widget(
            Paragraph::new(format!("Jump To Chapter: {}_", state.input_buffer))
                .style(Style::default().fg(Color::White).bold()),
            foot_chunks[1],
        );

        // Footer hints
        frame.render_widget(
            Paragraph::new(" [Enter] Go  [Esc] Cancel")
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::Gray)),
            foot_chunks[2],
        );
    } else {
        let (status_text, status_color) = match state.is_loading {
            true => ("  LOADING  ", Color::Yellow),
            false if state.loading_success => ("  LOADED  ", Color::Green),
            _ => ("  FAILED  ", Color::Red),
        };

        let current_line = state.content_state.selected().map(|i| i + 1).unwrap_or(0);
        let total_lines = content.len();
        // let progress_text = format!(" {}/{} ", current_line, total_lines);

        frame.render_widget(
            Paragraph::new(status_text)
                .style(Style::default().bg(status_color).fg(Color::Black).bold()),
            foot_chunks[0],
        );

        frame.render_widget(
            Paragraph::new(format!(" URL: {} ", url))
                .alignment(Alignment::Left)
                .style(Style::default().fg(Color::Gray)),
            foot_chunks[1],
        );

        frame.render_widget(
            Paragraph::new(format!(
                "{}/{}  [/] Jump [n] Next [p] Prev [q] Quit [r] Refresh",
                current_line, total_lines
            ))
            .alignment(Alignment::Right)
            .style(Style::default().fg(Color::Gray)),
            foot_chunks[2],
        );
    }
}
