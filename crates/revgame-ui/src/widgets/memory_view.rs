use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::Theme;

/// Memory hex view widget
pub struct MemoryView<'a> {
    /// Memory data to display
    data: Option<&'a [u8]>,
    /// Start address
    start_addr: u32,
    /// Whether this panel is focused
    focused: bool,
    /// Theme
    theme: &'a Theme,
    /// Bytes per row
    bytes_per_row: usize,
}

impl<'a> MemoryView<'a> {
    pub fn new(data: Option<&'a [u8]>, start_addr: u32, theme: &'a Theme) -> Self {
        Self {
            data,
            start_addr,
            focused: false,
            theme,
            bytes_per_row: 16,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn bytes_per_row(mut self, count: usize) -> Self {
        self.bytes_per_row = count;
        self
    }
}

impl<'a> Widget for MemoryView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border_style()
        };

        let block = Block::default()
            .title(" Memory ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        let Some(data) = self.data else {
            let line = Line::from(Span::styled("No memory loaded", self.theme.muted_style()));
            buf.set_line(inner.x, inner.y, &line, inner.width);
            return;
        };

        let rows = data.chunks(self.bytes_per_row);

        for (row_idx, row_data) in rows.enumerate() {
            if row_idx >= inner.height as usize {
                break;
            }

            let y = inner.y + row_idx as u16;
            let addr = self.start_addr + (row_idx * self.bytes_per_row) as u32;

            let mut spans = Vec::new();

            // Address
            spans.push(Span::styled(
                format!("{:08X}: ", addr),
                self.theme.address(),
            ));

            // Hex bytes
            for (i, byte) in row_data.iter().enumerate() {
                spans.push(Span::styled(format!("{:02X}", byte), self.theme.hex_byte()));
                if i < row_data.len() - 1 {
                    spans.push(Span::raw(" "));
                }
            }

            // Padding for incomplete rows
            for _ in row_data.len()..self.bytes_per_row {
                spans.push(Span::raw("   "));
            }

            spans.push(Span::raw("  "));

            // ASCII representation
            for byte in row_data {
                let ch = if *byte >= 0x20 && *byte < 0x7F {
                    *byte as char
                } else {
                    '.'
                };
                spans.push(Span::styled(ch.to_string(), self.theme.ascii()));
            }

            let line = Line::from(spans);
            buf.set_line(inner.x, y, &line, inner.width);
        }
    }
}
