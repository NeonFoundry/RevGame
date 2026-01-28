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
    screens::{render_debugger, render_achievements},
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
        Screen::PuzzleComplete { ref message } => render_puzzle_complete(frame, app, message),
        _ => render_main_menu(frame, app), // Fallback
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
    match app.screen {
        Screen::MainMenu => handle_main_menu_key(app, code),
        Screen::Debugger => handle_debugger_key(app, code, modifiers),
        Screen::Achievements => handle_achievements_key(app, code),
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
        KeyCode::Char('s') => {
            app.step();
            app.trigger_tutorial(TutorialTrigger::Step);
        }
        KeyCode::Char('r') => {
            app.run();
            app.trigger_tutorial(TutorialTrigger::Run);
        }
        KeyCode::F(9) => {
            app.toggle_breakpoint();
            app.trigger_tutorial(TutorialTrigger::SetBreakpoint);
        }
        KeyCode::Char('b') => {
            app.toggle_breakpoint();
            app.trigger_tutorial(TutorialTrigger::SetBreakpoint);
        }

        // Other
        KeyCode::F(1) | KeyCode::Char('?') => {
            app.message = Some(revgame_ui::app::Message {
                text: "F5=Run F10=Step F9=Breakpoint Tab=Focus h=Hint :=Command".to_string(),
                is_error: false,
            });
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
