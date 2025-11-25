use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Options {
    /// Input file to display
    pub file_path: PathBuf,
}
