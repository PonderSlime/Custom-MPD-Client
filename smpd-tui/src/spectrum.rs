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

pub struct SpectrumBars<'a> {
    data: &'a [u8],
}

impl<'a> SpectrumBars<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }
}

impl<'a> Widget for SpectrumBars<'a> {
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
                        1 => 0x2582,
                        2 => 0x2584,
                        3 => 0x2586,
                        4 => 0x2588,
                        //5 => 0x2586,
                        //6 => 0x2587,
                        //7 => 0x2588,
                        _ => 0x200C,
                    };
                    let symbol = char::from_u32(char_code).unwrap_or(' ');
                    let style = Style::default().fg(Color::LightGreen);
                    buf.get_mut(x, y_coord).set_symbol(&symbol.to_string().as_str()).set_style(style);
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

