// src/tui/netrender.rs
use ratatui::{prelude::*, widgets::*};

pub struct AppState {
    pub content_state: ListState,
    pub loading_success: bool,
    pub is_loading: bool,
    // 新增：是否处于输入模式
    pub show_input: bool,
    // 新增：输入框的内容
    pub input_buffer: String,
}

pub fn render_ui(frame: &mut Frame, state: &mut AppState, content: &[String], title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Main Content
            Constraint::Length(1), // Status Bar
        ])
        .split(frame.area());

    // 1. Content Rendering
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
                .title(format!(" {} ", title))
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

    // 2. Footer Rendering (修改逻辑)
    let foot_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11),
            Constraint::Min(1),
            Constraint::Length(30),
        ])
        .split(chunks[1]);

    // 如果处于输入模式，渲染输入框
    if state.show_input {
        // Footer 左侧：提示 Jump To
        frame.render_widget(
            Paragraph::new("  JUMP TO")
                .style(Style::default().bg(Color::DarkGray).fg(Color::Black).bold()),
            foot_chunks[0],
        );

        // Footer 中间：输入的内容
        frame.render_widget(
            Paragraph::new(format!("Chapter: {}_", state.input_buffer)) // 加个下划线模拟光标
                .style(Style::default().fg(Color::White).bg(Color::DarkGray)),
            foot_chunks[1],
        );

        // Footer 右侧：操作提示
        frame.render_widget(
            Paragraph::new(" [Enter] Go [Esc] Cancel ")
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::Gray)),
            foot_chunks[2],
        );
    } else {
        let (status_text, status_color) = match state.is_loading {
            true => ("  LOADING ", Color::Yellow),
            false if state.loading_success => ("  LOADED", Color::Green),
            _ => ("  FAILED  ", Color::Red),
        };

        let current_line = state.content_state.selected().map(|i| i + 1).unwrap_or(0);
        let total_lines = content.len();
        let progress_text = format!(" Line: {}/{} ", current_line, total_lines);

        frame.render_widget(
            Paragraph::new(status_text)
                .style(Style::default().bg(status_color).fg(Color::Black).bold()),
            foot_chunks[0],
        );

        frame.render_widget(
            Paragraph::new(progress_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray)),
            foot_chunks[1],
        );

        frame.render_widget(
            Paragraph::new(" [/] Jump [N] Next [P] Prev [Q] Quit ")
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::Gray)),
            foot_chunks[2],
        );
    }
}
