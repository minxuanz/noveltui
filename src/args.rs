use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
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

    /// simple mode: hide title and footer(default: false)
    #[arg(short, long, default_value_t = false)]
    pub simple_mode: bool,

    /// auto-scroll speed: <NUM> seconds per line (default: 1.5)
    #[arg(long, value_name = "NUM", default_value_t = 1.5)]
    pub speed: f64,

    /// show bookmark menu
    #[arg(long)]
    pub show_bookmark: bool,

}
