use clap::Parser;
use std::path::PathBuf;
use noveltui_theme::ThemePreset;

#[derive(Parser, Debug, Clone, Default)]
#[command(author, version, about, long_about = None)]
pub struct Options {
    /// Path to the novel file
    #[arg(value_name = "FILE")]
    pub file_path: PathBuf,

    /// Jump to the bookmark number
    #[arg(short, long, value_name = "NUM", conflicts_with = "chapter")]
    pub bookmark: Option<usize>,

    /// Jump to the chapter number
    #[arg(short, long, value_name = "NUM", conflicts_with = "bookmark")]
    pub chapter: Option<usize>,

    /// Simple mode: hide title and footer (default: false)
    #[arg(short, long, default_value_t = false)]
    pub simple_mode: bool,

    /// Auto-scroll speed: <NUM> seconds per line (default: 1.5)
    #[arg(long, value_name = "NUM", default_value_t = 1.5)]
    pub speed: f64,

    /// Show bookmark menu in CLI mode and exit
    #[arg(long)]
    pub show_bookmarks: bool,

    /// Set the number of rows per page (default: 8)
    #[arg(short, long, value_name = "NUM", default_value_t = 8)]
    pub page_size: usize,

    /// Set regex filter for chapter titles
    #[arg(short, long)]
    pub regex: Option<String>,

    /// Set the color theme (default: default)
    /// Available themes: default, ocean, forest, sunset, midnight, sakura
    #[arg(short, long, value_name = "THEME", default_value = "default")]
    pub theme: ThemePreset,
}
