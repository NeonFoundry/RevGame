# Changelog

All notable changes to RevGame will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Built-in x86 instruction reference
- Search functionality (bytes, strings, instructions)
- Save/load puzzle progress
- More puzzle categories
- Syntax highlighting for instruction types

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
