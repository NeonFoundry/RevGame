use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use revgame_core::emulator::{CpuState, Eflags};

use crate::Theme;

/// Register view widget
pub struct RegisterView<'a> {
    /// CPU state to display
    cpu: &'a CpuState,
    /// Set of changed register names (for highlighting)
    changed: &'a HashSet<String>,
    /// Whether this panel is focused
    focused: bool,
    /// Theme
    theme: &'a Theme,
}

impl<'a> RegisterView<'a> {
    pub fn new(cpu: &'a CpuState, changed: &'a HashSet<String>, theme: &'a Theme) -> Self {
        Self {
            cpu,
            changed,
            focused: false,
            theme,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl<'a> Widget for RegisterView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border_style()
        };

        let block = Block::default()
            .title(" Registers ")
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        let registers = [
            ("EAX", self.cpu.regs.eax),
            ("EBX", self.cpu.regs.ebx),
            ("ECX", self.cpu.regs.ecx),
            ("EDX", self.cpu.regs.edx),
            ("ESI", self.cpu.regs.esi),
            ("EDI", self.cpu.regs.edi),
            ("EBP", self.cpu.regs.ebp),
            ("ESP", self.cpu.regs.esp),
            ("EIP", self.cpu.eip),
        ];

        for (i, (name, value)) in registers.iter().enumerate() {
            if i >= inner.height as usize - 1 {
                break;
            }

            let y = inner.y + i as u16;
            let is_changed = self.changed.contains(*name);

            let value_style = if is_changed {
                self.theme.changed()
            } else {
                self.theme.register_value()
            };

            let line = Line::from(vec![
                Span::styled(format!("{}: ", name), self.theme.register_name()),
                Span::styled(format!("0x{:08X}", value), value_style),
            ]);

            buf.set_line(inner.x, y, &line, inner.width);
        }

        // Display flags on the last line
        let flags_y = inner.y + registers.len().min(inner.height as usize - 1) as u16;
        if flags_y < inner.y + inner.height {
            let flags_line = Line::from(vec![
                Span::styled("FLAGS: ", self.theme.register_name()),
                Span::styled(self.cpu.eflags.display(), self.theme.register_value()),
            ]);
            buf.set_line(inner.x, flags_y, &flags_line, inner.width);
        }
    }
}
