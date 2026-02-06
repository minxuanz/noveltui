use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event};
use noveltui_theme::ThemePreset;
use ratatui::prelude::*;
use webnovel::app::{
    AppAction, AppState, ContentData, ContentLoader, EventHandler, receive_updates,
};
use webnovel::net::crawler::UrlHandler;
use webnovel::ui::netrender::render_ui;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    url: Option<String>,

    /// Set the number of rows per page (default: 8)
    #[arg(short, long, default_value_t = 8)]
    page_size: usize,

    /// Set the color theme (default: default)
    /// Available themes: default, ocean, forest, sunset, midnight, sakura
    #[arg(short, long, value_name = "THEME", default_value = "default")]
    theme: ThemePreset,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 验证初始 URL
    if let Some(ref url) = args.url {
        if !UrlHandler::is_valid(url) {
            eprintln!("Error: URL not supported.");
            return Ok(());
        }
    }

    let mut terminal = ratatui::init();
    let result = run_app(&mut terminal, args);
    ratatui::restore();

    result
}

fn run_app(terminal: &mut Terminal<impl Backend>, args: Args) -> Result<()> {
    // 初始化状态
    let theme_colors = args.theme.colors();
    let mut state = AppState::new(args.page_size);
    let mut event_handler = EventHandler::new();
    let (loader, receiver) = ContentLoader::new();

    // 当前显示的内容和 URL
    let mut content_data = ContentData::welcome();
    let mut current_url = args.url.clone().unwrap_or_default();

    // 初始加载
    if let Some(url) = args.url {
        loader.send_loading(&url);
        loader.load(url);
    } else {
        loader.send_welcome();
    }

    // 主循环
    loop {
        // 接收更新
        if let Some(new_data) = receive_updates(&receiver) {
            content_data = new_data;
        }

        // 渲染 UI
        terminal.draw(|f| {
            render_ui(
                f,
                &mut state.content_state,
                &content_data.lines,
                &content_data.title,
                &current_url,
                state.show_input,
                state.show_title,
                &state.input_buffer,
                content_data.is_loading,
                content_data.success,
                state.inc_line_space,
                theme_colors,
            )
        })?;

        // 处理事件
        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    let action = event_handler.handle_key(key.code);

                    match action {
                        AppAction::Quit => break,
                        AppAction::StartInput => state.start_input(),
                        AppAction::CancelInput => state.cancel_input(),
                        AppAction::ConfirmInput => {
                            let input = state.confirm_input();
                            if UrlHandler::is_valid(&input) {
                                current_url = input.clone();
                                state.history.add(input);
                                loader.send_loading(&current_url);
                                loader.load(current_url.clone());
                                state.reset_cursor();
                            } else {
                                content_data = ContentData::error(
                                    UrlHandler::validation_error(&input).to_string(),
                                );
                            }
                        }
                        AppAction::InputChar(c) => state.handle_char_input(c),
                        AppAction::InputBackspace => state.handle_backspace(),
                        AppAction::HistoryUp => state.navigate_history_up(),
                        AppAction::HistoryDown => state.navigate_history_down(),
                        AppAction::MoveUp => state.move_cursor_up(),
                        AppAction::MoveDown => state.move_cursor_down(content_data.lines.len()),
                        AppAction::PageUp => state.page_up(),
                        AppAction::PageDown => state.page_down(content_data.lines.len()),
                        AppAction::NextChapter => {
                            current_url = UrlHandler::update_chapter(&current_url, 1);
                            loader.send_loading(&current_url);
                            loader.load(current_url.clone());
                            state.reset_cursor();
                        }
                        AppAction::PrevChapter => {
                            current_url = UrlHandler::update_chapter(&current_url, -1);
                            loader.send_loading(&current_url);
                            loader.load(current_url.clone());
                            state.reset_cursor();
                        }
                        AppAction::Refresh => {
                            loader.send_loading(&current_url);
                            loader.load(current_url.clone());
                            state.reset_cursor();
                        }
                        AppAction::ToggleTitle => state.show_title = !state.show_title,
                        AppAction::IncreaseLineSpace => {
                            state.inc_line_space = !state.inc_line_space;
                        }
                        AppAction::None => {}
                    }
                }
            }
        }
    }

    Ok(())
}
