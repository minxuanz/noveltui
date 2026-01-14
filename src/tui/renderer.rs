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

    if state.show_delete_confirmation && state.focus == FocusArea::Bookmark {
        render_dim_layer(frame, frame.area());
        render_delete_confirmation(frame, frame.area());
    }
}

fn get_main_layout(area: Rect, state: &AppState) -> Vec<Rect> {
    let mut constraints = Vec::new();
    if state.show_title {
        constraints.push(Constraint::Length(1)); // Title
    }
    constraints.push(Constraint::Min(1)); // Content
    if state.show_title {
        constraints.push(Constraint::Length(1)); // Footer
    }
    if state.show_help {
        constraints.push(Constraint::Length(4)); // Help
    }

    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area)
        .to_vec()
}

fn render_dim_layer(frame: &mut Frame, area: Rect) {
    let dim_block =
        Block::default().style(Style::default().bg(Color::Reset).fg(Color::Indexed(240)));

    frame.render_widget(dim_block, area);
}

fn render_middle_section(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    render_content(frame, area, state, novel);

    if state.show_toc_menu || state.show_bookmark_menu {
        render_dim_layer(frame, area);
    }

    if state.show_toc_menu {
        let popup_area = centered_rect(60, 70, area);
        render_toc(frame, popup_area, state, novel);
    }

    if state.show_bookmark_menu {
        let popup_area = centered_rect(60, 70, area);
        render_bookmarks(frame, popup_area, state, novel);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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

fn render_toc(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    let clear_area = Rect {
        x: area.x.saturating_sub(1),
        y: area.y.saturating_sub(1),
        width: area.width.saturating_add(2),
        height: area.height.saturating_add(2),
    };
    frame.render_widget(Clear, clear_area); // 清理背景

    let items: Vec<ListItem> = novel
        .chapters
        .iter()
        .map(|c| ListItem::new(c.title.clone()))
        .collect();

    let theme_color = Color::Rgb(129, 199, 212); // #81C7D4
    let highlight = if state.focus == FocusArea::Toc {
        Style::default().fg(theme_color)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme_color))
        .title(" TOC ");

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight)
        .highlight_symbol(" > ");
    frame.render_stateful_widget(list, area, &mut state.toc_state);
}

fn render_content(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    let chapter_lines = novel
        .get_chapter_lines(state.active_chapter_index)
        .unwrap_or(&[]);
    let inner_width = area.width.saturating_sub(4) as usize;

    let title = if let Some(meta) = novel.chapters.get(state.active_chapter_index)
        && state.show_title
    {
        format!(" {} ", meta.title)
    } else {
        "".to_string()
    };

    let items: Vec<ListItem> = chapter_lines
        .iter()
        .map(|line| {
            let wrapped = textwrap::wrap(line, inner_width);
            let mut joined = wrapped
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            joined.push_str("\n\n");
            ListItem::new(Text::from(joined))
        })
        .collect();

    let highlight = if state.focus == FocusArea::Content {
        Style::default().fg(Color::Rgb(168, 216, 185))
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(title);

    let list = List::new(items)
        .block(block)
        .highlight_style(highlight)
        .highlight_symbol(">");

    frame.render_stateful_widget(list, area, &mut state.content_state);
}

fn render_bookmarks(frame: &mut Frame, area: Rect, state: &mut AppState, _novel: &Novel) {
    // area of clear must be bigger than the area of block
    let clear_area = Rect {
        x: area.x.saturating_sub(1),
        y: area.y.saturating_sub(1),
        width: area.width.saturating_add(2),
        height: area.height.saturating_add(2),
    };
    frame.render_widget(Clear, clear_area); // clear background

    let items: Vec<ListItem> = state
        .cached_bookmarks
        .iter()
        .map(|b| ListItem::new(format!("[{}] {}", b.chapter_title, b.content)))
        .collect();

    let theme_color = Color::Rgb(248, 195, 205); // #F8C3CD
    let highlight = if state.focus == FocusArea::Bookmark {
        Style::default().fg(theme_color)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color))
                .title(" Bookmarks "),
        )
        .highlight_style(highlight)
        .highlight_symbol(" ● ");

    frame.render_stateful_widget(list, area, &mut state.bookmark_state);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState, novel: &Novel) {
    frame.render_widget(Block::default().style(Style::default()), area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12), // Focus Mode
            Constraint::Min(1),     // Chapter Name
            Constraint::Length(15), // Auto Scroll
            Constraint::Length(25), // Progress
            Constraint::Length(10), // Help
        ])
        .split(area);

    let (label, color) = match state.focus {
        FocusArea::Toc => ("    TOC", Color::Rgb(81, 168, 221)),
        FocusArea::Content => ("  CONTENT", Color::Rgb(0, 170, 144)),
        FocusArea::Bookmark => ("    MARK", Color::Rgb(203, 27, 69)),
    };

    frame.render_widget(
        Paragraph::new(label).alignment(Alignment::Left).style(
            Style::default()
                .fg(Color::White)
                .bg(color)
                .add_modifier(Modifier::BOLD),
        ),
        layout[0],
    );

    let toc_idx = state
        .toc_state
        .selected()
        .unwrap_or(state.active_chapter_index);
    if let Some(meta) = novel.chapters.get(toc_idx) {
        frame.render_widget(
            Paragraph::new(format!(" {}", meta.title)).style(Style::default().fg(Color::White)),
            layout[1],
        );

        if state.auto_scroll {
            frame.render_widget(
                Paragraph::new(format!(" {}ms", state.auto_scroll_speed_ms))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow)),
                layout[2],
            );
        }

        let current_line = state.content_state.selected().unwrap_or(0);
        let global_line = meta.range.start + current_line + 1;
        let total = novel.lines.len();
        let progress = (global_line as f64 / total as f64 * 100.0) as usize;

        frame.render_widget(
            Paragraph::new(format!(" {}%  {}/{} ", progress, global_line, total))
                .alignment(Alignment::Right)
                .style(Style::default().fg(Color::White)),
            layout[3],
        );

        frame.render_widget(
            Paragraph::new(" ? Help ")
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Black).bg(Color::Gray)),
            layout[4],
        );
    }
}

fn render_help(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));
    frame.render_widget(block, area);

    let help_groups = [
        " k/↑  Up            j/↓  Down               PgUp   Pageup        PgDn   Pagedown   Enter  Select",
        " m    Toggle Mark   M    Clear All Marks    q/esc  Mark&Quit     Q      Quit       ",
        " b    Bookmarks     t    TOC                s      Title&Footer  Space  AutoScroll ",
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    for (i, keys) in help_groups.iter().enumerate() {
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                *keys,
                Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
            )])),
            chunks[i + 1],
        );
    }
}

fn render_delete_confirmation(frame: &mut Frame, area: Rect) {
    let dialog_area = centered_rect(30, 20, area);
    let clear_area = centered_rect(32, 22, area);
    frame.render_widget(Clear, clear_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Green));
    //frame.render_widget(block, dialog_area);
    let inner_area = block.inner(dialog_area);
    frame.render_widget(block, dialog_area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner_area);

    frame.render_widget(
        Paragraph::new("Delete all bookmarks?").alignment(Alignment::Center),
        layout[0],
    );
    let buttons = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    frame.render_widget(
        Paragraph::new(" Yes (Y) ")
            .alignment(Alignment::Center)
            .style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM),
            ),
        buttons[0],
    );
    frame.render_widget(
        Paragraph::new(" No (N) ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        buttons[1],
    );
}
