use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
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
}
