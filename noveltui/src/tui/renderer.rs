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
use std::rc::Rc;

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

    if state.show_delete_confirmation {
        render_dim_layer(frame, frame.area());
        render_delete_confirmation(frame, frame.area());
    }
}

fn get_main_layout(area: Rect, state: &AppState) -> Rc<[Rect]> {
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
    frame.render_widget(Clear, clear_area); // Clear background

    let items: Vec<ListItem> = novel
        .chapters
        .iter()
        .map(|c| ListItem::new(c.title.as_ref()))
        .collect();

    let theme_color = state.theme_colors.toc;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme_color))
        .title(" TOC ");

    let list = List::new(items)
        .block(block)
        .highlight_style(theme_color)
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
        format!(" {} ", meta.title.as_ref())
    } else {
        String::default()
    };

    let items: Vec<ListItem> = chapter_lines
        .iter()
        .map(|line| {
            let wrapped = textwrap::wrap(line, inner_width);
            let mut lines: Vec<Line> = wrapped.into_iter().map(|c| Line::raw(c)).collect();

            if state.inc_line_space {
                lines.push(Line::raw(""));
            }
            ListItem::new(Text::from(lines))
        })
        .collect();

    let highlight = Style::default().fg(state.theme_colors.content);

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

fn render_bookmarks(frame: &mut Frame, area: Rect, state: &mut AppState, novel: &Novel) {
    // area of clear must be bigger than the area of block
    let clear_area = Rect {
        x: area.x.saturating_sub(1),
        y: area.y.saturating_sub(1),
        width: area.width.saturating_add(2),
        height: area.height.saturating_add(2),
    };
    frame.render_widget(Clear, clear_area); // clear background

    let bookmarks = novel.collect_bookmarks();
    let items: Vec<ListItem> = bookmarks
        .iter()
        .map(|b| {
            let title = novel
                .chapters
                .get(b.chapter_index)
                .map(|c| c.title.as_ref())
                .unwrap_or("");
            ListItem::new(format!("[{}] {}", title, b.content))
        })
        .collect();

    let theme_color = state.theme_colors.bookmark;

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color))
                .title(" Bookmarks "),
        )
        .highlight_style(theme_color)
        .highlight_symbol(" ● ");

    frame.render_stateful_widget(list, area, &mut state.bookmark_state);
}

fn render_footer(frame: &mut Frame, area: Rect, state: &AppState, novel: &Novel) {
    let width = area.width;
    let is_narrow = width < 70; // Narrow screen threshold
    let is_tiny = width < 45; // Tiny screen threshold

    let mut constraints = Vec::new();

    // [0] Focus Area
    let focus_width = if is_tiny { 8 } else { 11 };
    constraints.push(Constraint::Length(focus_width));

    // [1] Chapter Title
    constraints.push(Constraint::Min(1));

    // [2] Auto Scroll
    if !is_narrow && state.auto_scroll {
        constraints.push(Constraint::Length(15));
    }

    // [3] Progress
    let progress_width = if is_tiny {
        0
    } else if is_narrow {
        12
    } else {
        25
    };
    if progress_width > 0 {
        constraints.push(Constraint::Length(progress_width));
    }

    // [4] Help
    let help_width = if is_tiny { 6 } else { 10 };
    constraints.push(Constraint::Length(help_width));

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    let mut chunk_idx = 0;

    // 1. Focus Area (Index 0)
    let (label, color) = match state.focus {
        FocusArea::Toc => (
            if is_tiny { " TOC" } else { "    TOC" },
            state.theme_colors.toc,
        ),
        FocusArea::Content => (
            if is_tiny { " CTX" } else { "  CONTENT" },
            state.theme_colors.content,
        ),
        FocusArea::Bookmark => (
            if is_tiny { " MRK" } else { " BOOKMARKS" },
            state.theme_colors.bookmark,
        ),
    };

    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Left)
            .style(Style::default().bg(color).add_modifier(Modifier::BOLD)),
        layout[chunk_idx],
    );
    chunk_idx += 1;

    // 2. Chapter Title (Index 1)
    let toc_idx = state
        .toc_state
        .selected()
        .unwrap_or(state.active_chapter_index);

    if let Some(meta) = novel.chapters.get(toc_idx) {
        frame.render_widget(
            Paragraph::new(format!(" {} ", meta.title.as_ref())).style(Style::default()),
            layout[chunk_idx],
        );
        chunk_idx += 1;

        // 3. Auto Scroll
        if !is_narrow && state.auto_scroll {
            frame.render_widget(
                Paragraph::new(format!(" {}ms", state.auto_scroll_speed_ms))
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::Yellow)),
                layout[chunk_idx],
            );
            chunk_idx += 1;
        }

        // 4. Progress
        if progress_width > 0 {
            let current_line = state.content_state.selected().unwrap_or(0);
            let global_line = meta.range.start + current_line + 1;
            let total = novel.lines.len();
            let progress = (global_line as f64 / total as f64 * 100.0) as usize;
            let chapter_total = meta.range.end - meta.range.start;

            let progress_text = if is_narrow {
                format!("{}% {}/{} ", progress, current_line + 1, chapter_total)
            } else {
                format!(
                    "{}/{} | {}% {}/{} ",
                    current_line + 1,
                    chapter_total,
                    progress,
                    global_line,
                    total
                )
            };

            frame.render_widget(
                Paragraph::new(progress_text)
                    .alignment(Alignment::Right)
                    .style(Style::default()),
                layout[chunk_idx],
            );
            chunk_idx += 1;
        }

        // 5. Help (Index Last)
        let help_text = if is_tiny { "?" } else { " ? Help " };
        frame.render_widget(
            Paragraph::new(help_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Black).bg(Color::DarkGray)),
            layout[chunk_idx],
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
        " m    Toggle Mark   M    Clear All Marks    q      Mark&Quit     Q      Quit       l      Adjust Line Space",
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
        .border_style(Style::default().fg(Color::Red));
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
            .style(Style::default().fg(Color::DarkGray)),
        buttons[0],
    );
    frame.render_widget(
        Paragraph::new(" No (N) ")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        buttons[1],
    );
}
