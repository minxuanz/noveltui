use clap::Parser;
use color_eyre::Result;
use noveltui::{app::App, args::Options};

fn main() -> Result<()> {
    color_eyre::install()?;   
    let args = Options::parse();
    
    if args.show_bookmark {
        let app = App::new(args);
        let (_, _, bookmarks) = app.load_file_and_get_bookmarks()?;
        
        // Display bookmarks to stdout
        if bookmarks.is_empty() {
            println!("No bookmarks found in the file.");
        } else {
            println!("Bookmarks in {}:", app.file_path().display());
            for (i, bookmark) in bookmarks.iter().enumerate() {
                println!("{}. {}", i + 1, bookmark.line_content);
            }
        }
        
        return Ok(());
    }
    
    // Normal TUI mode
    let terminal = ratatui::init();
    let result = App::new(args).run(terminal);

    ratatui::restore();
    result
}
