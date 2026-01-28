use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

use crate::tutorial::{ArrowDirection, HighlightTarget, TutorialStep};
use crate::Theme;

/// Layout information for positioning tutorial elements
#[derive(Debug, Clone, Default)]
pub struct DebuggerLayout {
    pub disassembly: Rect,
    pub registers: Rect,
    pub memory: Rect,
    pub stack: Rect,
    pub command: Rect,
    pub status: Rect,
}

/// Tutorial overlay widget
pub struct TutorialOverlay<'a> {
    step: &'a TutorialStep,
    layout: &'a DebuggerLayout,
    theme: &'a Theme,
    progress: (usize, usize),
}

impl<'a> TutorialOverlay<'a> {
    pub fn new(
        step: &'a TutorialStep,
        layout: &'a DebuggerLayout,
        theme: &'a Theme,
        progress: (usize, usize),
    ) -> Self {
        Self {
            step,
            layout,
            theme,
            progress,
        }
    }

    /// Get the rect for a highlight target
    fn get_target_rect(&self, target: HighlightTarget) -> Rect {
        match target {
            HighlightTarget::Disassembly => self.layout.disassembly,
            HighlightTarget::Registers => self.layout.registers,
            HighlightTarget::Memory => self.layout.memory,
            HighlightTarget::Stack => self.layout.stack,
            HighlightTarget::CommandLine => self.layout.command,
            HighlightTarget::StatusBar => self.layout.status,
            HighlightTarget::CurrentInstruction => {
                // First line of disassembly area
                Rect {
                    x: self.layout.disassembly.x,
                    y: self.layout.disassembly.y + 1,
                    width: self.layout.disassembly.width,
                    height: 1,
                }
            }
            HighlightTarget::Flags => {
                // Bottom of registers area
                Rect {
                    x: self.layout.registers.x,
                    y: self.layout.registers.y + self.layout.registers.height.saturating_sub(2),
                    width: self.layout.registers.width,
                    height: 1,
                }
            }
        }
    }

    /// Draw a highlight border around a rect
    fn draw_highlight(&self, area: Rect, buf: &mut Buffer) {
        let highlight_color = Color::Rgb(249, 226, 175); // Yellow

        // Top border
        for x in area.x..area.x + area.width {
            if x < buf.area().width && area.y > 0 {
                let cell = &mut buf[(x, area.y.saturating_sub(1))];
                cell.set_char('▀');
                cell.set_fg(highlight_color);
            }
        }

        // Bottom border
        let bottom_y = area.y + area.height;
        for x in area.x..area.x + area.width {
            if x < buf.area().width && bottom_y < buf.area().height {
                let cell = &mut buf[(x, bottom_y)];
                cell.set_char('▄');
                cell.set_fg(highlight_color);
            }
        }

        // Left border
        for y in area.y..area.y + area.height {
            if area.x > 0 && y < buf.area().height {
                let cell = &mut buf[(area.x.saturating_sub(1), y)];
                cell.set_char('▐');
                cell.set_fg(highlight_color);
            }
        }

        // Right border
        let right_x = area.x + area.width;
        for y in area.y..area.y + area.height {
            if right_x < buf.area().width && y < buf.area().height {
                let cell = &mut buf[(right_x, y)];
                cell.set_char('▌');
                cell.set_fg(highlight_color);
            }
        }
    }

    /// Draw an arrow pointing to a rect
    fn draw_arrow(&self, target: Rect, direction: ArrowDirection, buf: &mut Buffer) {
        let arrow_color = Color::Rgb(166, 227, 161); // Green

        match direction {
            ArrowDirection::Right => {
                // Arrow pointing right (from left of target)
                let arrow_x = target.x.saturating_sub(4);
                let arrow_y = target.y + target.height / 2;

                if arrow_y < buf.area().height {
                    let chars = ['─', '─', '─', '►'];
                    for (i, ch) in chars.iter().enumerate() {
                        let x = arrow_x + i as u16;
                        if x < buf.area().width && x < target.x {
                            let cell = &mut buf[(x, arrow_y)];
                            cell.set_char(*ch);
                            cell.set_style(Style::default().fg(arrow_color).add_modifier(Modifier::BOLD));
                        }
                    }
                }
            }
            ArrowDirection::Left => {
                // Arrow pointing left (from right of target)
                let arrow_x = target.x + target.width;
                let arrow_y = target.y + target.height / 2;

                if arrow_y < buf.area().height {
                    let chars = ['◄', '─', '─', '─'];
                    for (i, ch) in chars.iter().enumerate() {
                        let x = arrow_x + i as u16;
                        if x < buf.area().width {
                            let cell = &mut buf[(x, arrow_y)];
                            cell.set_char(*ch);
                            cell.set_style(Style::default().fg(arrow_color).add_modifier(Modifier::BOLD));
                        }
                    }
                }
            }
            ArrowDirection::Down => {
                // Arrow pointing down (from above target)
                let arrow_x = target.x + target.width / 2;
                let arrow_y = target.y.saturating_sub(3);

                if arrow_x < buf.area().width {
                    let chars = ['│', '│', '▼'];
                    for (i, ch) in chars.iter().enumerate() {
                        let y = arrow_y + i as u16;
                        if y < buf.area().height && y < target.y {
                            let cell = &mut buf[(arrow_x, y)];
                            cell.set_char(*ch);
                            cell.set_style(Style::default().fg(arrow_color).add_modifier(Modifier::BOLD));
                        }
                    }
                }
            }
            ArrowDirection::Up => {
                // Arrow pointing up (from below target)
                let arrow_x = target.x + target.width / 2;
                let arrow_y = target.y + target.height;

                if arrow_x < buf.area().width {
                    let chars = ['▲', '│', '│'];
                    for (i, ch) in chars.iter().enumerate() {
                        let y = arrow_y + i as u16;
                        if y < buf.area().height {
                            let cell = &mut buf[(arrow_x, y)];
                            cell.set_char(*ch);
                            cell.set_style(Style::default().fg(arrow_color).add_modifier(Modifier::BOLD));
                        }
                    }
                }
            }
        }
    }

    /// Calculate dialog box position
    fn calculate_dialog_rect(&self, area: Rect) -> Rect {
        // Count lines in text to determine height
        let text_lines = self.step.text.lines().count() as u16;
        let title_line = 1u16;
        let hint_line = if self.step.hint.is_some() { 2 } else { 0 };
        let progress_line = 1u16;
        let padding = 4u16; // borders + margins

        let height = (text_lines + title_line + hint_line + progress_line + padding).min(area.height - 4);
        let width = 50.min(area.width - 10);

        // Position based on highlight target
        let (x, y) = if let Some(target) = self.step.highlight {
            let target_rect = self.get_target_rect(target);

            // Try to position opposite to arrow direction
            match self.step.arrow {
                Some(ArrowDirection::Right) => {
                    // Target is on right, put dialog on left
                    let x = area.x + 2;
                    let y = target_rect.y.saturating_sub(2);
                    (x, y.max(1))
                }
                Some(ArrowDirection::Left) => {
                    // Target is on left, put dialog on right
                    let x = area.width.saturating_sub(width + 2);
                    let y = target_rect.y.saturating_sub(2);
                    (x, y.max(1))
                }
                Some(ArrowDirection::Up) | Some(ArrowDirection::Down) => {
                    // Center horizontally
                    let x = (area.width - width) / 2;
                    let y = (area.height - height) / 2;
                    (x, y)
                }
                None => {
                    // Default: center
                    let x = (area.width - width) / 2;
                    let y = (area.height - height) / 2;
                    (x, y)
                }
            }
        } else {
            // No highlight, center the dialog
            let x = (area.width - width) / 2;
            let y = (area.height - height) / 2;
            (x, y)
        };

        Rect {
            x,
            y,
            width,
            height,
        }
    }
}

impl<'a> Widget for TutorialOverlay<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw highlight if specified
        if let Some(target) = self.step.highlight {
            let target_rect = self.get_target_rect(target);
            self.draw_highlight(target_rect, buf);

            // Draw arrow if specified
            if let Some(direction) = self.step.arrow {
                self.draw_arrow(target_rect, direction, buf);
            }
        }

        // Calculate dialog position
        let dialog_rect = self.calculate_dialog_rect(area);

        // Clear the dialog area
        Clear.render(dialog_rect, buf);

        // Build dialog content
        let mut lines: Vec<Line> = Vec::new();

        // Title
        lines.push(Line::from(Span::styled(
            &self.step.title,
            Style::default()
                .fg(Color::Rgb(137, 180, 250)) // Blue
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Main text
        for text_line in self.step.text.lines() {
            lines.push(Line::from(Span::styled(
                text_line,
                Style::default().fg(Color::Rgb(205, 214, 244)), // Light text
            )));
        }

        // Hint if present
        if let Some(ref hint) = self.step.hint {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!("→ {}", hint),
                Style::default()
                    .fg(Color::Rgb(166, 227, 161)) // Green
                    .add_modifier(Modifier::BOLD),
            )));
        }

        // Progress indicator
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Step {}/{} • Press ESC to skip tutorial", self.progress.0, self.progress.1),
            Style::default().fg(Color::Rgb(127, 132, 156)), // Muted
        )));

        // Create the dialog
        let dialog = Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(137, 180, 250)))
                    .style(Style::default().bg(Color::Rgb(30, 30, 46))),
            )
            .wrap(Wrap { trim: false });

        dialog.render(dialog_rect, buf);
    }
}
