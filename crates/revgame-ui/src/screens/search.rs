use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use revgame_core::debugger::SearchResult;

use crate::theme::Theme;

/// Search mode (what type of search to perform)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchMode {
    Bytes,
    String,
    FindStrings,
}

/// Search dialog state
pub struct SearchState {
    pub mode: SearchMode,
    pub input: String,
    pub results: Vec<SearchResult>,
    pub selected_result: usize,
    pub case_sensitive: bool,
    pub min_string_length: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self {
            mode: SearchMode::String,
            input: String::new(),
            results: Vec::new(),
            selected_result: 0,
            case_sensitive: false,
            min_string_length: 4,
        }
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
        self.selected_result = 0;
    }

    pub fn navigate_up(&mut self) {
        if self.selected_result > 0 {
            self.selected_result -= 1;
        }
    }

    pub fn navigate_down(&mut self) {
        if self.selected_result < self.results.len().saturating_sub(1) {
            self.selected_result += 1;
        }
    }

    pub fn get_selected_address(&self) -> Option<u32> {
        self.results.get(self.selected_result).map(|r| r.address)
    }
}

/// Render the search dialog
pub fn render_search_dialog(frame: &mut Frame, state: &SearchState, theme: &Theme) {
    let area = centered_rect(80, 80, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Mode selection
            Constraint::Length(3),  // Input
            Constraint::Length(2),  // Options
            Constraint::Min(5),     // Results
            Constraint::Length(2),  // Help
        ])
        .split(area);

    // Background
    let block = Block::default()
        .title(" Search Memory ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    frame.render_widget(block, area);

    // Mode selection
    let mode_text = match state.mode {
        SearchMode::Bytes => "[1] Bytes  [2] String  [3] Find Strings",
        SearchMode::String => " [1] Bytes [2] String  [3] Find Strings",
        SearchMode::FindStrings => " [1] Bytes  [2] String [3] Find Strings",
    };

    let mode_para = Paragraph::new(mode_text)
        .style(theme.normal())
        .alignment(Alignment::Center);

    frame.render_widget(mode_para, chunks[0]);

    // Input
    let input_text = match state.mode {
        SearchMode::Bytes => format!("Hex Pattern: {}", state.input),
        SearchMode::String => format!("Search String: {}", state.input),
        SearchMode::FindStrings => format!("Min Length: {}", state.min_string_length),
    };

    let input_para = Paragraph::new(input_text)
        .style(Style::default().fg(theme.accent))
        .alignment(Alignment::Left);

    frame.render_widget(input_para, chunks[1]);

    // Options
    let options_text = if state.mode == SearchMode::String {
        format!(
            "Case Sensitive: {} [Toggle: C]",
            if state.case_sensitive { "Yes" } else { "No" }
        )
    } else {
        String::new()
    };

    let options_para = Paragraph::new(options_text)
        .style(theme.muted_style())
        .alignment(Alignment::Left);

    frame.render_widget(options_para, chunks[2]);

    // Results
    if !state.results.is_empty() {
        let items: Vec<ListItem> = state
            .results
            .iter()
            .enumerate()
            .map(|(idx, result)| {
                let data_str = if result.data.iter().all(|&b| b.is_ascii_graphic() || b.is_ascii_whitespace()) {
                    // Display as string if all ASCII
                    String::from_utf8_lossy(&result.data).to_string()
                } else {
                    // Display as hex
                    result
                        .data
                        .iter()
                        .map(|b| format!("{:02X}", b))
                        .collect::<Vec<_>>()
                        .join(" ")
                };

                let text = format!("  0x{:08X}: {}", result.address, data_str);

                let style = if idx == state.selected_result {
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
                .title(format!(" {} Results ", state.results.len()))
                .borders(Borders::ALL),
        );

        frame.render_widget(list, chunks[3]);
    } else {
        let no_results = Paragraph::new("No results")
            .style(theme.muted_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(no_results, chunks[3]);
    }

    // Help
    let help_text = match state.mode {
        SearchMode::Bytes => " [1-3] Mode  [Enter] Search  [↑↓] Navigate  [G] Go to  [Esc] Close ",
        SearchMode::String => " [1-3] Mode  [C] Case  [Enter] Search  [↑↓] Navigate  [G] Go to  [Esc] Close ",
        SearchMode::FindStrings => " [1-3] Mode  [+/-] Length  [Enter] Search  [↑↓] Navigate  [G] Go to  [Esc] Close ",
    };

    let help = Paragraph::new(help_text)
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[4]);
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
