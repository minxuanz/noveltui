use anyhow::{anyhow, Result};
use clap::Parser;
use noveltui::cmd::args::Options;
use noveltui::core::novel::Novel;
use noveltui::infra::fs;
use noveltui::tui::app::App;

fn main() -> Result<()> {
    let args = Options::parse();

    let extension = args
        .file_path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    if extension.as_deref() != Some("txt") {
        return Err(anyhow!(
            "Unsupported file format. Please provide a .txt file."
        ));
    }

    let lines = match fs::load_content(&args.file_path) {
        Ok(lines) => lines,
        Err(e) => {
            return Err(anyhow!("{}: {}", args.file_path.display(), e));
        }
    };

    let novel = Novel::new(lines);
    // CLI Mode: Show bookmarks
    if args.show_bookmarks {
        let bookmarks = novel.collect_bookmarks();

        if bookmarks.is_empty() {
            println!("No bookmarks found.");
        } else {
            println!("Bookmarks in {:?}:", args.file_path);
            for (i, b) in bookmarks.iter().enumerate() {
                println!("{}. {}", i + 1, b.content);
            }
        }
        return Ok(());
    }

    // TUI Mode
    let terminal = ratatui::init();
    let result = App::new(args)?.run(terminal);
    ratatui::restore();

    result
}
