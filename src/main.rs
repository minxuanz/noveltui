use clap::Parser;
use color_eyre::Result;
use noveltui::cmd::args::Options;
use noveltui::core::novel::Novel;
use noveltui::infra::fs;
use noveltui::tui::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Options::parse();

    // CLI Mode: Show bookmarks
    if args.show_bookmark {
        let lines = fs::load_content(&args.file_path)?;
        let novel = Novel::new(lines);
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
