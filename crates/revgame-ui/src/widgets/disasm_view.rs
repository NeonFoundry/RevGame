use std::collections::HashSet;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use revgame_core::emulator::DisassemblyLine;

use crate::{Theme, SyntaxHighlighter};

/// Disassembly view widget
pub struct DisasmView<'a> {
    /// Disassembly lines to display
    lines: &'a [DisassemblyLine],
    /// Current instruction pointer
    current_eip: u32,
    /// Set of breakpoint addresses
    breakpoints: &'a HashSet<u32>,
    /// Currently selected line index
    selected: usize,
    /// Whether this panel is focused
    focused: bool,
    /// Theme
    theme: &'a Theme,
    /// Widget title
    title: &'a str,
    /// Syntax highlighter
    syntax_highlighter: &'a SyntaxHighlighter,
}

impl<'a> DisasmView<'a> {
    pub fn new(
        lines: &'a [DisassemblyLine],
        current_eip: u32,
        breakpoints: &'a HashSet<u32>,
        theme: &'a Theme,
        syntax_highlighter: &'a SyntaxHighlighter,
    ) -> Self {
        Self {
            lines,
            current_eip,
            breakpoints,
            selected: 0,
            focused: false,
            theme,
            title: " Disassembly ",
            syntax_highlighter,
        }
    }

    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }
}

impl<'a> Widget for DisasmView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.focused {
            self.theme.border_focused()
        } else {
            self.theme.border_style()
        };

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(border_style);

        let inner = block.inner(area);
        block.render(area, buf);

        for (i, line) in self.lines.iter().enumerate() {
            if i >= inner.height as usize {
                break;
            }

            let y = inner.y + i as u16;
            let is_current = line.address == self.current_eip;
            let is_selected = i == self.selected;
            let has_bp = self.breakpoints.contains(&line.address);

            // Build the line with colored spans
            let mut spans = Vec::new();

            // Breakpoint indicator
            if has_bp {
                spans.push(Span::styled("●", self.theme.breakpoint()));
            } else {
                spans.push(Span::raw(" "));
            }

            // Current instruction arrow
            if is_current {
                spans.push(Span::styled("►", self.theme.current_instruction()));
            } else {
                spans.push(Span::raw(" "));
            }

            spans.push(Span::raw(" "));

            // Address
            spans.push(Span::styled(
                format!("{:08X}", line.address),
                self.theme.address(),
            ));

            spans.push(Span::raw(": "));

            // Instruction text with syntax highlighting
            if is_current || is_selected {
                // For current/selected lines, use theme colors
                let instr_style = if is_current {
                    self.theme.current_instruction()
                } else {
                    self.theme.selected()
                };
                spans.push(Span::styled(&line.text, instr_style));
            } else {
                // Apply syntax highlighting
                // Split instruction into mnemonic and operands
                if let Some((mnemonic, operands)) = line.text.split_once(' ') {
                    let mnemonic_color = self.syntax_highlighter.mnemonic_color(mnemonic);
                    let operand_color = self.syntax_highlighter.operand_color(mnemonic);

                    spans.push(Span::styled(
                        mnemonic.to_string(),
                        Style::default().fg(mnemonic_color).add_modifier(Modifier::BOLD),
                    ));
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        operands.to_string(),
                        Style::default().fg(operand_color),
                    ));
                } else {
                    // No operands (e.g., NOP, HLT, RET)
                    let mnemonic_color = self.syntax_highlighter.mnemonic_color(&line.text);
                    spans.push(Span::styled(
                        line.text.clone(),
                        Style::default().fg(mnemonic_color).add_modifier(Modifier::BOLD),
                    ));
                }
            }

            // Pad and render
            let line_widget = Line::from(spans);

            // Apply selection background if selected
            let line_style = if is_selected && !is_current {
                self.theme.selected()
            } else {
                Style::default()
            };

            // Render background for selected line
            if is_selected {
                for x in inner.x..inner.x + inner.width {
                    buf[(x, y)].set_style(line_style);
                }
            }

            // Render the line content
            buf.set_line(inner.x, y, &line_widget, inner.width);
        }
    }
}
