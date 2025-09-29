extern crate mpd;

use ratatui::{
    widgets::{Widget, Block, Borders, BorderType},
    layout::Rect,
    style::{Style, Color},
};

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
                    if let Some(cell) = buf.cell_mut((x, y_coord)) {
                        cell.set_symbol(&symbol.to_string());
                        cell.set_style(style);
                    } 
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

