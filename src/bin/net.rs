use clap::Parser;
use crossterm::event::{self, Event, KeyCode};
use noveltui::net::crawler;
use noveltui::tui::netrender::{AppState, render_ui};
use ratatui::prelude::*;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    url: String,
}

struct SharedData {
    content: Vec<String>,
    title: String,
    is_loading: bool,
    success: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if !args.url.starts_with("https://ixd") {
        eprintln!("Error: URL don't supported.");
        return Ok(());
    }
    let mut terminal = ratatui::init();

    // 初始化状态，增加 show_input 和 input_buffer
    let mut state = AppState {
        content_state: ratatui::widgets::ListState::default(),
        loading_success: false,
        is_loading: true,
        show_input: false,
        input_buffer: String::new(),
    };
    state.content_state.select(Some(0));

    let data = Arc::new(Mutex::new(SharedData {
        content: vec!["Initializing...".to_string()],
        title: "Waiting...".to_string(),
        is_loading: true,
        success: false,
    }));

    let current_url = args.url.clone();

    load_chapter(Arc::clone(&data), current_url.clone());

    let result = run_loop(&mut terminal, &mut state, data, current_url);

    ratatui::restore();
    result
}

fn run_loop(
    terminal: &mut Terminal<impl Backend>,
    state: &mut AppState,
    data: Arc<Mutex<SharedData>>,
    mut current_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let (display_content, display_title) = {
            let d = data.lock().unwrap();
            state.is_loading = d.is_loading;
            state.loading_success = d.success;
            (d.content.clone(), d.title.clone())
        };

        terminal.draw(|f| render_ui(f, state, &display_content, &display_title))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == crossterm::event::KeyEventKind::Press {
                    // === 输入模式逻辑 ===
                    if state.show_input {
                        match key.code {
                            // 输入数字
                            KeyCode::Char(c) if c.is_ascii_digit() => {
                                state.input_buffer.push(c);
                            }
                            // 退格键删除
                            KeyCode::Backspace => {
                                state.input_buffer.pop();
                            }
                            // 确认跳转
                            KeyCode::Enter => {
                                if let Ok(page_num) = state.input_buffer.parse::<i32>() {
                                    if page_num > 0 {
                                        current_url = get_url_by_page(&current_url, page_num);
                                        load_chapter(Arc::clone(&data), current_url.clone());
                                        state.content_state.select(Some(0));
                                    }
                                }
                                // 重置并退出输入模式
                                state.show_input = false;
                                state.input_buffer.clear();
                            }
                            // 取消
                            KeyCode::Esc => {
                                state.show_input = false;
                                state.input_buffer.clear();
                            }
                            _ => {}
                        }
                    }
                    // === 正常模式逻辑 ===
                    else {
                        match key.code {
                            KeyCode::Char('q') => break,
                            KeyCode::Char('/') => {
                                state.show_input = true;
                                state.input_buffer.clear();
                            }
                            KeyCode::Char('j') | KeyCode::Down => {
                                let i = state.content_state.selected().unwrap_or(0);
                                if i < display_content.len().saturating_sub(1) {
                                    state.content_state.select(Some(i + 1));
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                let i = state.content_state.selected().unwrap_or(0);
                                if i > 0 {
                                    state.content_state.select(Some(i - 1));
                                }
                            }
                            KeyCode::Char('n') => {
                                current_url = update_url(&current_url, 1);
                                load_chapter(Arc::clone(&data), current_url.clone());
                                state.content_state.select(Some(0));
                            }
                            KeyCode::Char('p') => {
                                current_url = update_url(&current_url, -1);
                                load_chapter(Arc::clone(&data), current_url.clone());
                                state.content_state.select(Some(0));
                            }
                            KeyCode::Char('r') => {
                                load_chapter(Arc::clone(&data), current_url.clone());
                                state.content_state.select(Some(0));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn load_chapter(shared_data: Arc<Mutex<SharedData>>, url: String) {
    {
        let mut d = shared_data.lock().unwrap();
        d.is_loading = true;
        d.title = format!("Fetching: {}", url);
        d.content = vec![
            "Loading content, please wait...".to_string(),
            format!("URL: {}", url),
        ];
    }

    thread::spawn(move || match crawler::fetch_novel(&url) {
        Ok(page) => {
            let mut d = shared_data.lock().unwrap();
            d.content = page.content;
            d.title = page.title;
            d.is_loading = false;
            d.success = true;
        }
        Err(e) => {
            let mut d = shared_data.lock().unwrap();
            d.content = vec![format!("Error: {}", e)];
            d.is_loading = false;
            d.success = false;
        }
    });
}

fn update_url(url: &str, delta: i32) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 5 {
        return url.to_string();
    }

    let last = parts.last().unwrap_or(&"");
    let chapter_num = last
        .strip_prefix('p')
        .and_then(|s| s.strip_suffix(".html"))
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(1);

    let next_chapter = (chapter_num + delta).max(1);
    format!(
        "https://{}/{}/{}/p{}.html",
        parts[2], parts[3], parts[4], next_chapter
    )
}

fn get_url_by_page(url: &str, page_num: i32) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    // parts[0] = "https:"
    // parts[1] = ""
    // parts[2] = domain
    // parts[3] = "read"
    // parts[4] = book_id
    // parts[5] = "p1.html"

    if parts.len() < 5 {
        return url.to_string();
    }

    format!(
        "https://{}/{}/{}/p{}.html",
        parts[2], parts[3], parts[4], page_num
    )
}
