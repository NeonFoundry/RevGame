use std::collections::HashSet;

use revgame_core::{
    debugger::Debugger,
    emulator::DisassemblyLine,
    puzzle::{load_puzzle, Puzzle, ValidationResult, Validator},
    game::{GameState, SaveManager},
};

use crate::Theme;
use crate::tutorial::{Tutorial, TutorialTrigger};
use crate::widgets::RewindEffect;
use crate::screens::{ReferenceState, SearchState, BookmarksViewState, PuzzleSelectState};
use crate::syntax::SyntaxHighlighter;

/// Which panel is currently focused
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPanel {
    Disassembly,
    Registers,
    Memory,
    Stack,
    Command,
}

impl FocusedPanel {
    pub fn next(&self) -> Self {
        match self {
            FocusedPanel::Disassembly => FocusedPanel::Registers,
            FocusedPanel::Registers => FocusedPanel::Memory,
            FocusedPanel::Memory => FocusedPanel::Stack,
            FocusedPanel::Stack => FocusedPanel::Command,
            FocusedPanel::Command => FocusedPanel::Disassembly,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            FocusedPanel::Disassembly => FocusedPanel::Command,
            FocusedPanel::Registers => FocusedPanel::Disassembly,
            FocusedPanel::Memory => FocusedPanel::Registers,
            FocusedPanel::Stack => FocusedPanel::Memory,
            FocusedPanel::Command => FocusedPanel::Stack,
        }
    }
}

/// Current screen/view
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    PuzzleSelect,
    Debugger,
    Tutorial,
    Settings,
    Help,
    Achievements,
    Reference,
    PuzzleComplete { message: String },
}

/// Message to display to user
#[derive(Debug, Clone)]
pub struct Message {
    pub text: String,
    pub is_error: bool,
}

/// Main application state
pub struct App {
    /// Current theme
    pub theme: Theme,

    /// Current screen
    pub screen: Screen,

    /// Debugger instance
    pub debugger: Option<Debugger>,

    /// Currently loaded puzzle
    pub puzzle: Option<Puzzle>,

    /// Game progress state
    pub game_state: GameState,

    /// Currently focused panel
    pub focused: FocusedPanel,

    /// Selected line in disassembly
    pub disasm_selection: usize,

    /// Memory view start address
    pub memory_view_addr: u32,

    /// Command input buffer
    pub command_input: String,

    /// Recent changed registers (for highlighting)
    pub changed_registers: HashSet<String>,

    /// Message to display
    pub message: Option<Message>,

    /// Disassembly cache
    pub disasm_cache: Vec<DisassemblyLine>,

    /// Whether to quit
    pub should_quit: bool,

    /// Current hint level shown
    pub hint_level: usize,

    /// Tutorial state (if in tutorial mode)
    pub tutorial: Option<Tutorial>,

    /// VHS rewind effect
    pub rewind_effect: RewindEffect,

    /// Instruction reference state
    pub reference_state: ReferenceState,

    /// Search dialog state
    pub search_state: SearchState,

    /// Whether search dialog is open
    pub search_dialog_open: bool,

    /// Bookmarks view state
    pub bookmarks_view_state: BookmarksViewState,

    /// Whether bookmarks dialog is open
    pub bookmarks_dialog_open: bool,

    /// Syntax highlighter for disassembly
    pub syntax_highlighter: SyntaxHighlighter,

    /// Puzzle select state
    pub puzzle_select_state: PuzzleSelectState,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            theme: Theme::default(),
            screen: Screen::MainMenu,
            debugger: None,
            puzzle: None,
            game_state: GameState::new(),
            focused: FocusedPanel::Disassembly,
            disasm_selection: 0,
            memory_view_addr: 0x1000,
            command_input: String::new(),
            changed_registers: HashSet::new(),
            message: None,
            disasm_cache: Vec::new(),
            should_quit: false,
            hint_level: 0,
            tutorial: None,
            rewind_effect: RewindEffect::new(),
            reference_state: ReferenceState::new(),
            search_state: SearchState::new(),
            search_dialog_open: false,
            bookmarks_view_state: BookmarksViewState::new(),
            bookmarks_dialog_open: false,
            syntax_highlighter: SyntaxHighlighter::new(),
            puzzle_select_state: PuzzleSelectState::new(),
        }
    }

    /// Start the tutorial
    pub fn start_tutorial(&mut self, puzzle_toml: &str) -> Result<(), String> {
        self.load_puzzle(puzzle_toml)?;
        self.tutorial = Some(Tutorial::interface_tutorial());
        Ok(())
    }

    /// Skip the tutorial
    pub fn skip_tutorial(&mut self) {
        if let Some(ref mut tutorial) = self.tutorial {
            tutorial.skip();
        }
        self.tutorial = None;
    }

    /// Check and advance tutorial based on trigger
    pub fn trigger_tutorial(&mut self, trigger: TutorialTrigger) {
        if let Some(ref mut tutorial) = self.tutorial {
            tutorial.check_trigger(&trigger);

            // Clear tutorial if finished
            if tutorial.is_finished() {
                self.tutorial = None;
            }
        }
    }

    /// Check if tutorial is active
    pub fn is_tutorial_active(&self) -> bool {
        self.tutorial.as_ref().map(|t| t.active).unwrap_or(false)
    }

    /// Load a puzzle from TOML content
    pub fn load_puzzle(&mut self, toml: &str) -> Result<(), String> {
        let puzzle = load_puzzle(toml)?;
        self.setup_puzzle(puzzle)
    }

    /// Set up a puzzle for playing
    pub fn setup_puzzle(&mut self, puzzle: Puzzle) -> Result<(), String> {
        // Create debugger with puzzle layout
        let mut debugger = Debugger::with_puzzle_layout(
            puzzle.entry_point(),
            puzzle.setup.code_start,
            puzzle.setup.data_start,
            puzzle.setup.stack_start,
        );

        // Load code
        let code = puzzle.code_bytes()?;
        debugger.load_code(puzzle.setup.code_start, &code).map_err(|e| e.to_string())?;

        // Load data if present
        if let Some(data) = puzzle.data_bytes()? {
            debugger.load_data(puzzle.setup.data_start, &data).map_err(|e| e.to_string())?;
        }

        // Set up registers
        let regs = &puzzle.setup.registers;
        if let Some(v) = regs.eax { debugger.cpu.regs.eax = v; }
        if let Some(v) = regs.ebx { debugger.cpu.regs.ebx = v; }
        if let Some(v) = regs.ecx { debugger.cpu.regs.ecx = v; }
        if let Some(v) = regs.edx { debugger.cpu.regs.edx = v; }
        if let Some(v) = regs.esi { debugger.cpu.regs.esi = v; }
        if let Some(v) = regs.edi { debugger.cpu.regs.edi = v; }
        if let Some(v) = regs.ebp { debugger.cpu.regs.ebp = v; }
        if let Some(v) = regs.esp { debugger.cpu.regs.esp = v; }

        // Save initial state for reset
        debugger.save_initial_state();

        // Update app state
        self.game_state.start_puzzle(&puzzle.metadata.id);
        self.memory_view_addr = puzzle.setup.data_start;
        self.hint_level = 0;
        self.debugger = Some(debugger);
        self.puzzle = Some(puzzle);
        self.screen = Screen::Debugger;
        self.refresh_disasm();

        Ok(())
    }

    /// Refresh disassembly cache
    pub fn refresh_disasm(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            self.disasm_cache = debugger.disassemble(20);
        }
    }

    /// Step one instruction
    pub fn step(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            self.changed_registers.clear();

            match debugger.step() {
                Ok(result) => {
                    for reg in result.changed_registers {
                        self.changed_registers.insert(reg);
                    }
                    self.refresh_disasm();
                    self.check_completion();
                }
                Err(e) => {
                    self.message = Some(Message {
                        text: format!("Error: {}", e),
                        is_error: true,
                    });
                }
            }
        }
    }

    /// Run until breakpoint or halt
    pub fn run(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            self.changed_registers.clear();

            match debugger.run() {
                Ok(result) => {
                    let msg = match result {
                        revgame_core::debugger::RunResult::Breakpoint(addr) => {
                            format!("Breakpoint at 0x{:08X}", addr)
                        }
                        revgame_core::debugger::RunResult::Halted => {
                            "Program halted".to_string()
                        }
                        revgame_core::debugger::RunResult::LimitExceeded(n) => {
                            format!("Execution limit exceeded ({} instructions)", n)
                        }
                        revgame_core::debugger::RunResult::Error(e) => {
                            format!("Error: {}", e)
                        }
                    };
                    self.message = Some(Message { text: msg, is_error: false });
                    self.refresh_disasm();
                    self.check_completion();
                }
                Err(e) => {
                    self.message = Some(Message {
                        text: format!("Error: {}", e),
                        is_error: true,
                    });
                }
            }
        }
    }

    /// Reset puzzle to initial state
    pub fn reset(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            debugger.reset();
            self.refresh_disasm();
            self.changed_registers.clear();
            self.message = Some(Message {
                text: "Reset to initial state".to_string(),
                is_error: false,
            });
        }
    }

    /// Toggle breakpoint at selected address
    pub fn toggle_breakpoint(&mut self) {
        if let Some(ref mut debugger) = self.debugger {
            if let Some(line) = self.disasm_cache.get(self.disasm_selection) {
                let addr = line.address;
                let set = debugger.toggle_breakpoint(addr);
                self.message = Some(Message {
                    text: if set {
                        format!("Breakpoint set at 0x{:08X}", addr)
                    } else {
                        format!("Breakpoint removed at 0x{:08X}", addr)
                    },
                    is_error: false,
                });
            }
        }
    }

    /// Show next hint
    pub fn show_hint(&mut self) {
        if let Some(ref puzzle) = self.puzzle {
            self.hint_level += 1;
            if let Some(hint) = puzzle.hints.get_hint(self.hint_level) {
                self.game_state.use_hint();
                self.message = Some(Message {
                    text: format!("Hint {}: {}", self.hint_level, hint),
                    is_error: false,
                });
            } else {
                self.message = Some(Message {
                    text: "No more hints available".to_string(),
                    is_error: false,
                });
            }
        }
    }

    /// Check if puzzle is completed
    fn check_completion(&mut self) {
        if let (Some(ref debugger), Some(ref puzzle)) = (&self.debugger, &self.puzzle) {
            if debugger.cpu.halted {
                let result = Validator::validate(puzzle, &debugger.cpu, &debugger.memory);
                match result {
                    ValidationResult::Success => {
                        let achievements = self.game_state.complete_puzzle(
                            &puzzle.metadata.id,
                            puzzle.metadata.difficulty,
                        );

                        // Build completion message with achievements
                        let mut msg = format!("Congratulations! You solved '{}'!", puzzle.metadata.title);

                        if !achievements.is_empty() {
                            msg.push_str("\n\nAchievements Unlocked:");
                            for ach in achievements.iter() {
                                msg.push_str(&format!("\n{} {} (+{} pts)", ach.icon(), ach.name(), ach.points()));
                            }
                        }

                        self.screen = Screen::PuzzleComplete { message: msg };
                    }
                    ValidationResult::Failure(msg) => {
                        self.message = Some(Message {
                            text: format!("Not quite: {}", msg),
                            is_error: true,
                        });
                    }
                    ValidationResult::Error(e) => {
                        self.message = Some(Message {
                            text: format!("Validation error: {}", e),
                            is_error: true,
                        });
                    }
                }
            }
        }
    }

    /// Get current instruction address
    pub fn current_eip(&self) -> Option<u32> {
        self.debugger.as_ref().map(|d| d.cpu.eip)
    }

    /// Check if address has breakpoint
    pub fn has_breakpoint(&self, addr: u32) -> bool {
        self.debugger
            .as_ref()
            .map(|d| d.has_breakpoint(addr))
            .unwrap_or(false)
    }

    /// Get stack values for display
    pub fn get_stack(&self, count: usize) -> Vec<(u32, u32)> {
        let mut result = Vec::new();
        if let Some(ref debugger) = self.debugger {
            let esp = debugger.cpu.regs.esp;
            for i in 0..count {
                let addr = esp.wrapping_add((i as u32) * 4);
                if let Ok(value) = debugger.memory.read_u32(addr) {
                    result.push((addr, value));
                }
            }
        }
        result
    }

    /// Get memory for display
    pub fn get_memory(&self, addr: u32, count: usize) -> Option<&[u8]> {
        self.debugger.as_ref().and_then(|d| d.memory.slice(addr, count))
    }

    /// Patch memory at address
    pub fn patch_memory(&mut self, addr: u32, bytes: &[u8]) -> Result<(), String> {
        if let Some(ref mut debugger) = self.debugger {
            debugger.patch(addr, bytes).map_err(|e| e.to_string())?;
            self.game_state.record_patch();
            self.refresh_disasm();
            self.message = Some(Message {
                text: format!("Patched {} bytes at 0x{:08X}", bytes.len(), addr),
                is_error: false,
            });
            Ok(())
        } else {
            Err("No debugger active".to_string())
        }
    }

    /// Undo the last patch
    pub fn undo_patch(&mut self) -> Result<(), String> {
        if let Some(ref mut debugger) = self.debugger {
            debugger.undo_patch().map_err(|e| e.to_string())?;
            let remaining = debugger.undo_count();

            // Check for undo achievement
            if let Some(achievement) = self.game_state.record_undo() {
                self.message = Some(Message {
                    text: format!("Achievement Unlocked: {} {} (+{} pts)",
                        achievement.icon(), achievement.name(), achievement.points()),
                    is_error: false,
                });
            }

            self.rewind_effect.trigger();
            self.refresh_disasm();

            // Only override message if no achievement was unlocked
            if self.message.is_none() {
                self.message = Some(Message {
                    text: format!("Undone ({} remaining)", remaining),
                    is_error: false,
                });
            }
            Ok(())
        } else {
            Err("No debugger active".to_string())
        }
    }

    /// Redo the last undone patch
    pub fn redo_patch(&mut self) -> Result<(), String> {
        if let Some(ref mut debugger) = self.debugger {
            debugger.redo_patch().map_err(|e| e.to_string())?;
            let remaining = debugger.redo_count();
            self.refresh_disasm();
            self.message = Some(Message {
                text: format!("Redone ({} remaining)", remaining),
                is_error: false,
            });
            Ok(())
        } else {
            Err("No debugger active".to_string())
        }
    }

    /// Process a command
    pub fn process_command(&mut self, cmd: &str) {
        let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0].to_lowercase().as_str() {
            "help" | "h" | "?" => {
                self.message = Some(Message {
                    text: "Commands: step/s, run/r, reset, bp <addr>, patch <addr> <bytes>, undo/u, redo, hint, quit".to_string(),
                    is_error: false,
                });
            }
            "step" | "s" => self.step(),
            "run" | "r" => self.run(),
            "reset" => self.reset(),
            "undo" | "u" => {
                if let Err(e) = self.undo_patch() {
                    self.message = Some(Message {
                        text: e,
                        is_error: true,
                    });
                }
            }
            "redo" => {
                if let Err(e) = self.redo_patch() {
                    self.message = Some(Message {
                        text: e,
                        is_error: true,
                    });
                }
            }
            "bp" | "breakpoint" => {
                if parts.len() > 1 {
                    if let Ok(addr) = u32::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        if let Some(ref mut debugger) = self.debugger {
                            let set = debugger.toggle_breakpoint(addr);
                            self.message = Some(Message {
                                text: if set {
                                    format!("Breakpoint set at 0x{:08X}", addr)
                                } else {
                                    format!("Breakpoint removed at 0x{:08X}", addr)
                                },
                                is_error: false,
                            });
                        }
                    } else {
                        self.message = Some(Message {
                            text: "Invalid address".to_string(),
                            is_error: true,
                        });
                    }
                } else {
                    self.toggle_breakpoint();
                }
            }
            "patch" => {
                if parts.len() >= 3 {
                    if let Ok(addr) = u32::from_str_radix(parts[1].trim_start_matches("0x"), 16) {
                        let bytes: Result<Vec<u8>, _> = parts[2..]
                            .iter()
                            .map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16))
                            .collect();
                        match bytes {
                            Ok(b) => {
                                if let Err(e) = self.patch_memory(addr, &b) {
                                    self.message = Some(Message {
                                        text: e,
                                        is_error: true,
                                    });
                                }
                            }
                            Err(_) => {
                                self.message = Some(Message {
                                    text: "Invalid bytes".to_string(),
                                    is_error: true,
                                });
                            }
                        }
                    } else {
                        self.message = Some(Message {
                            text: "Invalid address".to_string(),
                            is_error: true,
                        });
                    }
                } else {
                    self.message = Some(Message {
                        text: "Usage: patch <addr> <bytes...>".to_string(),
                        is_error: true,
                    });
                }
            }
            "hint" => self.show_hint(),
            "quit" | "q" => self.should_quit = true,
            _ => {
                self.message = Some(Message {
                    text: format!("Unknown command: {}", parts[0]),
                    is_error: true,
                });
            }
        }
    }

    /// Save game progress
    pub fn save_game(&mut self, slot: &str) -> Result<(), String> {
        let save_manager = SaveManager::new()?;
        save_manager.save(&self.game_state, slot)?;

        self.message = Some(Message {
            text: format!("Game saved to slot: {}", slot),
            is_error: false,
        });

        Ok(())
    }

    /// Load game progress
    pub fn load_game(&mut self, slot: &str) -> Result<(), String> {
        let save_manager = SaveManager::new()?;
        let game_state = save_manager.load(slot)?;

        self.game_state = game_state;

        self.message = Some(Message {
            text: format!("Game loaded from slot: {}", slot),
            is_error: false,
        });

        Ok(())
    }

    /// Quick save (slot "quick")
    pub fn quick_save(&mut self) -> Result<(), String> {
        self.save_game("quick")
    }

    /// Quick load (slot "quick")
    pub fn quick_load(&mut self) -> Result<(), String> {
        self.load_game("quick")
    }

    /// Perform byte pattern search
    pub fn search_bytes(&mut self) -> Result<(), String> {
        use revgame_core::debugger::MemorySearch;

        if let Some(ref dbg) = self.debugger {
            // Parse hex pattern
            let pattern = MemorySearch::parse_hex_pattern(&self.search_state.input)
                .map_err(|e| format!("Invalid hex pattern: {}", e))?;

            // Search in memory (0x0 to 0x10000 for now)
            let results = MemorySearch::search_bytes(&dbg.memory, &pattern, 0x0, 0x10000)
                .map_err(|e| format!("Search error: {}", e))?;

            self.search_state.results = results;
            self.search_state.selected_result = 0;

            self.message = Some(Message {
                text: format!("Found {} matches", self.search_state.results.len()),
                is_error: false,
            });

            Ok(())
        } else {
            Err("No debugger active".to_string())
        }
    }

    /// Perform string search
    pub fn search_string(&mut self) -> Result<(), String> {
        use revgame_core::debugger::MemorySearch;

        if let Some(ref dbg) = self.debugger {
            let results = MemorySearch::search_string(
                &dbg.memory,
                &self.search_state.input,
                0x0,
                0x10000,
                self.search_state.case_sensitive,
            )
            .map_err(|e| format!("Search error: {}", e))?;

            self.search_state.results = results;
            self.search_state.selected_result = 0;

            self.message = Some(Message {
                text: format!("Found {} matches", self.search_state.results.len()),
                is_error: false,
            });

            Ok(())
        } else {
            Err("No debugger active".to_string())
        }
    }

    /// Find all strings in memory
    pub fn find_strings(&mut self) -> Result<(), String> {
        use revgame_core::debugger::MemorySearch;

        if let Some(ref dbg) = self.debugger {
            let results = MemorySearch::find_strings(
                &dbg.memory,
                self.search_state.min_string_length,
                0x0,
                0x10000,
            )
            .map_err(|e| format!("Search error: {}", e))?;

            self.search_state.results = results;
            self.search_state.selected_result = 0;

            self.message = Some(Message {
                text: format!("Found {} strings", self.search_state.results.len()),
                is_error: false,
            });

            Ok(())
        } else {
            Err("No debugger active".to_string())
        }
    }

    /// Jump to selected search result
    pub fn goto_search_result(&mut self) {
        if let Some(address) = self.search_state.get_selected_address() {
            self.memory_view_addr = address;
            self.search_dialog_open = false;

            self.message = Some(Message {
                text: format!("Jumped to 0x{:08X}", address),
                is_error: false,
            });
        }
    }

    /// Toggle bookmark at current disassembly address
    pub fn toggle_bookmark_at_cursor(&mut self) {
        if let Some(ref mut dbg) = self.debugger {
            if self.disasm_selection < self.disasm_cache.len() {
                let address = self.disasm_cache[self.disasm_selection].address;
                let added = dbg.bookmarks.toggle(address, format!("Address 0x{:08X}", address));

                self.message = Some(Message {
                    text: if added {
                        format!("Bookmark added at 0x{:08X}", address)
                    } else {
                        format!("Bookmark removed at 0x{:08X}", address)
                    },
                    is_error: false,
                });
            }
        }
    }

    /// Go to next bookmark
    pub fn goto_next_bookmark(&mut self) {
        if let Some(ref dbg) = self.debugger {
            let current_addr = if self.disasm_selection < self.disasm_cache.len() {
                self.disasm_cache[self.disasm_selection].address
            } else {
                self.memory_view_addr
            };

            if let Some(next_addr) = dbg.bookmarks.next_after(current_addr) {
                self.memory_view_addr = next_addr;
                self.message = Some(Message {
                    text: format!("Jumped to bookmark at 0x{:08X}", next_addr),
                    is_error: false,
                });
            } else {
                self.message = Some(Message {
                    text: "No more bookmarks after current address".to_string(),
                    is_error: false,
                });
            }
        }
    }

    /// Go to previous bookmark
    pub fn goto_prev_bookmark(&mut self) {
        if let Some(ref dbg) = self.debugger {
            let current_addr = if self.disasm_selection < self.disasm_cache.len() {
                self.disasm_cache[self.disasm_selection].address
            } else {
                self.memory_view_addr
            };

            if let Some(prev_addr) = dbg.bookmarks.prev_before(current_addr) {
                self.memory_view_addr = prev_addr;
                self.message = Some(Message {
                    text: format!("Jumped to bookmark at 0x{:08X}", prev_addr),
                    is_error: false,
                });
            } else {
                self.message = Some(Message {
                    text: "No more bookmarks before current address".to_string(),
                    is_error: false,
                });
            }
        }
    }

    /// Delete selected bookmark
    pub fn delete_selected_bookmark(&mut self) {
        if let Some(ref mut dbg) = self.debugger {
            let bookmarks = dbg.bookmarks.list();
            if self.bookmarks_view_state.selected < bookmarks.len() {
                let address = bookmarks[self.bookmarks_view_state.selected].address;
                dbg.bookmarks.remove(address);

                // Adjust selection if needed
                if self.bookmarks_view_state.selected > 0
                    && self.bookmarks_view_state.selected >= dbg.bookmarks.count()
                {
                    self.bookmarks_view_state.selected -= 1;
                }

                self.message = Some(Message {
                    text: format!("Bookmark deleted at 0x{:08X}", address),
                    is_error: false,
                });
            }
        }
    }

    /// Start editing selected bookmark
    pub fn start_editing_bookmark(&mut self) {
        if let Some(ref dbg) = self.debugger {
            let bookmarks = dbg.bookmarks.list();
            if self.bookmarks_view_state.selected < bookmarks.len() {
                let bookmark = bookmarks[self.bookmarks_view_state.selected];
                self.bookmarks_view_state
                    .start_editing(bookmark.address, bookmark.note.clone());
            }
        }
    }

    /// Save edited bookmark
    pub fn save_edited_bookmark(&mut self) {
        if let Some(ref editing) = self.bookmarks_view_state.editing.clone() {
            if let Some(ref mut dbg) = self.debugger {
                dbg.bookmarks.update_note(editing.address, &editing.note);
                self.bookmarks_view_state.cancel_editing();

                self.message = Some(Message {
                    text: format!("Bookmark updated at 0x{:08X}", editing.address),
                    is_error: false,
                });
            }
        }
    }

    /// Jump to selected bookmark in bookmarks dialog
    pub fn goto_selected_bookmark(&mut self) {
        if let Some(ref dbg) = self.debugger {
            let bookmarks = dbg.bookmarks.list();
            if self.bookmarks_view_state.selected < bookmarks.len() {
                let address = bookmarks[self.bookmarks_view_state.selected].address;
                self.memory_view_addr = address;
                self.bookmarks_dialog_open = false;

                self.message = Some(Message {
                    text: format!("Jumped to 0x{:08X}", address),
                    is_error: false,
                });
            }
        }
    }
}
