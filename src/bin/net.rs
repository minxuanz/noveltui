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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut terminal = ratatui::init();

    // Initial App State
    let mut state = AppState {
        content_state: ratatui::widgets::ListState::default(),
        loading_success: false,
        is_loading: true,
    };
    state.content_state.select(Some(0));

    // Shared thread-safe data
    let data = Arc::new(Mutex::new(SharedData {
        content: vec!["Initializing...".to_string()],
        title: "Waiting...".to_string(),
        is_loading: true,
        success: false,
    }));

    let current_url = args.url.clone();

    // Load initial page
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
        // Sync local app state with shared thread data
        let (display_content, display_title) = {
            let d = data.lock().unwrap();
            state.is_loading = d.is_loading;
            state.loading_success = d.success;
            (d.content.clone(), d.title.clone())
        };

        terminal.draw(|f| render_ui(f, state, &display_content, &display_title))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
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
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

/// Unified function to handle background fetching and UI feedback
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
