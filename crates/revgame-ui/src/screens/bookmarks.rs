use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use revgame_core::debugger::Bookmark;

use crate::theme::Theme;

/// Bookmarks viewer state
pub struct BookmarksViewState {
    pub selected: usize,
    pub editing: Option<EditingBookmark>,
}

/// State when editing a bookmark
#[derive(Debug, Clone)]
pub struct EditingBookmark {
    pub address: u32,
    pub note: String,
}

impl Default for BookmarksViewState {
    fn default() -> Self {
        Self {
            selected: 0,
            editing: None,
        }
    }
}

impl BookmarksViewState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn navigate_up(&mut self, max: usize) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn navigate_down(&mut self, max: usize) {
        if self.selected < max.saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn start_editing(&mut self, address: u32, current_note: String) {
        self.editing = Some(EditingBookmark {
            address,
            note: current_note,
        });
    }

    pub fn cancel_editing(&mut self) {
        self.editing = None;
    }

    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }
}

/// Render the bookmarks dialog
pub fn render_bookmarks_dialog(
    frame: &mut Frame,
    bookmarks: &[&Bookmark],
    state: &BookmarksViewState,
    theme: &Theme,
) {
    let area = centered_rect(80, 80, frame.area());

    // If editing, show edit dialog
    if let Some(ref editing) = state.editing {
        render_edit_dialog(frame, editing, theme);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(5),     // Bookmark list
            Constraint::Length(2),  // Help
        ])
        .split(area);

    // Background
    let block = Block::default()
        .title(" Bookmarks ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    frame.render_widget(block, area);

    // Header
    let header = Paragraph::new(format!("{} Bookmarks", bookmarks.len()))
        .style(theme.normal())
        .alignment(Alignment::Center);

    frame.render_widget(header, chunks[0]);

    // Bookmark list
    if !bookmarks.is_empty() {
        let items: Vec<ListItem> = bookmarks
            .iter()
            .enumerate()
            .map(|(idx, bookmark)| {
                let note = if bookmark.note.is_empty() {
                    "<no note>".to_string()
                } else {
                    bookmark.note.clone()
                };

                let text = format!("  0x{:08X}: {}", bookmark.address, note);

                let style = if idx == state.selected {
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD)
                } else {
                    theme.normal()
                };

                ListItem::new(Line::from(Span::styled(text, style)))
            })
            .collect();

        let list = List::new(items).block(Block::default().borders(Borders::ALL));

        frame.render_widget(list, chunks[1]);
    } else {
        let no_bookmarks = Paragraph::new("No bookmarks yet\n\nPress [B] on any address to add a bookmark")
            .style(theme.muted_style())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(no_bookmarks, chunks[1]);
    }

    // Help
    let help = if bookmarks.is_empty() {
        " [Esc] Close "
    } else {
        " [↑↓] Navigate  [G] Go to  [E] Edit  [D] Delete  [Esc] Close "
    };

    let help_para = Paragraph::new(help)
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help_para, chunks[2]);
}

/// Render the edit bookmark dialog
fn render_edit_dialog(frame: &mut Frame, editing: &EditingBookmark, theme: &Theme) {
    let area = centered_rect(60, 30, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Address
            Constraint::Length(3),  // Note input
            Constraint::Length(2),  // Help
        ])
        .split(area);

    // Background
    let block = Block::default()
        .title(" Edit Bookmark ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    frame.render_widget(block, area);

    // Address
    let address_text = format!("Address: 0x{:08X}", editing.address);
    let address_para = Paragraph::new(address_text)
        .style(theme.normal())
        .alignment(Alignment::Left);

    frame.render_widget(address_para, chunks[0]);

    // Note input
    let note_text = format!("Note: {}", editing.note);
    let note_para = Paragraph::new(note_text)
        .style(Style::default().fg(theme.accent))
        .alignment(Alignment::Left);

    frame.render_widget(note_para, chunks[1]);

    // Help
    let help = Paragraph::new(" [Enter] Save  [Esc] Cancel  [Type to edit note] ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[2]);
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
