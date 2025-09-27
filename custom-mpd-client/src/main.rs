extern crate mpd;

use mpd::Client;
use std::net::TcpStream;

use regex::Regex;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    text::{Text, Line},
    widgets::{ListItem, Paragraph, Widget, List, Block, Scrollbar, ScrollbarOrientation, ScrollbarState},
    layout::Rect,
    style::{Style, Stylize},
    Frame,
    buffer::Buffer,
};

mod menus;
use menus::MenuTabs;

fn process_queue(mut conn: Client) -> Vec<String> {
    let mut list_items = vec![];

    let queue_result = conn.queue();

    match queue_result {
        Ok(songs) => {
            //println!("Queue loaded successfully. Contains {} songs.", songs.len());

            for song in songs {
                let title = song.title.as_deref().unwrap_or("No Title");
                let artist = song.artist.as_deref().unwrap_or("No Artist");
                let queue_place = format!("{:?}", &song.place);
                let mut queue_place_pos = String::from("");

                let re = Regex::new(r"pos: (\d+)").unwrap();
                if let Some(caps) = re.captures(&queue_place) { 
                    if let Some(pos_str) = caps.get(1) {
                        if let Ok(pos_val) = pos_str.as_str().parse::<i32>() {
                            //println!("Pos val: {:?}", pos_val);
                            queue_place_pos = pos_val.to_string();
                        }
                    }
                }
                //println!("---");
                //println!("  Title: {}", title);
                //println!("  Artist: {}", artist);
                //println!("  Pos: {}", queue_place_pos);
                list_items.push(format!("{}: {} - {}", queue_place_pos, artist, title));

            }
        }
        Err(e) => {
            eprintln!("Failed to get queue: {:?}", e);
        }
    }

    return list_items;
}
fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut conn = Client::connect("127.0.0.1:6600").unwrap();
    
    let app_result = App::default().run(&mut terminal, conn);
    ratatui::restore();
    app_result
    //conn.volume(100).unwrap();
    //conn.play().unwrap();
    //println!("Queue: {:?}", conn.queue());

}

#[derive(Debug, Default)]
pub struct App {
    pub queue: String,
    pub exit: bool,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
}
impl App {
    const fn scroll_down(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    const fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
    }

    pub fn run(mut self, terminal: &mut ratatui::DefaultTerminal, conn: Client) -> std::io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| {
                let size = frame.area();
                let mut conn = Client::connect("127.0.0.1:6600").unwrap();
                let list = &process_queue(conn);

                self.vertical_scroll_state = self.vertical_scroll_state.content_length(list.len());

                let list_formatted = List::new(list.iter().map(String::as_str))
                    .block(Block::bordered().title("Queue"))
                    .highlight_style(Style::new().italic())
                    .highlight_symbol(">>")
                    .repeat_highlight_symbol(true);
                    /*.scroll((self.vertical_scroll as u16, 0));*/

                               
                frame.render_widget(list_formatted, size);
                /*frame.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                        .symbols(scrollbar::VERTICAL)
                        .begin_symbol(None)
                        .track_symbol(None)
                        .end_symbol(None),
                    chunks[2].inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut self.vertical_scroll_state,
                );
                */
            });
            self.handle_events();
        
        }
        Ok(())
            
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
