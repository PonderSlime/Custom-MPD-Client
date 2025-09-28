extern crate mpd;

use mpd::Client;
use std::{
    net::TcpStream,
    time::{Duration, Instant, SystemTime},
};

use regex::Regex;

use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind, MouseEvent, KeyEvent, KeyEventKind},
    terminal
};
use ratatui::{
    text::{Text, Line},
    widgets::{ListItem, Paragraph, Widget, List, ListState, Block, Scrollbar, ScrollbarOrientation, ScrollbarState},
    layout::{Rect, Constraint, Direction, Layout, Margin},
    style::{Style, Stylize},
    symbols::scrollbar,
    Frame,
    buffer::Buffer,
};

mod menus;
use menus::MenuTabs;
mod functions;
use functions::process_queue;
mod app;
use app::App;
fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut conn = Client::connect("127.0.0.1:6600").unwrap();
    terminal::enable_raw_mode();

    let app_result = App::default().run(&mut terminal, &mut conn);
    terminal::disable_raw_mode();
    ratatui::restore();
    app_result
}


