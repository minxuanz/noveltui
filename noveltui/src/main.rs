use anyhow::{Result, anyhow};
use clap::Parser;
use noveltui::cmd::args::Options;
use noveltui::core::novel::Novel;
use noveltui::infra::fs;
use noveltui::tui::app::App;
use std::path::Path;

fn initialize_novel(file_path: &Path, options: &Options) -> Result<Novel> {
    let extension = file_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    if extension.as_deref() != Some("txt") {
        return Err(anyhow!(
            "Unsupported file format. Please provide a .txt file."
        ));
    }

    let lines = fs::load_content(file_path)?;
    Ok(Novel::new(lines, options))
}

fn show_bookmarks_cli(novel: &Novel, file_path: &Path) {
    let bookmarks = novel.collect_bookmarks();

    if bookmarks.is_empty() {
        println!("No bookmarks found.");
    } else {
        println!("Bookmarks in {:?}:", file_path);
        for (i, b) in bookmarks.iter().enumerate() {
            let chapter_title = novel
                .chapters
                .get(b.chapter_index)
                .map(|c| c.title.as_ref())
                .unwrap_or("Unknown Chapter");
            println!("{}. [{}] {}", i + 1, chapter_title, b.content);
        }
    }
}

fn main() -> Result<()> {
    let args = Options::parse();

    let novel = initialize_novel(&args.file_path, &args)?;

    // CLI Mode: Show bookmarks
    if args.show_bookmarks {
        show_bookmarks_cli(&novel, &args.file_path);
        return Ok(());
    }

    // TUI Mode
    let terminal = ratatui::init();
    let result = App::new(args, novel)?.run(terminal);
    ratatui::restore();

    result
}
