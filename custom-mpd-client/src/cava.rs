
use ratatui::{
    widgets::{Rect, Block, Border, BorderType},
    style::{Color, Style, Stylize},
    Frame, Terminal
};
use crossterm::{
    event::{self}
};

pub struct CavaBars<'a> {
    data: &'a [ua],
}

pub impl<'a> CavaBars<'a> {
    fn new(data: &'a [ua]) -> Self {
        Self { data }
    }
}

pub impl<'a> Widget for CavaBars<'a> {
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
                    buf.get_mut(x, y_coord).set_symbol(symbol).set_style(style);
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
