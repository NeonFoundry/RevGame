# Changelog

All notable changes to RevGame will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **x86 Instruction Reference** - Built-in manual with 27 documented instructions across 7 categories
  - Navigate by category or search specific instructions
  - Syntax examples, descriptions, and flag effects
  - Accessible via [R] from menu or [F1] during gameplay
- **Search Functionality** - Advanced memory search capabilities
  - Byte pattern search with hex input
  - String search (case-sensitive/insensitive)
  - Auto-detect null-terminated strings in memory
  - Keyboard shortcut: [Ctrl+F]
- **Bookmarks System** - Mark and annotate memory addresses
  - Add/remove bookmarks at any address
  - Edit notes for each bookmark
  - Navigate between bookmarks (next/previous)
  - View all bookmarks in organized list
  - Keyboard shortcuts: [Ctrl+B] toggle, [M] list, [N]/[P] navigate
- **Syntax Highlighting** - Color-coded instructions by category
  - Data Movement (Cyan): MOV, LEA, XCHG
  - Arithmetic (Green): ADD, SUB, MUL, DIV, INC, DEC
  - Logic (Yellow): AND, OR, XOR, NOT, TEST
  - Control Flow (Red): JMP, CALL, RET, LOOP
  - Conditional (Light Red): Jcc, CMOVcc, SETcc
  - Stack (Magenta): PUSH, POP
  - Comparison (Light Yellow): CMP
  - Shift (Blue): SHL, SHR, ROL, ROR
  - Special (Gray): NOP, HLT, INT
- **Save/Load System** - Persistent game progress
  - Save puzzle state to disk (achievements, progress, statistics)
  - Quick save/load functionality
  - Platform-specific save directories
  - Keyboard shortcuts: [F6]/[Ctrl+S] save, [F7]/[Ctrl+L] load
- **Additional Puzzles** - 4 new challenging puzzles
  - XOR decryption challenge
  - String comparison puzzle
  - Array checksum with loops
  - Multi-stage validation crackme

### Planned
- Web version (WASM support)
- More puzzle categories (functions, advanced)
- Settings menu with key remapping

## [0.1.0] - 2026-01-27

### Added

#### Core Features
- **Real x86 Emulator** - 30+ x86 instructions implemented
  - Arithmetic: MOV, ADD, SUB, INC, DEC, NEG, IMUL, XOR, OR, AND, NOT, TEST
  - Control Flow: JMP, Jcc (JE, JNE, JG, JL, JGE, JLE, JA, JB, etc.), CALL, RET
  - Stack: PUSH, POP, PUSHAD, POPAD
  - Special: NOP, INT, HLT
- **CPU State Management** - Registers, flags (ZF, SF, CF, OF), EIP
- **Memory System** - Segmented memory with code/data/stack regions

#### Debugger
- Step-by-step execution (F10)
- Run until halt/breakpoint (F5)
- Breakpoint system
- Memory patching with hex values
- Reset to initial state (F4)
- **Undo/Redo System** - Up to 100 actions with full state history
- **VHS Rewind Effect** - Retro visual effect on undo

#### Achievement System
- 15 unique achievements with icons and points
- Statistics tracking: patches, hints, time, undos
- Category completion detection
- Win streak tracking
- Special achievements:
  - One Shot (100 pts) - Single patch solve
  - Speed Runner (50 pts) - Beat par time
  - Self Taught (25 pts) - No hints used
  - Time Traveler (25 pts) - 10 undos
  - Win Streaks (50-500 pts)

#### User Interface
- **Beautiful TUI** - Catppuccin-inspired color scheme
- Disassembly view with current instruction highlighting
- Register panel with change tracking
- Memory hex viewer with ASCII display
- Stack visualization
- Command-line interface
- Status bar with keyboard shortcuts
- **Interactive Tutorial** - Visual arrows pointing to UI elements

#### Puzzles
- 11 puzzles across 3 categories:
  - 5 Basic puzzles (NOPs, immediate values, jumps)
  - 5 Control flow puzzles (loops, flags, arithmetic)
  - 1 Crackme puzzle (serial validation)
- TOML-based puzzle format
- 3-level hint system per puzzle
- Prerequisite system
- Validation engine (register, memory, flags)

#### Documentation
- Comprehensive README
- Contributing guidelines
- Puzzle creation guide
- MIT License

### Technical
- **Architecture** - Clean separation: core → UI → native binary
- **Testing** - 28 passing unit tests
- **Dependencies** - iced-x86 for disassembly, ratatui for TUI
- **Rust 2021 Edition** - Modern Rust features

### Developer Experience
- Modular crate structure
- Extensive inline documentation
- Error handling with thiserror
- Serde for serialization

---

## Version Legend

- **Major version (X.0.0)** - Breaking changes, major rewrites
- **Minor version (0.X.0)** - New features, backwards compatible
- **Patch version (0.0.X)** - Bug fixes, small improvements

[Unreleased]: https://github.com/yourusername/revgame/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/revgame/releases/tag/v0.1.0
