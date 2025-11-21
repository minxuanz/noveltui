use color_eyre::Result;
use noveltui::app::App;
fn main() -> Result<()> {
    let terminal = ratatui::init();
    let result = App::new().run(terminal);

    ratatui::restore();
    result
}
