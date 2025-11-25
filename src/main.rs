use clap::Parser;
use color_eyre::Result;
use noveltui::{app::App, args::Options};
fn main() -> Result<()> {
    let args = Options::parse();
    let terminal = ratatui::init();
    let result = App::new(args).run(terminal);

    ratatui::restore();
    result
}
