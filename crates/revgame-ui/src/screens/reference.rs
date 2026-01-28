use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use revgame_core::reference::{InstructionCategory, InstructionInfo, InstructionReference};

use crate::{app::App, theme::Theme};

/// Reference viewer state
pub struct ReferenceState {
    pub reference: InstructionReference,
    pub selected_category: usize,
    pub selected_instruction: usize,
    pub view_mode: ReferenceViewMode,
}

/// What the user is currently viewing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceViewMode {
    CategoryList,
    InstructionList,
    InstructionDetail,
}

impl Default for ReferenceState {
    fn default() -> Self {
        Self {
            reference: InstructionReference::new(),
            selected_category: 0,
            selected_instruction: 0,
            view_mode: ReferenceViewMode::CategoryList,
        }
    }
}

impl ReferenceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn navigate_up(&mut self) {
        match self.view_mode {
            ReferenceViewMode::CategoryList => {
                if self.selected_category > 0 {
                    self.selected_category -= 1;
                }
            }
            ReferenceViewMode::InstructionList => {
                if self.selected_instruction > 0 {
                    self.selected_instruction -= 1;
                }
            }
            ReferenceViewMode::InstructionDetail => {}
        }
    }

    pub fn navigate_down(&mut self) {
        match self.view_mode {
            ReferenceViewMode::CategoryList => {
                let max = InstructionCategory::all().len() - 1;
                if self.selected_category < max {
                    self.selected_category += 1;
                }
            }
            ReferenceViewMode::InstructionList => {
                let category = InstructionCategory::all()[self.selected_category];
                let max = self.reference.by_category(category).len() - 1;
                if self.selected_instruction < max {
                    self.selected_instruction += 1;
                }
            }
            ReferenceViewMode::InstructionDetail => {}
        }
    }

    pub fn enter(&mut self) {
        match self.view_mode {
            ReferenceViewMode::CategoryList => {
                self.view_mode = ReferenceViewMode::InstructionList;
                self.selected_instruction = 0;
            }
            ReferenceViewMode::InstructionList => {
                self.view_mode = ReferenceViewMode::InstructionDetail;
            }
            ReferenceViewMode::InstructionDetail => {}
        }
    }

    pub fn back(&mut self) {
        match self.view_mode {
            ReferenceViewMode::CategoryList => {}
            ReferenceViewMode::InstructionList => {
                self.view_mode = ReferenceViewMode::CategoryList;
            }
            ReferenceViewMode::InstructionDetail => {
                self.view_mode = ReferenceViewMode::InstructionList;
            }
        }
    }

    pub fn get_current_instruction(&self) -> Option<&InstructionInfo> {
        if self.view_mode == ReferenceViewMode::InstructionList
            || self.view_mode == ReferenceViewMode::InstructionDetail
        {
            let category = InstructionCategory::all()[self.selected_category];
            let instructions = self.reference.by_category(category);
            instructions.get(self.selected_instruction).copied()
        } else {
            None
        }
    }
}

/// Render the instruction reference screen
pub fn render_reference(frame: &mut Frame, _app: &App, state: &ReferenceState, theme: &Theme) {
    let area = frame.area();

    match state.view_mode {
        ReferenceViewMode::CategoryList => render_category_list(frame, state, theme, area),
        ReferenceViewMode::InstructionList => render_instruction_list(frame, state, theme, area),
        ReferenceViewMode::InstructionDetail => {
            render_instruction_detail(frame, state, theme, area)
        }
    }
}

fn render_category_list(
    frame: &mut Frame,
    state: &ReferenceState,
    theme: &Theme,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Categories
            Constraint::Length(1),  // Help
        ])
        .split(area);

    // Header
    let header = Paragraph::new(vec![Line::from(Span::styled(
        "📚 x86 INSTRUCTION REFERENCE",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, chunks[0]);

    // Categories
    let categories = InstructionCategory::all();
    let items: Vec<ListItem> = categories
        .iter()
        .enumerate()
        .map(|(idx, cat)| {
            let count = state.reference.by_category(*cat).len();
            let text = format!("  {} ({} instructions)", cat.name(), count);

            let style = if idx == state.selected_category {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.normal()
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Select Category ")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, chunks[1]);

    // Help
    let help = Paragraph::new(" [↑↓] Navigate  [Enter] Select  [Esc] Back ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

fn render_instruction_list(
    frame: &mut Frame,
    state: &ReferenceState,
    theme: &Theme,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Instructions
            Constraint::Length(1),  // Help
        ])
        .split(area);

    // Header
    let category = InstructionCategory::all()[state.selected_category];
    let header = Paragraph::new(vec![Line::from(Span::styled(
        format!("📚 {} Instructions", category.name()),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, chunks[0]);

    // Instructions
    let instructions = state.reference.by_category(category);
    let items: Vec<ListItem> = instructions
        .iter()
        .enumerate()
        .map(|(idx, info)| {
            let text = format!("  {:<8} - {}", info.mnemonic, info.name);

            let style = if idx == state.selected_instruction {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.normal()
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Select Instruction ")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, chunks[1]);

    // Help
    let help = Paragraph::new(" [↑↓] Navigate  [Enter] Details  [Esc] Back ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

fn render_instruction_detail(
    frame: &mut Frame,
    state: &ReferenceState,
    theme: &Theme,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    if let Some(info) = state.get_current_instruction() {
        let mut lines = Vec::new();

        // Mnemonic
        lines.push(Line::from(Span::styled(
            format!("{} - {}", info.mnemonic, info.name),
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Description
        lines.push(Line::from(Span::styled("Description:", theme.highlight())));
        lines.push(Line::from(Span::styled(info.description, theme.normal())));
        lines.push(Line::from(""));

        // Syntax
        lines.push(Line::from(Span::styled("Syntax:", theme.highlight())));
        for syntax in &info.syntax {
            lines.push(Line::from(Span::styled(
                format!("  {}", syntax),
                Style::default().fg(theme.success),
            )));
        }
        lines.push(Line::from(""));

        // Examples
        lines.push(Line::from(Span::styled("Examples:", theme.highlight())));
        for example in &info.examples {
            lines.push(Line::from(Span::styled(
                format!("  {}", example),
                theme.muted_style(),
            )));
        }
        lines.push(Line::from(""));

        // Flags
        lines.push(Line::from(Span::styled("Flags Affected:", theme.highlight())));
        lines.push(Line::from(Span::styled(
            format!("  {}", info.flags_affected),
            theme.normal(),
        )));

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {} ", info.mnemonic))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(para, chunks[0]);
    }

    // Help
    let help = Paragraph::new(" [Esc] Back ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[1]);
}
