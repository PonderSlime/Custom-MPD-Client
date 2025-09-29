extern crate mpd;

use mpd::Client;

mod menus;
mod functions;
mod app;
mod spectrum;
use app::App;
fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut conn = Client::connect("127.0.0.1:6600").unwrap();

    let app_result = App::default().run(&mut terminal, &mut conn);
    ratatui::restore();
    app_result
}


