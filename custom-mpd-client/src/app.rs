extern crate mpd;

use mpd::{Client};
use std::{
    net::TcpStream,
    time::{Duration, Instant, SystemTime},
};
use crossterm::{
    event::{self, Event, KeyCode, MouseEventKind, MouseEvent, KeyEvent, KeyEventKind},
    terminal
};
use ratatui::{
    text::{Text, Line},
    widgets::{ListItem, Paragraph, Widget, List, ListState, Block, Scrollbar, ScrollbarOrientation, ScrollbarState, Borders, BorderType},
    layout::{Rect, Constraint, Direction, Layout, Margin},
    style::{Style, Stylize, Color},
    symbols::scrollbar,
    Frame,
    buffer::Buffer,
};
use rand::Rng;


use crate::functions::process_queue;


pub struct CavaBars<'a> {
    data: &'a [u8],
}

impl<'a> CavaBars<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Widget for CavaBars<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let height = area.height as u8;

        let bar_width = 1;
        
        for (bar_index, &bar_height) in self.data.iter().enumerate() {
            let x = area.left() + (bar_index * bar_width) as u16;
            if x >= area.right() {
                break;
            }

            let actual_bar_height = bar_height.min(height * 4);

            for y in 0..area.height {
                let y_coord = area.bottom() - 1 -y;

                let remaining_dots_at_row_start = actual_bar_height.saturating_sub((y * 4) as u8);

                let dots_to_light = remaining_dots_at_row_start.min(4);

                if dots_to_light > 0 {
                    let char_code = match dots_to_light {
                        1 => 0x2801,
                        2 => 0x2803,
                        3 => 0x2807,
                        4 => 0x280F,
                        _ => 0x2800,
                    };
                    let symbol = char::from_u32(char_code).unwrap_or(' ');
                    let style = Style::default().fg(Color::LightGreen);
                    buf.get_mut(x, y_coord).set_symbol(symbol.to_string().as_str()).set_style(style);
                }
            }
        }

        Block::default()
            .borders(Borders::BOTTOM)
            .border_type(BorderType::Double)
            .title(" SPECTRUM ")
            .render(area, buf);
    }
}

#[derive(Debug, Default)]
pub struct App {
    pub queue: String,
    pub exit: bool,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub active_list: Vec<String>,
    pub list_state: ListState,
    pub last_scroll_time: Option<SystemTime>,
    pub spectrum_data: Vec<u8>,
}


impl App {
    pub fn scroll_down(&mut self, list_len: usize) {
        let new_scroll_time = SystemTime::now();
        let difference = new_scroll_time.duration_since(self.last_scroll_time.unwrap())
            .expect("failed!");
        if difference > Duration::from_millis(19) {
            self.last_scroll_time = Some(new_scroll_time);
            self.vertical_scroll = self.vertical_scroll.saturating_add(1);
            self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
           
            if list_len == 0 {
                self.list_state.select(None);
                return;
            }
             let current_index = self.list_state.selected().unwrap_or(0);
            let next_index = if current_index < list_len.saturating_sub(1) {
                current_index + 1
            } else {
                list_len.saturating_sub(1)
            };

            self.list_state.select(Some(next_index));
        }

    }

    pub fn scroll_up(&mut self, list_len: usize) {
        let new_scroll_time = SystemTime::now();
        let difference = new_scroll_time.duration_since(self.last_scroll_time.unwrap())
            .expect("failed!");
        if difference > Duration::from_millis(19) {
            self.last_scroll_time = Some(new_scroll_time);
            self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
            self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
            
            if list_len == 0 {
                self.list_state.select(None);
                return;
            }
            let current_index = self.list_state.selected().unwrap_or(0);
            let prev_index = current_index.saturating_sub(1);

            self.list_state.select(Some(prev_index));
        }
    }
    /*pub fn update_spectrum_data(&mut self, conn: &mut Client) -> Result<(), mpd::error::Error> {
        let status = conn.status()?;
        if status.state == PlaybackState::Play {
            let mut rng = rand::rng();
            for height in self.spectrum_data.iter_mut() {
                if rng.random_bool(0.5) {
                    *height = height.saturating_add(rng.random_range(0..=1));
                } else {
                    *height = height.saturating_add(rng.random_range(0..=2));
                }
                *height = *height.min(&mut 30).max(&mut 1);
            }

        } else {
            for height in self.spectrum_data.iter_mut() {
                *height = height.saturating_sub(1).max(1);
            }
        }
        Ok(())
    }*/
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal, conn: &mut Client) -> std::io::Result<()> {
        while !self.exit {
            self.last_scroll_time = Some(SystemTime::now());
            self.spectrum_data = vec![0; 40];

            //self.update_spectrum_data(conn);

            terminal.draw(|frame| {
                let size = frame.area();
                let mut conn = Client::connect("127.0.0.1:6600").unwrap();
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(59), Constraint::Percentage(1), Constraint::Percentage(40)])
                    .split(size);
                let list = &process_queue(conn);
                self.draw_list(frame, chunks[0], list);
                
                frame.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                        .symbols(scrollbar::VERTICAL)
                        .begin_symbol(None)
                        .track_symbol(None)
                        .end_symbol(None),
                    chunks[1].inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut self.vertical_scroll_state,
                );
                let cava_widget = CavaBars::new(&self.spectrum_data);
                frame.render_widget(cava_widget, chunks[2]);
               
            });
            self.handle_events();
       
        }
        Ok(())
            
    }
    pub fn draw_list(&mut self, frame: &mut Frame<'_>, layout: Rect, list: &Vec<String>) {
        let size = frame.area();
        if self.active_list != list.clone() {
            self.active_list = list.clone();
        }
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(list.len());

        let list_formatted = List::new(list.iter().map(String::as_str))
            .block(Block::bordered().title("Queue"))
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true);
                       
        frame.render_stateful_widget(list_formatted, layout, &mut self.list_state);

    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        //if event::poll(Duration::from_millis(250))? {
            match event::read()? {
   
                Event::Key(key_event) =>  {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event)
                    }
                }
                _ => {}
           }
        //}
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.scroll_up(self.active_list.len()),
            KeyCode::Down => self.scroll_down(self.active_list.len()), 
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

