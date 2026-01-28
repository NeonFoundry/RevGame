use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::{
    app::App,
    theme::Theme,
};

use revgame_core::game::AchievementId;

/// Render the achievements screen
pub fn render_achievements(frame: &mut Frame, app: &App, theme: &Theme) {
    let area = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Header
            Constraint::Min(10),    // Achievement list
            Constraint::Length(3),  // Stats
            Constraint::Length(1),  // Help bar
        ])
        .split(area);

    // Header
    let total_points = app.game_state.achievements.total_points;
    let unlocked_count = app.game_state.achievements.unlocked.len();
    let total_achievements = 15; // Total number of achievements
    let progress = app.game_state.achievements.progress_percentage();

    let header_text = vec![
        Line::from(Span::styled(
            "🏆 ACHIEVEMENTS 🏆",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("Progress: ", theme.muted_style()),
            Span::styled(
                format!("{}/{} ({:.0}%)", unlocked_count, total_achievements, progress),
                theme.highlight(),
            ),
            Span::raw("  "),
            Span::styled("Total Points: ", theme.muted_style()),
            Span::styled(format!("{}", total_points), Style::default().fg(theme.success)),
        ]),
    ];

    let header = Paragraph::new(header_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM));

    frame.render_widget(header, chunks[0]);

    // Achievement list
    let all_achievements = vec![
        AchievementId::FirstPatch,
        AchievementId::TutorialComplete,
        AchievementId::NoHintsUsed,
        AchievementId::SpeedRunner,
        AchievementId::Minimalist,
        AchievementId::OneShot,
        AchievementId::BasicMaster,
        AchievementId::FlowMaster,
        AchievementId::CrackmeMaster,
        AchievementId::UndoMaster,
        AchievementId::Perfectionist,
        AchievementId::Experimenter,
        AchievementId::WinStreak3,
        AchievementId::WinStreak5,
        AchievementId::WinStreak10,
    ];

    let items: Vec<ListItem> = all_achievements
        .iter()
        .map(|&ach| {
            let unlocked = app.game_state.achievements.is_unlocked(ach);

            let icon = if unlocked { ach.icon() } else { "🔒" };
            let name = if unlocked { ach.name() } else { "???" };
            let desc = if unlocked { ach.description() } else { "Locked" };
            let points = ach.points();

            let style = if unlocked {
                theme.highlight()
            } else {
                theme.muted_style()
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", icon), style),
                Span::styled(format!("{:<25}", name), style.add_modifier(Modifier::BOLD)),
                Span::styled(format!(" {} ", desc), style),
                Span::styled(
                    format!("({}pts)", points),
                    if unlocked {
                        Style::default().fg(theme.success)
                    } else {
                        theme.muted_style()
                    },
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let achievement_list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Achievements "));

    frame.render_widget(achievement_list, chunks[1]);

    // Stats section
    let stats = vec![
        Line::from(vec![
            Span::styled("Puzzles Completed: ", theme.muted_style()),
            Span::styled(
                format!("{}", app.game_state.achievements.total_completed),
                theme.highlight(),
            ),
            Span::raw("  "),
            Span::styled("Win Streak: ", theme.muted_style()),
            Span::styled(
                format!("{}", app.game_state.achievements.current_streak),
                Style::default().fg(theme.warning),
            ),
            Span::raw("  "),
            Span::styled("Best Streak: ", theme.muted_style()),
            Span::styled(
                format!("{}", app.game_state.achievements.best_streak),
                Style::default().fg(theme.accent),
            ),
        ]),
        Line::from(vec![
            Span::styled("Total Patches: ", theme.muted_style()),
            Span::styled(
                format!("{}", app.game_state.achievements.total_patches),
                theme.highlight(),
            ),
            Span::raw("  "),
            Span::styled("Total Undos: ", theme.muted_style()),
            Span::styled(
                format!("{}", app.game_state.achievements.total_undos),
                theme.highlight(),
            ),
        ]),
    ];

    let stats_para = Paragraph::new(stats)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));

    frame.render_widget(stats_para, chunks[2]);

    // Help bar
    let help = Paragraph::new(" [Esc] Back ")
        .style(theme.muted_style())
        .alignment(Alignment::Center);

    frame.render_widget(help, chunks[3]);
}
