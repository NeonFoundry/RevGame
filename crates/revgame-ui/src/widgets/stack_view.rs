use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::Theme;

/// Stack view widget
pub struct StackView<'a> {
    /// Stack entries (address, value)
    entries: Vec<(u32, u32)>,
    /// Current ESP value (for highlighting)
    esp: u32,
    /// Whether this panel is focused
    focused: bool,
    /// Theme
    theme: &'a Theme,
}

impl<'a> StackView<'a> {
    pub fn new(entries: Vec<(u32, u32)>, esp: u32, theme: &'a Theme) -> Self {
        Self {
            entries,
            esp,
            focused: false,
            theme,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<'a> Widget for StackView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border_style()
        };

        let block = Block::default()
            .title(" Stack ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        if self.entries.is_empty() {
            let line = Line::from(Span::styled("Empty stack", self.theme.muted_style()));
            buf.set_line(inner.x, inner.y, &line, inner.width);
            return;
        }

        for (i, (addr, value)) in self.entries.iter().enumerate() {
            if i >= inner.height as usize {
                break;
            }

            let y = inner.y + i as u16;
            let offset = addr.wrapping_sub(self.esp);
            let is_top = *addr == self.esp;

            let offset_str = if offset == 0 {
                "ESP".to_string()
            } else {
                format!("+{:02X}", offset)
            };

            let value_style = if is_top {
                self.theme.highlight()
            } else {
                self.theme.register_value()
            };

            let line = Line::from(vec![
                Span::styled(format!("{}: ", offset_str), self.theme.address()),
                Span::styled(format!("0x{:08X}", value), value_style),
            ]);

            buf.set_line(inner.x, y, &line, inner.width);
        }
    }
}
