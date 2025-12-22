use crate::core::novel::Novel;
use crate::tui::state::{AppState, FocusArea};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::*,
};
use std::path::Path;

pub fn render_ui(frame: &mut Frame, state: &mut AppState, novel: &Novel, file_path: &Path) {
    let chunks = get_main_layout(frame.area(), state);

    if state.show_title {
        render_title(frame, chunks[0], file_path);
    }

    let mid_area_idx = if state.show_title { 1 } else { 0 };
    render_middle_section(frame, chunks[mid_area_idx], state, novel);

    let footer_idx = if state.show_title { 2 } else { 1 };

    if state.show_title {
        render_footer(frame, chunks[footer_idx], state, novel);
    }

    if state.show_help {
        render_help(frame, *chunks.last().unwrap());
    }
}

fn get_main_layout(area: Rect, state: &AppState) -> Vec<Rect> {
    let mut constraints = Vec::new();
    if state.show_title {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Min(1));
    if state.show_title {
        constraints.push(Constraint::Length(1));
    }
    if state.show_help {
        constraints.push(Constraint::Length(4));
    }

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

fn render_title(frame: &mut Frame, area: Rect, path: &Path) {
    let text = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("NovelTUI");
    let p = Paragraph::new(text)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}

fn render_middle_section(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    let constraints = if state.show_bookmark_menu {
        vec![
            Constraint::Percentage(10),
            Constraint::Min(1),
            Constraint::Percentage(15),
        ]
    } else {
        vec![Constraint::Percentage(10), Constraint::Min(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    render_toc(frame, chunks[0], state, novel);
    render_content(frame, chunks[1], state, novel);

    if state.show_bookmark_menu {
        render_bookmarks(frame, chunks[2], state);
    }
}

fn render_toc(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    let items: Vec<ListItem> = novel
        .chapters
        .iter()
        .map(|c| ListItem::new(c.title.clone()))
        .collect();

    let highlight = if state.focus == FocusArea::Toc {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" TOC ");

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight)
        .highlight_symbol(" ");
    frame.render_stateful_widget(list, area, &mut state.toc_state);
}

fn render_content(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    let chapter_lines = novel
        .get_chapter_lines(state.active_chapter_index)
        .unwrap_or(&[]);
    let inner_width = area.width.saturating_sub(4) as usize;

    let items: Vec<ListItem> = chapter_lines
        .iter()
        .map(|line| {
            let wrapped = textwrap::wrap(line, inner_width);
            let joined = wrapped
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            ListItem::new(Text::from(joined))
        })
        .collect();

    let highlight = if state.focus == FocusArea::Content {
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(" Content ");

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight)
        .highlight_symbol(">");

    frame.render_stateful_widget(list, area, &mut state.content_state);
}

fn render_bookmarks(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let items: Vec<ListItem> = state
        .cached_bookmarks
        .iter()
        .enumerate()
        .map(|(i, b)| ListItem::new(format!("{:02}. {}", i + 1, b.content)))
        .collect();

    let highlight = if state.focus == FocusArea::Bookmark {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Magenta)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Bookmarks "),
        )
        .highlight_style(highlight);

    frame.render_stateful_widget(list, area, &mut state.bookmark_state);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState, novel: &Novel) {
    // 渲染背景
    frame.render_widget(Block::default().style(Style::default()), area);

    // 核心修改：左右各留 1 列空白，使其与上方 Block 的边框垂直对齐
    let side_padding = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let inner_area = side_padding[1];

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(11), // Focus Mode
            Constraint::Min(1),     // Chapter Name
            Constraint::Length(15), // Auto Scroll
            Constraint::Length(25), // Progress & Total
            Constraint::Length(10), // Help Hint
        ])
        .split(inner_area);

    // 1. Focus Mode Segment (Lable 居中)
    let (label, color) = match state.focus {
        FocusArea::Toc => ("  TOC  ", Color::Cyan),
        FocusArea::Content => ("  CONTENT  ", Color::Green),
        FocusArea::Bookmark => (" MARKS ", Color::Magenta),
    };

    frame.render_widget(
        Paragraph::new(label).alignment(Alignment::Center).style(
            Style::default()
                .fg(Color::Black)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        layout[0],
    );

    // 2. Chapter Segment
    if let Some(meta) = novel.chapters.get(state.active_chapter_index) {
        frame.render_widget(
            Paragraph::new(format!(" {}", meta.title)).style(Style::default().fg(Color::White)),
            layout[1],
        );

        // 3. Auto Scroll Segment
        if state.auto_scroll {
            let speed = format!(" {}ms", state.auto_scroll_speed_ms);
            frame.render_widget(
                Paragraph::new(speed)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow)),
                layout[2],
            );
        }

        // 4. Percentage & Line Segment
        let current_line = state.content_state.selected().unwrap_or(0);
        let global_line = meta.range.start + current_line + 1;
        let total = novel.lines.len();
        let progress = (global_line as f64 / total as f64 * 100.0) as usize;

        let progress_text = format!(" {}%  {}/{} ", progress, global_line, total);
        frame.render_widget(
            Paragraph::new(progress_text)
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::Green)),
            layout[3],
        );

        // 5. Help Hint (背景色块，右对齐)
        frame.render_widget(
            Paragraph::new(" ? Help ")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            layout[4],
        );
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    // 渲染边框
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(block, area);

    let side_padding = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let inner_area = side_padding[1];

    let help_groups = [
        ("Movement", "k/↑ Up  j/↓ Down  h/← Left  l/→ Right"),
        (
            "Actions ",
            "m Mark  M Clear All  Space AutoScroll  Enter Focus",
        ),
        ("System  ", "b Bookmarks  s UI Toggle  q Quit  Q Save&Quit"),
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner_area);

    for (i, (category, keys)) in help_groups.iter().enumerate() {
        let line = Line::from(vec![
            Span::styled(
                format!(" {}: ", category),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(*keys, Style::default().fg(Color::Gray)),
        ]);
        // 从 chunks[1] 开始渲染，避开边框
        frame.render_widget(Paragraph::new(line), chunks[i + 1]);
    }
}
