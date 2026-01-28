use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::{App, FocusedPanel},
    widgets::{DisasmView, MemoryView, RegisterView, StackView, TutorialOverlay, DebuggerLayout, RewindOverlay},
};

/// Render the debugger screen
pub fn render_debugger(frame: &mut Frame, app: &App) {
    let area = frame.area();

    // Main layout: top section (debugger panels) + bottom (command/status)
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),    // Main content
            Constraint::Length(3), // Command input
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    // Left/right split for main content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Left: Disassembly + Memory
            Constraint::Percentage(40), // Right: Registers + Stack
        ])
        .split(main_chunks[0]);

    // Left column: Disassembly on top, Memory below
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(60), // Disassembly
            Constraint::Percentage(40), // Memory
        ])
        .split(content_chunks[0]);

    // Right column: Registers on top, Stack below
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Registers
            Constraint::Percentage(50), // Stack
        ])
        .split(content_chunks[1]);

    // Build layout info for tutorial overlay
    let layout = DebuggerLayout {
        disassembly: left_chunks[0],
        registers: right_chunks[0],
        memory: left_chunks[1],
        stack: right_chunks[1],
        command: main_chunks[1],
        status: main_chunks[2],
    };

    // Render disassembly
    if let Some(ref debugger) = app.debugger {
        let disasm_view = DisasmView::new(
            &app.disasm_cache,
            debugger.cpu.eip,
            &debugger.breakpoints,
            &app.theme,
        )
        .selected(app.disasm_selection)
        .focused(app.focused == FocusedPanel::Disassembly);

        frame.render_widget(disasm_view, left_chunks[0]);

        // Render registers
        let reg_view = RegisterView::new(&debugger.cpu, &app.changed_registers, &app.theme)
            .focused(app.focused == FocusedPanel::Registers);

        frame.render_widget(reg_view, right_chunks[0]);

        // Render memory
        let mem_data = app.get_memory(app.memory_view_addr, 256);
        let mem_view = MemoryView::new(mem_data, app.memory_view_addr, &app.theme)
            .focused(app.focused == FocusedPanel::Memory)
            .bytes_per_row(8);

        frame.render_widget(mem_view, left_chunks[1]);

        // Render stack
        let stack_entries = app.get_stack(10);
        let stack_view = StackView::new(stack_entries, debugger.cpu.regs.esp, &app.theme)
            .focused(app.focused == FocusedPanel::Stack);

        frame.render_widget(stack_view, right_chunks[1]);
    }

    // Command input
    let command_style = if app.focused == FocusedPanel::Command {
        app.theme.border_focused()
    } else {
        app.theme.border_style()
    };

    let command_block = Block::default()
        .title(" Command ")
        .borders(Borders::ALL)
        .border_style(command_style);

    let command_text = format!("> {}", app.command_input);
    let command_para = Paragraph::new(command_text)
        .block(command_block)
        .style(app.theme.normal());

    frame.render_widget(command_para, main_chunks[1]);

    // Status bar
    let status_content = if let Some(ref msg) = app.message {
        let style = if msg.is_error {
            app.theme.error_style()
        } else {
            app.theme.success_style()
        };
        Span::styled(&msg.text, style)
    } else {
        Span::styled(
            " [F5] Run  [F10] Step  [F9] BP  [u] Undo  [Ctrl+Y] Redo  [Tab] Focus  [:] Cmd  [Esc] Menu ",
            app.theme.muted_style(),
        )
    };

    let status_line = Line::from(status_content);
    let status_para = Paragraph::new(status_line);

    frame.render_widget(status_para, main_chunks[2]);

    // Render tutorial overlay if active
    if let Some(ref tutorial) = app.tutorial {
        if tutorial.active {
            if let Some(step) = tutorial.current() {
                let overlay = TutorialOverlay::new(
                    step,
                    &layout,
                    &app.theme,
                    tutorial.progress(),
                );
                frame.render_widget(overlay, area);
            }
        }
    }

    // Render rewind effect if active
    if app.rewind_effect.is_active() {
        let rewind_overlay = RewindOverlay::new(&app.rewind_effect);
        frame.render_widget(rewind_overlay, area);
    }
}
