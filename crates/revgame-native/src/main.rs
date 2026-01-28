use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use revgame_ui::{
    app::{App, FocusedPanel, Screen},
    screens::{
        render_debugger, render_achievements, render_reference, render_search_dialog,
        render_bookmarks_dialog, SearchMode,
    },
    TutorialTrigger,
};

/// Sample puzzle embedded for testing
const SAMPLE_PUZZLE: &str = r#"
[metadata]
id = "basic-001"
title = "Your First Patch"
difficulty = 1
category = "patching"
tags = ["nop", "jump", "beginner"]

[description]
brief = "Learn to use NOP to skip unwanted instructions"
detailed = """
The program below checks if EAX equals 0x1337.
Unfortunately, EAX is set to 0xDEAD.

Your goal: Patch the code so the program sets EAX to 1 (success).

Hint: What if the comparison never happened?
"""

[setup]
memory_size = 16384
code_start = 0x1000
data_start = 0x2000
stack_start = 0x3000

[setup.registers]
eax = 0xDEAD
esp = 0x3000

[setup.code]
# Assembly:
#   cmp eax, 0x1337      ; 3D 37 13 00 00
#   jne fail             ; 75 07
#   mov eax, 1           ; B8 01 00 00 00
#   jmp end              ; EB 05
# fail:
#   mov eax, 0           ; B8 00 00 00 00
# end:
#   hlt                  ; F4
bytes = "3D 37 13 00 00 75 07 B8 01 00 00 00 EB 05 B8 00 00 00 00 F4"
entry_point = 0

[validation]
type = "register_value"
register = "eax"
expected = 1

[hints]
level1 = "Look at the JNE instruction at 0x1005. What does it check?"
level2 = "The NOP instruction does nothing and is 1 byte (0x90)"
level3 = "Try replacing the JNE (75 07) with two NOPs (90 90) to skip the branch"
"#;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new();

    // Run the app
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if app.should_quit {
            return Ok(());
        }

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers);
            }
        }

        // Tick animations
        app.rewind_effect.tick();
    }
}

fn ui(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::MainMenu => render_main_menu(frame, app),
        Screen::Debugger => render_debugger(frame, app),
        Screen::Achievements => render_achievements(frame, app, &app.theme),
        Screen::Reference => render_reference(frame, app, &app.reference_state, &app.theme),
        Screen::PuzzleComplete { ref message } => render_puzzle_complete(frame, app, message),
        _ => render_main_menu(frame, app), // Fallback
    }

    // Render search dialog overlay if open
    if app.search_dialog_open {
        render_search_dialog(frame, &app.search_state, &app.theme);
    }

    // Render bookmarks dialog overlay if open
    if app.bookmarks_dialog_open {
        if let Some(ref dbg) = app.debugger {
            let bookmarks = dbg.bookmarks.list();
            render_bookmarks_dialog(frame, &bookmarks, &app.bookmarks_view_state, &app.theme);
        }
    }
}

fn render_main_menu(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),  // Title
            Constraint::Min(10),    // Menu
            Constraint::Length(3), // Footer
        ])
        .split(area);

    // Title
    let title = r#"
  ██████╗ ███████╗██╗   ██╗ ██████╗  █████╗ ███╗   ███╗███████╗
  ██╔══██╗██╔════╝██║   ██║██╔════╝ ██╔══██╗████╗ ████║██╔════╝
  ██████╔╝█████╗  ██║   ██║██║  ███╗███████║██╔████╔██║█████╗
  ██╔══██╗██╔══╝  ╚██╗ ██╔╝██║   ██║██╔══██║██║╚██╔╝██║██╔══╝
  ██║  ██║███████╗ ╚████╔╝ ╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗
  ╚═╝  ╚═╝╚══════╝  ╚═══╝   ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝
"#;

    let title_para = Paragraph::new(title)
        .style(Style::default().fg(app.theme.accent).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(title_para, chunks[0]);

    // Menu items
    let menu_items = vec![
        ListItem::new("  [1] Start Tutorial (with walkthrough)"),
        ListItem::new("  [2] Quick Start (skip tutorial)"),
        ListItem::new("  [3] Puzzle Select (coming soon)"),
        ListItem::new("  [A] Achievements"),
        ListItem::new("  [R] x86 Reference Manual"),
        ListItem::new("  [Q] Quit"),
    ];

    let menu = List::new(menu_items)
        .block(
            Block::default()
                .title(" Menu ")
                .borders(Borders::ALL)
                .border_style(app.theme.border_style()),
        )
        .style(app.theme.normal())
        .highlight_style(app.theme.selected());

    frame.render_widget(menu, chunks[1]);

    // Footer
    let footer = Paragraph::new("Learn reverse engineering through interactive puzzles")
        .style(app.theme.muted_style())
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(footer, chunks[2]);
}

fn render_puzzle_complete(frame: &mut Frame, app: &App, message: &str) {
    let area = frame.area();

    let block = Block::default()
        .title(" Puzzle Complete! ")
        .borders(Borders::ALL)
        .border_style(app.theme.success_style());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(message, app.theme.success_style())),
        Line::from(""),
        Line::from(Span::styled(
            format!("Hints used: {}", app.game_state.hints_used),
            app.theme.normal(),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Press [Enter] to continue or [Esc] for menu",
            app.theme.muted_style(),
        )),
    ];

    let para = Paragraph::new(text).alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(para, inner);
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // If search dialog is open, handle search keys first
    if app.search_dialog_open {
        handle_search_key(app, code);
        return;
    }

    // If bookmarks dialog is open, handle bookmarks keys first
    if app.bookmarks_dialog_open {
        handle_bookmarks_key(app, code);
        return;
    }

    match app.screen {
        Screen::MainMenu => handle_main_menu_key(app, code),
        Screen::Debugger => handle_debugger_key(app, code, modifiers),
        Screen::Achievements => handle_achievements_key(app, code),
        Screen::Reference => handle_reference_key(app, code),
        Screen::PuzzleComplete { .. } => handle_complete_key(app, code),
        _ => {}
    }
}

fn handle_main_menu_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('1') => {
            // Start tutorial with walkthrough
            if let Err(e) = app.start_tutorial(SAMPLE_PUZZLE) {
                app.message = Some(revgame_ui::app::Message {
                    text: format!("Failed to load tutorial: {}", e),
                    is_error: true,
                });
            }
        }
        KeyCode::Char('2') => {
            // Quick start without tutorial
            if let Err(e) = app.load_puzzle(SAMPLE_PUZZLE) {
                app.message = Some(revgame_ui::app::Message {
                    text: format!("Failed to load puzzle: {}", e),
                    is_error: true,
                });
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.screen = Screen::Achievements;
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.screen = Screen::Reference;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            app.should_quit = true;
        }
        KeyCode::Esc => {
            app.should_quit = true;
        }
        _ => {}
    }
}

fn handle_achievements_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.screen = Screen::MainMenu;
        }
        _ => {}
    }
}

fn handle_reference_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.reference_state.navigate_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.reference_state.navigate_down();
        }
        KeyCode::Enter => {
            app.reference_state.enter();
        }
        KeyCode::Esc | KeyCode::Backspace => {
            app.reference_state.back();
            // If we're back at category list and press Esc, go to menu
            if app.reference_state.view_mode == revgame_ui::screens::ReferenceViewMode::CategoryList {
                app.screen = Screen::MainMenu;
            }
        }
        _ => {}
    }
}

fn handle_debugger_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Handle Ctrl+C to quit
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        app.should_quit = true;
        return;
    }

    // If tutorial is active, handle tutorial-specific input first
    // BUT: If Command panel is focused, let all keys through for typing
    if app.is_tutorial_active() && app.focused != FocusedPanel::Command {
        match code {
            // ESC skips the tutorial
            KeyCode::Esc => {
                app.skip_tutorial();
                return;
            }
            // Enter/Space advances tutorial (for Continue triggers)
            KeyCode::Enter | KeyCode::Char(' ') => {
                app.trigger_tutorial(TutorialTrigger::Continue);
                return;
            }
            // Let other keys fall through to normal handling
            _ => {}
        }
    }

    match app.focused {
        FocusedPanel::Command => handle_command_input(app, code),
        _ => handle_panel_key(app, code, modifiers),
    }
}

fn handle_panel_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        // Navigation
        KeyCode::Tab => {
            app.focused = app.focused.next();
        }
        KeyCode::BackTab => {
            app.focused = app.focused.prev();
        }

        // Disassembly navigation
        KeyCode::Up | KeyCode::Char('k') => {
            if app.disasm_selection > 0 {
                app.disasm_selection -= 1;
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.disasm_selection < app.disasm_cache.len().saturating_sub(1) {
                app.disasm_selection += 1;
            }
        }

        // Execution controls
        KeyCode::F(5) => {
            app.run();
            app.trigger_tutorial(TutorialTrigger::Run);
            // Check if program halted
            if let Some(ref dbg) = app.debugger {
                if dbg.cpu.halted {
                    app.trigger_tutorial(TutorialTrigger::ProgramHalts);
                }
            }
        }
        KeyCode::F(10) => {
            app.step();
            app.trigger_tutorial(TutorialTrigger::Step);
        }
        KeyCode::Char('s') if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.step();
            app.trigger_tutorial(TutorialTrigger::Step);
        }
        KeyCode::Char('r') if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.run();
            app.trigger_tutorial(TutorialTrigger::Run);
        }
        KeyCode::F(9) => {
            app.toggle_breakpoint();
            app.trigger_tutorial(TutorialTrigger::SetBreakpoint);
        }
        KeyCode::Char('b') if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_breakpoint();
            app.trigger_tutorial(TutorialTrigger::SetBreakpoint);
        }

        // x86 Reference Manual
        KeyCode::F(1) | KeyCode::Char('?') => {
            app.screen = Screen::Reference;
        }
        KeyCode::Char('h') => {
            app.show_hint();
            app.trigger_tutorial(TutorialTrigger::Hint);
        }
        KeyCode::Char(':') | KeyCode::Char('/') => {
            app.focused = FocusedPanel::Command;
            app.command_input.clear();
            app.trigger_tutorial(TutorialTrigger::EnterCommand);
        }

        // Reset
        KeyCode::F(4) => {
            app.reset();
            app.trigger_tutorial(TutorialTrigger::Reset);
        }

        // Undo (Ctrl+Z or 'u')
        KeyCode::Char('z') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.undo_patch() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            }
        }
        KeyCode::Char('u') => {
            if let Err(e) = app.undo_patch() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            }
        }

        // Redo (Ctrl+Y or Ctrl+Shift+Z)
        KeyCode::Char('y') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.redo_patch() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            }
        }
        KeyCode::Char('Z') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.redo_patch() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            }
        }

        // Quick Save (F6 or Ctrl+S)
        KeyCode::F(6) => {
            if let Err(e) = app.quick_save() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            } else {
                app.message = Some(revgame_ui::app::Message {
                    text: "Game saved to quicksave slot".to_string(),
                    is_error: false,
                });
            }
        }
        KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.quick_save() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            } else {
                app.message = Some(revgame_ui::app::Message {
                    text: "Game saved to quicksave slot".to_string(),
                    is_error: false,
                });
            }
        }

        // Quick Load (F7 or Ctrl+L)
        KeyCode::F(7) => {
            if let Err(e) = app.quick_load() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            } else {
                app.message = Some(revgame_ui::app::Message {
                    text: "Game loaded from quicksave slot".to_string(),
                    is_error: false,
                });
            }
        }
        KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.quick_load() {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            } else {
                app.message = Some(revgame_ui::app::Message {
                    text: "Game loaded from quicksave slot".to_string(),
                    is_error: false,
                });
            }
        }

        // Search (Ctrl+F)
        KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.search_dialog_open = true;
            app.search_state.clear_results();
            app.search_state.input.clear();
        }

        // Bookmarks
        KeyCode::Char('b') if modifiers.contains(KeyModifiers::CONTROL) => {
            app.toggle_bookmark_at_cursor();
        }
        KeyCode::Char('m') | KeyCode::Char('M') => {
            app.bookmarks_dialog_open = true;
            app.bookmarks_view_state.selected = 0;
        }
        KeyCode::Char('n') if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.goto_next_bookmark();
        }
        KeyCode::Char('p') if !modifiers.contains(KeyModifiers::CONTROL) => {
            app.goto_prev_bookmark();
        }

        // Menu (only if not in tutorial)
        KeyCode::Esc => {
            if !app.is_tutorial_active() {
                app.screen = Screen::MainMenu;
                app.debugger = None;
                app.puzzle = None;
                app.tutorial = None;
            }
        }

        _ => {}
    }
}

fn handle_command_input(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            let cmd = app.command_input.clone();
            app.command_input.clear();
            app.focused = FocusedPanel::Disassembly;

            // Check if it's a patch command for tutorial
            if cmd.to_lowercase().starts_with("patch") {
                app.process_command(&cmd);
                app.trigger_tutorial(TutorialTrigger::Patch);
            } else {
                app.process_command(&cmd);
            }
        }
        KeyCode::Esc => {
            app.command_input.clear();
            app.focused = FocusedPanel::Disassembly;
        }
        KeyCode::Backspace => {
            app.command_input.pop();
        }
        KeyCode::Char(c) => {
            app.command_input.push(c);
        }
        _ => {}
    }
}

fn handle_search_key(app: &mut App, code: KeyCode) {
    match code {
        // Mode selection
        KeyCode::Char('1') => {
            app.search_state.mode = SearchMode::Bytes;
            app.search_state.clear_results();
        }
        KeyCode::Char('2') => {
            app.search_state.mode = SearchMode::String;
            app.search_state.clear_results();
        }
        KeyCode::Char('3') => {
            app.search_state.mode = SearchMode::FindStrings;
            app.search_state.clear_results();
        }

        // Toggle case sensitivity (String mode only)
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if app.search_state.mode == SearchMode::String {
                app.search_state.case_sensitive = !app.search_state.case_sensitive;
            }
        }

        // Adjust min string length (FindStrings mode only)
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.search_state.mode == SearchMode::FindStrings {
                app.search_state.min_string_length = app.search_state.min_string_length.saturating_add(1);
            }
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if app.search_state.mode == SearchMode::FindStrings {
                app.search_state.min_string_length = app.search_state.min_string_length.saturating_sub(1).max(1);
            }
        }

        // Navigation
        KeyCode::Up | KeyCode::Char('k') => {
            app.search_state.navigate_up();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.search_state.navigate_down();
        }

        // Execute search
        KeyCode::Enter => {
            let result = match app.search_state.mode {
                SearchMode::Bytes => app.search_bytes(),
                SearchMode::String => app.search_string(),
                SearchMode::FindStrings => app.find_strings(),
            };

            if let Err(e) = result {
                app.message = Some(revgame_ui::app::Message {
                    text: e,
                    is_error: true,
                });
            }
        }

        // Go to selected result
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.goto_search_result();
        }

        // Text input (for Bytes and String modes)
        KeyCode::Char(c) if matches!(app.search_state.mode, SearchMode::Bytes | SearchMode::String) => {
            app.search_state.input.push(c);
        }

        // Backspace
        KeyCode::Backspace => {
            app.search_state.input.pop();
        }

        // Close dialog
        KeyCode::Esc => {
            app.search_dialog_open = false;
            app.search_state.clear_results();
            app.search_state.input.clear();
        }

        _ => {}
    }
}

fn handle_bookmarks_key(app: &mut App, code: KeyCode) {
    // If editing a bookmark, handle edit keys
    if app.bookmarks_view_state.is_editing() {
        match code {
            KeyCode::Enter => {
                app.save_edited_bookmark();
            }
            KeyCode::Esc => {
                app.bookmarks_view_state.cancel_editing();
            }
            KeyCode::Backspace => {
                if let Some(ref mut editing) = app.bookmarks_view_state.editing {
                    editing.note.pop();
                }
            }
            KeyCode::Char(c) => {
                if let Some(ref mut editing) = app.bookmarks_view_state.editing {
                    editing.note.push(c);
                }
            }
            _ => {}
        }
        return;
    }

    // Normal bookmark list navigation
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(ref dbg) = app.debugger {
                let max = dbg.bookmarks.count();
                app.bookmarks_view_state.navigate_up(max);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref dbg) = app.debugger {
                let max = dbg.bookmarks.count();
                app.bookmarks_view_state.navigate_down(max);
            }
        }
        KeyCode::Char('g') | KeyCode::Char('G') => {
            app.goto_selected_bookmark();
        }
        KeyCode::Char('e') | KeyCode::Char('E') => {
            app.start_editing_bookmark();
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.delete_selected_bookmark();
        }
        KeyCode::Esc => {
            app.bookmarks_dialog_open = false;
            app.bookmarks_view_state.selected = 0;
        }
        _ => {}
    }
}

fn handle_complete_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => {
            app.trigger_tutorial(TutorialTrigger::PuzzleSolved);
            app.screen = Screen::MainMenu;
            app.debugger = None;
            app.puzzle = None;
            app.tutorial = None;
        }
        KeyCode::Esc => {
            app.screen = Screen::MainMenu;
            app.debugger = None;
            app.puzzle = None;
            app.tutorial = None;
        }
        _ => {}
    }
}
