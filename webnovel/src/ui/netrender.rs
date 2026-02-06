// src/ui/netrender.rs
use ratatui::text::Line;
use ratatui::widgets::ListState;
use ratatui::{prelude::*, widgets::*};
use noveltui_theme::ThemeColors;

/// 渲染 UI
pub fn render_ui(
    frame: &mut Frame,
    content_state: &mut ListState,
    content: &[String],
    title: &str,
    url: &str,
    show_input: bool,
    show_title: bool,
    input_buffer: &str,
    is_loading: bool,
    loading_success: bool,
    inc_line_space: bool,
    theme_colors: ThemeColors,
) {
    let constraint = if show_input || show_title {
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
            let mut lines: Vec<Line> = wrapped.into_iter().map(|c| Line::raw(c)).collect();
            if inc_line_space {
                lines.push(Line::raw(""));
            }
            ListItem::new(lines)
        })
        .collect();

    let blocks = if show_title {
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
        .highlight_style(Style::default().fg(theme_colors.content))
        .highlight_symbol(" > ");

    frame.render_stateful_widget(list, chunks[0], content_state);

    // 2. Footer Rendering
    if show_input {
        render_input_footer(frame, chunks[1], input_buffer);
    } else {
        render_status_footer(
            frame,
            chunks[1],
            content_state,
            content.len(),
            url,
            is_loading,
            loading_success,
        );
    }
}

fn render_input_footer(frame: &mut Frame, area: Rect, input_buffer: &str) {
    let foot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(10), Constraint::Length(25)])
        .split(area);

    frame.render_widget(
        Paragraph::new(format!("Please input URL: {}_", input_buffer))
            .style(Style::default().bold()),
        foot_chunks[0],
    );

    // Footer hints
    frame.render_widget(
        Paragraph::new(" [Enter] Go  [Esc] Cancel")
            .alignment(Alignment::Right)
            .style(Style::default()),
        foot_chunks[1],
    );
}

fn render_status_footer(
    frame: &mut Frame,
    area: Rect,
    content_state: &ListState,
    total_lines: usize,
    url: &str,
    is_loading: bool,
    loading_success: bool,
) {
    let foot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(1),
            Constraint::Max(55),
        ])
        .split(area);

    let (status_text, status_color) = match is_loading {
        true => ("  LOADING  ", Color::Yellow),
        false if loading_success => ("  LOADED  ", Color::Green),
        _ => ("  FAILED  ", Color::Red),
    };

    let current_line = content_state.selected().map(|i| i + 1).unwrap_or(0);

    frame.render_widget(
        Paragraph::new(status_text)
            .style(Style::default().bg(status_color).fg(Color::White).bold()),
        foot_chunks[0],
    );

    frame.render_widget(
        Paragraph::new(format!(" URL: {} ", url))
            .alignment(Alignment::Left)
            .style(Style::default()),
        foot_chunks[1],
    );

    // Only show full help text if there's enough width
    let help_text = if area.width > 60 {
        format!(
            "{}/{}  [/] Jump [n] Next [p] Prev [q] Quit [r] Refresh",
            current_line, total_lines
        )
    } else {
        format!("{}/{}", current_line, total_lines)
    };

    frame.render_widget(
        Paragraph::new(help_text)
            .alignment(Alignment::Right)
            .style(Style::default()),
        foot_chunks[2],
    );
}
