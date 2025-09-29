extern crate mpd;

use mpd::{Client, State};
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
    symbols::{scrollbar, line},
    Frame,
    buffer::Buffer,
};
use rand::Rng;

use crate::functions::process_queue;
use crate::spectrum::SpectrumBars;

#[derive(Debug)]
pub struct App {
    pub queue: String,
    pub exit: bool,
    pub pause: bool,
    pub vertical_scroll_state: ScrollbarState,
    pub vertical_scroll: usize,
    pub active_list: Vec<String>,
    pub list_state: ListState,
    pub last_scroll_time: Option<SystemTime>,
    pub spectrum_data: Vec<u8>,
    pub spectrum_width: u16, 
    pub target_heights: Vec<f32>,
    pub start_time: Instant,
}

impl Default for App {
    fn default() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        let num_bars = 40;

        Self { 
            queue: String::new(),
            vertical_scroll_state: ratatui::widgets::ScrollbarState::default(),
            vertical_scroll: 0,
            active_list: Vec::new(),
            last_scroll_time: None,

            exit: false,
            pause: false,
            list_state, 
            spectrum_data: vec![1; num_bars],
            spectrum_width: num_bars as u16,
            target_heights: vec![1.0; num_bars], 
            start_time: Instant::now(),
        }
    }
}
impl App {
    pub fn scroll_down(&mut self, list_len: usize) {
        let new_scroll_time = SystemTime::now();
        let difference = new_scroll_time.duration_since(self.last_scroll_time.unwrap())
            .expect("failed!");
        if difference > Duration::from_millis(20) {
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
        if difference > Duration::from_millis(20) {
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
    pub fn update_spectrum_data(&mut self, conn: &mut Client) -> Result<(), mpd::error::Error> {
        const MAX_HEIGHT: f32 = 400.0;
        const ATTACK_FACTOR: f32 = 0.325; 
        const DECAY_FACTOR: f32 = 0.10;  
        const WAVE_MULT_FACTOR: f32 = 7.0;
        let status = conn.status()?;
        let time = self.start_time.elapsed().as_secs_f32();
        let num_bars_f32 = self.spectrum_data.len() as f32;
        let mut rng = rand::rng();

        for i in 0..self.spectrum_data.len() {
            let bar_index_f32 = i as f32;
            let current_smoothed_height = self.target_heights[i];
            
            if status.state == State::Play {
                // bass
                let wave_low = ((time / 5.0).sin() * 0.5 + 0.5) * (5.0 * WAVE_MULT_FACTOR) * (rng.random_range(0.75..1.0) as f32); 

                // mid-range/rhythm 
                let wave_mid_freq = (time.cos() * 0.5 + 0.5) * (15.0 * WAVE_MULT_FACTOR) * (rng.random_range(0.75..1.0) as f32); 
                let wave_mid = ((time * 3.0 + bar_index_f32 / 2.0).sin() * 0.5 + 0.5) * wave_mid_freq;
                
                // treble/synth
                let treble_weight = (bar_index_f32 / num_bars_f32).powi(2) * (20.0 * WAVE_MULT_FACTOR) * (rng.random_range(0.75..1.0) as f32);
                let wave_high = ((time * 10.0 + bar_index_f32 * 5.0).cos() * 0.5 + 0.5) * treble_weight;

                let raw_target_level = 5.0 + wave_low + wave_mid + wave_high;

                // beat 
                let beat_spike = if rng.random_bool(0.05) { 
                    rng.random_range(10.0..30.0) 
                } else { 
                    0.0 
                };
 
                let effective_target = (raw_target_level + beat_spike).min(MAX_HEIGHT).max(1.0); 

                let difference = effective_target - current_smoothed_height;
                let new_height: f32;

                if difference > 0.0 {
                    // attack
                    new_height = current_smoothed_height + difference * ATTACK_FACTOR;
                } else {
                    // decay
                    new_height = current_smoothed_height + difference * DECAY_FACTOR;
                }
                
                self.target_heights[i] = new_height.min(MAX_HEIGHT).max(1.0); 
                self.spectrum_data[i] = self.target_heights[i] as u8; 

            } else {
                let decay_rate = 0.30;
                let new_smoothed_height = current_smoothed_height + (1.0 - current_smoothed_height) * decay_rate;

                self.target_heights[i] = new_smoothed_height.max(1.0);
                self.spectrum_data[i] = self.target_heights[i] as u8;
            }
        }
        Ok(())
    }    
    pub fn run(&mut self, terminal: &mut ratatui::DefaultTerminal, conn: &mut Client) -> std::io::Result<()> {
        while !self.exit {
            self.last_scroll_time = Some(SystemTime::now());
            //self.spectrum_data = vec![0; 40];
            self.update_spectrum_data(conn);
            terminal.draw(|frame| {
                let size = frame.area();
                let mut conn = Client::connect("127.0.0.1:6600").unwrap();
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(59), Constraint::Percentage(1), Constraint::Percentage(40)])
                    .split(size);
                let list = &process_queue(conn);
                self.draw_list(frame, chunks[0], list);
                if self.spectrum_width != chunks[2].width {
                    self.spectrum_data.resize(chunks[2].width as usize, 0);
                    self.target_heights.resize(chunks[2].width as usize, 0 as f32);
                    self.spectrum_width = chunks[2].width;
                }
                frame.render_stateful_widget(
                    Scrollbar::new(ScrollbarOrientation::VerticalLeft)
                        .symbols(scrollbar::VERTICAL)
                        .begin_symbol(Some("▲"))
                        .end_symbol(Some("▼"))
                        .track_symbol(Some(line::VERTICAL)),
                    chunks[1].inner(Margin {
                        vertical: 1,
                        horizontal: 0,
                    }),
                    &mut self.vertical_scroll_state,
                );
                let cava_widget = SpectrumBars::new(&self.spectrum_data);
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
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
   
                Event::Key(key_event) =>  {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event)
                    }
                }
                _ => {}
           }
        }
        Ok(())
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('p') => self.play_pause(),
            KeyCode::Up => self.scroll_up(self.active_list.len()),
            KeyCode::Down => self.scroll_down(self.active_list.len()), 
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;    
    }
    fn play_pause(&mut self) {
        let mut conn = Client::connect("127.0.0.1:6600").unwrap();
        if self.pause == true {
            conn.pause(true);
            self.pause = false;
        } else {
            conn.pause(false);
            self.pause = true;
        }
    }
}

