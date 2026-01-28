use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use revgame_core::puzzle::{PuzzleCategory, PuzzleListItem};

use crate::{app::App, theme::Theme};

/// Puzzle select state
pub struct PuzzleSelectState {
    pub categories: Vec<PuzzleCategory>,
    pub selected_category: usize,
    pub selected_puzzle: usize,
    pub view_mode: SelectViewMode,
}

/// What the user is viewing
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectViewMode {
    CategoryList,
    PuzzleList,
    PuzzleDetail,
}

impl Default for PuzzleSelectState {
    fn default() -> Self {
        Self {
            categories: Vec::new(),
            selected_category: 0,
            selected_puzzle: 0,
            view_mode: SelectViewMode::CategoryList,
        }
    }
}

impl PuzzleSelectState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn navigate_up(&mut self) {
        match self.view_mode {
            SelectViewMode::CategoryList => {
                if self.selected_category > 0 {
                    self.selected_category -= 1;
                }
            }
            SelectViewMode::PuzzleList | SelectViewMode::PuzzleDetail => {
                if self.selected_puzzle > 0 {
                    self.selected_puzzle -= 1;
                }
            }
        }
    }

    pub fn navigate_down(&mut self) {
        match self.view_mode {
            SelectViewMode::CategoryList => {
                if self.selected_category < self.categories.len().saturating_sub(1) {
                    self.selected_category += 1;
                }
            }
            SelectViewMode::PuzzleList | SelectViewMode::PuzzleDetail => {
                if let Some(category) = self.categories.get(self.selected_category) {
                    if self.selected_puzzle < category.puzzles.len().saturating_sub(1) {
                        self.selected_puzzle += 1;
                    }
                }
            }
        }
    }

    pub fn enter(&mut self) {
        match self.view_mode {
            SelectViewMode::CategoryList => {
                self.view_mode = SelectViewMode::PuzzleList;
                self.selected_puzzle = 0;
            }
            SelectViewMode::PuzzleList => {
                self.view_mode = SelectViewMode::PuzzleDetail;
            }
            SelectViewMode::PuzzleDetail => {}
        }
    }

    pub fn back(&mut self) {
        match self.view_mode {
            SelectViewMode::CategoryList => {}
            SelectViewMode::PuzzleList => {
                self.view_mode = SelectViewMode::CategoryList;
            }
            SelectViewMode::PuzzleDetail => {
                self.view_mode = SelectViewMode::PuzzleList;
            }
        }
    }

    pub fn get_selected_puzzle(&self) -> Option<&PuzzleListItem> {
        if matches!(
            self.view_mode,
            SelectViewMode::PuzzleList | SelectViewMode::PuzzleDetail
        ) {
            self.categories
                .get(self.selected_category)?
                .puzzles
                .get(self.selected_puzzle)
        } else {
            None
        }
    }

    pub fn load_puzzles(&mut self, puzzles_dir: &std::path::Path) -> Result<(), String> {
        self.categories = revgame_core::puzzle::load_puzzle_list(puzzles_dir)?;
        self.selected_category = 0;
        self.selected_puzzle = 0;
        self.view_mode = SelectViewMode::CategoryList;
        Ok(())
    }
}

/// Render the puzzle select screen
pub fn render_puzzle_select(frame: &mut Frame, _app: &App, state: &PuzzleSelectState, theme: &Theme) {
    let area = frame.area();

    match state.view_mode {
        SelectViewMode::CategoryList => render_category_list(frame, state, theme, area),
        SelectViewMode::PuzzleList => render_puzzle_list(frame, state, theme, area),
        SelectViewMode::PuzzleDetail => render_puzzle_detail(frame, state, theme, area),
    }
}

fn render_category_list(
    frame: &mut Frame,
    state: &PuzzleSelectState,
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
        "🎮 PUZZLE SELECT",
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, chunks[0]);

    // Categories
    if state.categories.is_empty() {
        let no_puzzles = Paragraph::new("No puzzles found!\n\nMake sure the puzzles directory exists.")
            .style(theme.muted_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(no_puzzles, chunks[1]);
    } else {
        let items: Vec<ListItem> = state
            .categories
            .iter()
            .enumerate()
            .map(|(idx, cat)| {
                let count = cat.puzzles.len();
                let text = format!("  {} ({} puzzles)", cat.display_name, count);

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
    }

    // Help
    let help = Paragraph::new(" [↑↓] Navigate  [Enter] Select  [Esc] Back to Menu ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

fn render_puzzle_list(
    frame: &mut Frame,
    state: &PuzzleSelectState,
    theme: &Theme,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Puzzles
            Constraint::Length(1),  // Help
        ])
        .split(area);

    // Header
    let category = &state.categories[state.selected_category];
    let header = Paragraph::new(vec![Line::from(Span::styled(
        format!("🎮 {} Puzzles", category.display_name),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ))])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, chunks[0]);

    // Puzzles
    let items: Vec<ListItem> = category
        .puzzles
        .iter()
        .enumerate()
        .map(|(idx, puzzle)| {
            let difficulty_stars = "★".repeat(puzzle.difficulty as usize);
            let lock_icon = if puzzle.is_locked { "🔒 " } else { "" };
            let text = format!(
                "  {}{} - {} ({})",
                lock_icon, puzzle.title, puzzle.brief, difficulty_stars
            );

            let style = if idx == state.selected_puzzle {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else if puzzle.is_locked {
                theme.muted_style()
            } else {
                theme.normal()
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title(" Select Puzzle ")
            .borders(Borders::ALL),
    );

    frame.render_widget(list, chunks[1]);

    // Help
    let help = Paragraph::new(" [↑↓] Navigate  [Enter] View Details  [S] Start  [Esc] Back ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
}

fn render_puzzle_detail(
    frame: &mut Frame,
    state: &PuzzleSelectState,
    theme: &Theme,
    area: ratatui::layout::Rect,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(1)])
        .split(area);

    if let Some(puzzle) = state.get_selected_puzzle() {
        let mut lines = Vec::new();

        // Title
        lines.push(Line::from(Span::styled(
            &puzzle.title,
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Difficulty
        let difficulty_stars = "★".repeat(puzzle.difficulty as usize);
        lines.push(Line::from(vec![
            Span::styled("Difficulty: ", theme.highlight()),
            Span::styled(
                format!("{} ({})", difficulty_stars, puzzle.difficulty),
                Style::default().fg(theme.success),
            ),
        ]));
        lines.push(Line::from(""));

        // Category
        lines.push(Line::from(vec![
            Span::styled("Category: ", theme.highlight()),
            Span::styled(&puzzle.category, theme.normal()),
        ]));
        lines.push(Line::from(""));

        // Description
        lines.push(Line::from(Span::styled("Description:", theme.highlight())));
        lines.push(Line::from(Span::styled(&puzzle.brief, theme.normal())));
        lines.push(Line::from(""));

        // Prerequisites
        if !puzzle.prerequisites.is_empty() {
            lines.push(Line::from(Span::styled("Prerequisites:", theme.highlight())));
            for prereq in &puzzle.prerequisites {
                lines.push(Line::from(Span::styled(
                    format!("  - {}", prereq),
                    theme.muted_style(),
                )));
            }
            lines.push(Line::from(""));
        }

        // Locked status
        if puzzle.is_locked {
            lines.push(Line::from(Span::styled(
                "🔒 This puzzle is locked. Complete prerequisites first.",
                Style::default().fg(theme.error),
            )));
        }

        let para = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(format!(" {} ", puzzle.id))
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(para, chunks[0]);
    }

    // Help
    let help = Paragraph::new(" [S] Start Puzzle  [Esc] Back ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[1]);
}
